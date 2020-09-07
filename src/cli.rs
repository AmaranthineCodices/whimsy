use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum CliCommand {
    #[structopt(
        name = "open-cfg",
        about = "Opens the whimsy configuration file in the default text editor for TOML files."
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
}
