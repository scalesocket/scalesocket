use {clap::Parser, std::net::SocketAddr, std::path::PathBuf};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Interface to bind to
    #[clap(long, default_value = "0.0.0.0:9000")]
    pub addr: SocketAddr,

    /// Increase level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    pub verbosity: usize,

    /// Serve static files from directory over HTTP
    #[clap(long, value_parser, value_name = "DIR")]
    pub staticdir: Option<PathBuf>,

    /// Log JSON
    #[clap(short, long, action)]
    pub json: bool,

    /// Command to wrap
    #[clap(required = true)]
    pub cmd: String,

    /// Arguments to command
    #[clap(last = true)]
    pub args: Vec<String>,
}
