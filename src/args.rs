use clap::Parser;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[clap(author = "Carlen White", version, about)]
/// Application configuration
pub struct Args {
    /// Whether to be verbose
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Reshow the XDG portal configuration dialog
    #[arg(long)]
    pub show_portal_config: bool,

    /// Override XDG_CONFIG_HOME/iron-button/config.yml with a different path
    #[arg(long)]
    pub config: Option<PathBuf>,
}
