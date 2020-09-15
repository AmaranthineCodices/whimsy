use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum CliCommand {
    #[structopt(
        name = "open-cfg",
        about = "Opens the whimsy configuration file in the default text editor for YAML files."
    )]
    OpenConfigFile,
    #[structopt(
        name = "regenerate-cfg",
        about = "Restores the whimsy configuration file to the default."
    )]
    RegenerateConfigFile,
}

#[derive(StructOpt, Debug)]
#[structopt(about, author)]
pub struct CliOptions {
    #[structopt(subcommand)]
    pub command: Option<CliCommand>,
    #[structopt(
        short,
        long,
        env = "WHIMSY_CFG",
        default_value(crate::config::DEFAULT_CONFIG_PATH.to_str().unwrap())
    )]
    /// The path to the whimsy configuration file to use.
    pub config_file: PathBuf,
}
