use miden_client_cli::Cli;

extern crate std;

#[tokio::main]
async fn main() -> miette::Result<()> {
    use clap::Parser;

    // read command-line args
    let cli = Cli::parse();

    // execute cli action
    Ok(cli.execute().await?)
}
