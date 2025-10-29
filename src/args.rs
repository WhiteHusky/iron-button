use clap::Parser;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[clap(author = "Carlen White", version, about)]
/// Application configuration
pub struct Args {
    /// whether to be verbose
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// reshow the XDG portal configuration dialog
    #[arg(long)]
    pub show_portal_config: bool,

    /// override XDG_CONFIG_HOME/iron-button/config.yml with a different path
    #[arg(long)]
    pub config: Option<PathBuf>,
}
