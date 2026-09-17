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
}
