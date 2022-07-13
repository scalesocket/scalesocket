use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Interface to bind to
    #[clap(long, default_value = "0.0.0.0:9000")]
    pub addr: SocketAddr,

    /// Increase level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    pub verbosity: usize,

    /// Command to wrap
    #[clap(required = true)]
    pub cmd: String,

    /// Arguments to command
    #[clap(last = true)]
    pub args: Vec<String>,
}
