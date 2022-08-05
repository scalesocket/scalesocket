use {clap::Parser, std::net::SocketAddr, std::ops::Range, std::path::PathBuf};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Interface to bind to
    #[clap(long, default_value = "0.0.0.0:9000")]
    pub addr: SocketAddr,

    /// Set scalesocket to experimental binary mode (default is line by line)
    #[clap(short, long, action)]
    pub binary: bool,

    /// Log JSON
    #[clap(short, long, action)]
    pub json: bool,

    /// Emit message to child on client connect (use %ID for id)
    #[clap(long, value_name = "MSG")]
    pub joinmsg: Option<String>,

    /// Emit message to child on client disconnect (use %ID for id)
    #[clap(long, value_name = "MSG")]
    pub leavemsg: Option<String>,

    /// Port range for TCP
    #[clap(long, parse(try_from_str = parse_ports), value_name = "START:END", default_value = "9001:9999")]
    pub tcpports: Range<u16>,

    /// Serve static files from directory over HTTP
    #[clap(long, value_parser, value_name = "DIR")]
    pub staticdir: Option<PathBuf>,

    /// Connect to child using TCP instead of stdio
    #[clap(long, action)]
    pub tcp: bool,

    /// Increase level of verbosity
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,

    /// Command to wrap
    #[clap(required = true)]
    pub cmd: String,

    /// Arguments to command
    #[clap(last = true)]
    pub args: Vec<String>,
}

fn parse_ports(arg: &str) -> Result<Range<u16>, &'static str> {
    if let Some((start, end)) = arg.split_once(':') {
        let range: (Option<u16>, Option<u16>) = (start.parse().ok(), end.parse().ok());
        if let (Some(start), Some(end)) = range {
            return Ok(start..end);
        }
    };
    Err("Could not parse port range")
}
