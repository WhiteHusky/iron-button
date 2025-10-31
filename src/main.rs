use ashpd::desktop::global_shortcuts::{Activated, Deactivated, GlobalShortcuts, NewShortcut};
use ashpd::desktop::Session;
use clap::Parser;
use futures::stream::StreamExt;
use futures::Stream;
use sd_notify::{notify, NotifyState};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Termination};
use std::sync::{Arc, Mutex};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

mod args;
use crate::args::Args;

mod config;
use crate::config::{Action, Configuration};

mod errors;
use crate::errors::Error;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(error) = _main().await {
        error.report()
    } else {
        ExitCode::SUCCESS
    }
}

async fn _main() -> Result<(), Error> {
    let args = Args::parse();
    if args.verbose {
        eprintln!("DEBUG {args:?}");
    };

    let global_shortcuts = GlobalShortcuts::new().await?;
    let session = global_shortcuts.create_session().await?;

    if args.show_portal_config {
        global_shortcuts
            .configure_shortcuts(&session, None, None)
            .await?;
        return Ok(());
    }

    let config_path = if let Some(path) = args.config {
        path
    } else {
        dirs::config_dir()
            .expect("can not locate config directory")
            .join("iron-button/config.yml")
    };

    let config = read_config(&config_path)?;

    let shortcuts = collect_shortcuts(&config);

    global_shortcuts
        .bind_shortcuts(&session, &shortcuts, None)
        .await?
        .response()?;

let _ = notify(false, &[NotifyState::Ready]).inspect_err(|e| eprintln!("{e}"));

    let config = Arc::new(Mutex::new(config));

    select! {
        r = handle_activations(global_shortcuts.receive_activated().await?, config.clone()) => {r?}
        r = handle_deactivations(global_shortcuts.receive_deactivated().await?, config.clone()) => {r?}
        r = handle_signals(global_shortcuts, session, config.clone(), config_path) => {r?}
    }

    Ok(())
}

fn collect_shortcuts(config: &Configuration) -> Vec<NewShortcut> {
    let mut shortcuts = Vec::new();
    for (id, bind) in &config.binds {
        let mut shortcut = NewShortcut::new(id, bind.description.as_ref().unwrap_or(id));
        if let Some(suggested_bind) = &bind.suggest {
            shortcut = shortcut.preferred_trigger(suggested_bind.as_str());
        }
        shortcuts.push(shortcut);
    }
    shortcuts
}

fn read_config(config_path: &PathBuf) -> Result<Configuration, Error> {
    Ok(serde_yaml_ng::from_str(
        fs::read_to_string(config_path)
            .map_err(Error::ConfigReadError)?
            .as_str(),
    )
    .map_err(Error::ConfigParseError)?)
}

async fn handle_activations(
    mut activations: impl Stream<Item = Activated> + Unpin,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), Error> {
    let config = config.clone();
    while let Some(activated) = activations.next().await {
        let config = config.lock().unwrap();
        if let Some(bind) = config.binds.get(activated.shortcut_id()) {
            if let Some(action) = &bind.on_down {
                run_action(action.clone());
            }
        } else {
            eprintln!("received unknown bind")
        }
    }
    Err(Error::UnexpectedEndOfKeys)
}

async fn handle_deactivations(
    mut deactivations: impl Stream<Item = Deactivated> + Unpin,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), Error> {
    let config = config.clone();
    while let Some(deactivated) = deactivations.next().await {
        let config = config.lock().unwrap();
let shortcut_id = deactivated.shortcut_id();
        if let Some(bind) = config.binds.get(shortcut_id) {
            if let Some(action) = &bind.on_up {
                run_action(action.clone());
            }
        } else {
            eprintln!("received unknown bind")
        }
    }
    Err(Error::UnexpectedEndOfKeys)
}

async fn handle_signals(
    global_shortcuts: GlobalShortcuts<'_>,
    session: Session<'_, GlobalShortcuts<'_>>,
    config: Arc<Mutex<Configuration>>,
    config_path: PathBuf,
) -> Result<(), Error> {
    let mut hangup = signal(SignalKind::hangup()).unwrap();

    loop {
        hangup.recv().await;
        eprintln!("reloading configuration...");
let _ = notify(false, &[NotifyState::Reloading]).inspect_err(|e| eprintln!("{e}"));
        let mut locked_config = config.lock().unwrap();
        match read_config(&config_path) {
            Ok(new_config) => {
                let new_shortcuts = collect_shortcuts(&new_config);
                *locked_config = new_config;
                global_shortcuts
                    .bind_shortcuts(&session, &new_shortcuts, None)
                    .await?
                    .response()?;
            }
            Err(err) => {
                drop(locked_config); // Don't wait on us.
                eprintln!("reloading config failed, see next line\n{err}");
            }
        }
let _ = notify(false, &[NotifyState::monotonic_usec_now().unwrap()])
            .inspect_err(|e| eprintln!("{e}"));
    }
}

fn run_action(action: Action) {
    match action {
        Action::Run(run) => {
            let mut command = Command::new(&run.program);
            if let Some(arguments) = &run.arguments {
                command.args(arguments);
            }
            if let Err(error) = command.spawn() {
                eprintln!("spawning command failed due to: {error}");
            }
        }
    };
}
