mod cli;

use {clap::Parser, cli::Config};
fn main() {
    let config = Config::parse();
}
