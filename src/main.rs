use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use clap::Parser;
use futures::stream::StreamExt;
use std::fs;
use std::process::{Command, ExitCode, Termination};
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;

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

    let config: Configuration = {
        let config_path = if let Some(path) = args.config {
            path
        } else {
            dirs::config_dir()
                .expect("can not locate config directory")
                .join("iron-button/config.yml")
        };
        serde_yaml_ng::from_str(
            fs::read_to_string(config_path)
                .map_err(Error::ConfigReadError)?
                .as_str(),
        )
        .map_err(Error::ConfigParseError)?
    };

    let shortcuts = {
        let mut shortcuts = Vec::new();
        for (id, bind) in &config.binds {
            let mut shortcut = NewShortcut::new(id, bind.description.as_ref().unwrap_or(id));
            if let Some(suggested_bind) = &bind.suggest {
                shortcut = shortcut.preferred_trigger(suggested_bind.as_str());
            }
            shortcuts.push(shortcut);
        }
        shortcuts
    };

    global_shortcuts
        .bind_shortcuts(&session, &shortcuts, None)
        .await?
        .response()?;

    let mut tasks = JoinSet::new();

    let config = Arc::new(Mutex::new(config));

    activation_thread(&mut tasks, &global_shortcuts, config.clone()).await?;
    deactivation_thread(&mut tasks, &global_shortcuts, config.clone()).await?;

    tasks.join_all().await;

    Ok(())
}

async fn activation_thread(
    tasks: &mut JoinSet<()>,
    global_shortcuts: &GlobalShortcuts<'_>,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), Error> {
    let mut rx_activated = global_shortcuts.receive_activated().await?;
    tasks.spawn(async move {
        let config = config.clone();
        while let Some(activated) = rx_activated.next().await {
            let config = config.lock().unwrap();
            let bind = config.binds.get(activated.shortcut_id()).unwrap();
            if let Some(action) = &bind.on_down {
                run_action(action.clone());
            }
        }
    });
    Ok(())
}

async fn deactivation_thread(
    tasks: &mut JoinSet<()>,
    global_shortcuts: &GlobalShortcuts<'_>,
    config: Arc<Mutex<Configuration>>,
) -> Result<(), Error> {
    let mut rx_deactivated = global_shortcuts.receive_deactivated().await?;
    tasks.spawn(async move {
        let config = config.clone();
        while let Some(deactivated) = rx_deactivated.next().await {
            let config = config.lock().unwrap();
            let bind = config.binds.get(deactivated.shortcut_id()).unwrap();
            if let Some(action) = &bind.on_up {
                run_action(action.clone());
            }
        }
    });
    Ok(())
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
