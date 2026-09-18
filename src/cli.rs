use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "forth-lsp",
    about = "LSP for the Forth programming language",
    disable_version_flag = true
)]
pub struct Cli {
    /// Print version information
    #[arg(short = 'V', long = "version")]
    pub version: bool,

    /// Extraneous arguments passed by LSP clients (e.g. --stdio); accepted and ignored
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub client_args: Vec<String>,
}
