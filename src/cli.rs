use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about, author)]
pub enum CliOptions {
    #[structopt(
        name = "open-cfg",
        about = "Opens the whimsy configuration file in the default text editor for TOML files."
    )]
    OpenConfigFile,
}
