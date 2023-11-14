use {
    clap::builder::ArgPredicate,
    clap::{ArgAction, Parser},
    std::net::SocketAddr,
    std::ops::Range,
    std::path::PathBuf,
};

use crate::types::{Frame, Log};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Interface to bind to
    #[clap(long, default_value = "0.0.0.0:9000")]
    pub addr: SocketAddr,

    /// Set scalesocket to experimental binary mode
    #[clap(short, long, action)]
    pub binary: bool,

    /// Delay before attaching to child [default: 1 for --tcp]
    #[clap(
        long = "delay",
        value_name = "SECONDS",
        default_value_if("tcp",  ArgPredicate::Equals("true".into()), Some("1"))
    )]
    pub delay: Option<u64>,

    /// Emit message to child on client connect (use #ID for id)
    #[clap(
        long,
        value_name = "MSG",
        default_value_if("json",  ArgPredicate::Equals("true".into()), Some(r#"{"t":"Join","_from":#ID}"#))
    )]
    pub joinmsg: Option<String>,

    /// Enable JSON framing with default join and leave messages
    ///
    /// This option is equivalent to
    /// --frame=json --joinmsg '{"t":"Join","_from":#ID}' --leavemsg '{"t":"Leave","_from":#ID}'
    #[clap(
        long,
        action,
        conflicts_with = "client_frame",
        conflicts_with = "server_frame",
        conflicts_with = "frame"
    )]
    pub json: bool,

    /// Emit message to child on client disconnect (use #ID for id)
    #[clap(
        long,
        value_name = "MSG",
        default_value_if("json",  ArgPredicate::Equals("true".into()), Some(r#"{"t":"Leave","_from":#ID}"#))
    )]
    pub leavemsg: Option<String>,

    /// Log format
    ///
    /// [default: text, possible values: text, json]
    #[clap(
        long,
        action,
        value_parser,
        value_name = "FMT",
        default_value = "text",
        hide_possible_values = true,
        hide_default_value = true
    )]
    pub log: Log,

    /// Expose OpenMetrics endpoint at /metrics
    #[clap(long, action)]
    pub metrics: bool,

    /// Serve only once.
    #[clap(long)]
    pub oneshot: bool,

    /// List of envvars to pass to child
    #[clap(
        long,
        value_name = "LIST",
        value_delimiter = ',',
        default_value = "PATH,DYLD_LIBRARY_PATH"
    )]
    pub passenv: Vec<String>,

    /// Enable framing and routing for all messages
    ///
    /// Client messages are tagged with an ID header (u32). Server messages with optional client ID are routed to clients.
    ///
    /// When set to `json`, messages are parsed as JSON.
    /// Client messages are amended with an "_from" field.
    /// Server messages are routed to clients based an optional "_to" field.
    ///
    /// Server messages with `_meta: true` will be dropped, and stored as room metadata accessible via the API.
    ///
    /// When set to `binary`, messages are parsed according to gwsocket's strict mode.
    /// Unparseable messages may be dropped.
    ///
    /// See --serverframe and --clientframe for specifying framing independently.
    ///
    /// [default: binary when set, possible values: binary, json]
    #[clap(
        long,
        alias = "strict",
        value_parser,
        value_name = "MODE",
        default_missing_value = "binary",
        default_value_if("json",  ArgPredicate::Equals("true".into()), Some("json")),
        num_args = 0..,
        require_equals = true,
        hide_possible_values = true
    )]
    pub frame: Option<Frame>,

    /// Enable framing and routing for client originated messages
    ///
    /// See --frame for options.
    #[clap(
        long = "clientframe",
        value_parser,
        value_name = "MODE",
        conflicts_with = "frame",
        require_equals = true,
        hide_possible_values = true
    )]
    pub client_frame: Option<Frame>,

    /// Enable framing and routing for server originated messages
    ///
    /// See --frame for options.
    #[clap(
        long = "serverframe",
        value_parser,
        value_name = "MODE",
        conflicts_with = "frame",
        require_equals = true,
        hide_possible_values = true
    )]
    pub server_frame: Option<Frame>,

    /// Serve static files from directory over HTTP
    #[clap(long, value_parser, value_name = "DIR")]
    pub staticdir: Option<PathBuf>,

    /// Expose room metadata API under /api/
    ///
    /// The exposed endpoints are:
    /// * /api/rooms/          - list rooms
    /// * /api/<ROOM>/         - get room metadata
    /// * /api/<ROOM>/<METRIC> - get room individual metric
    #[clap(long, action, verbatim_doc_comment)]
    pub api: bool,

    /// Connect to child using TCP instead of stdio. Use PORT to bind
    #[clap(long, action)]
    pub tcp: bool,

    /// Port range for TCP
    #[clap(long, value_parser = parse_ports, value_name = "START:END", default_value = "9001:9999")]
    pub tcpports: Range<u16>,

    /// Increase level of verbosity
    #[clap(short, action = ArgAction::Count)]
    pub verbosity: u8,

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
