use crate::types::Framing;
use {clap::Parser, std::net::SocketAddr, std::ops::Range, std::path::PathBuf};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Interface to bind to
    #[clap(long, default_value = "0.0.0.0:9000")]
    pub addr: SocketAddr,

    /// Set scalesocket to experimental binary mode
    #[clap(short, long, action)]
    pub binary: bool,

    /// Log JSON
    #[clap(long, action)]
    pub json: bool,

    /// Emit message to child on client connect (use #ID for id)
    #[clap(long, value_name = "MSG")]
    pub joinmsg: Option<String>,

    /// Emit message to child on client disconnect (use #ID for id)
    #[clap(long, value_name = "MSG")]
    pub leavemsg: Option<String>,

    /// Expose OpenMetrics endpoint at /metrics
    #[clap(long, action)]
    pub metrics: bool,

    /// List of envvars to pass to child
    #[clap(
        long,
        value_name = "LIST",
        value_delimiter = ',',
        default_value = "PATH,DYLD_LIBRARY_PATH"
    )]
    pub passenv: Vec<String>,

    /// Enable framing and routing for messages
    ///
    /// Client messages are amended with ID header. Server messages with optional client ID routed to clients.
    ///
    /// When set to `json` messages are parsed as JSON. Client messages are amended with an "id" field. Server messages are routed to clients based an optional "id" field.
    /// When set to `binary` messages are parsed according to gwsocket's strict mode.
    /// Unparseable messages are dropped.
    ///
    /// [default: binary when set, possible values: binary, json]
    #[clap(
        long,
        alias = "strict",
        value_parser,
        value_name = "MODE",
        default_missing_value = "binary",
        min_values = 0,
        require_equals = true,
        hide_possible_values = true
    )]
    pub frame: Option<Framing>,

    /// Serve static files from directory over HTTP
    #[clap(long, value_parser, value_name = "DIR")]
    pub staticdir: Option<PathBuf>,

    /// Expose stats endpoint at /<ROOM>/stats
    #[clap(long, action)]
    pub stats: bool,

    /// Port range for TCP
    #[clap(long, parse(try_from_str = parse_ports), value_name = "START:END", default_value = "9001:9999")]
    pub tcpports: Range<u16>,

    /// Connect to child using TCP instead of stdio. Use PORT to bind
    #[clap(long, action)]
    pub tcp: bool,

    /// Increase level of verbosity
    #[clap(short, parse(from_occurrences))]
    pub verbosity: usize,

    /// Delay before attaching to child [default: 1 for --tcp]
    #[clap(
        long,
        value_name = "SECONDS",
        default_value_if("tcp", Some("true"), Some("1"))
    )]
    pub cmd_attach_delay: Option<u64>,

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
