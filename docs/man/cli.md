---
hide_side_table_of_contents: true
---
# Command Line Arguments

```console
$ scalesocket --help
scalesocket 0.1.4
A websocket server and autoscaler

USAGE:
    scalesocket [OPTIONS] <CMD> [-- <ARGS>...]

ARGS:
    <CMD>
            Command to wrap

    <ARGS>...
            Arguments to command

OPTIONS:
        --addr <ADDR>
            Interface to bind to
            
            [default: 0.0.0.0:9000]

    -b, --binary
            Set scalesocket to experimental binary mode

        --client-frame=<MODE>
            Enable framing and routing for client originated messages
            
            See --frame for options.

        --cmd-attach-delay <SECONDS>
            Delay before attaching to child [default: 1 for --tcp]

        --frame[=<MODE>...]
            Enable framing and routing for all messages
            
            Client messages are tagged with an ID header (u32). Server messages with optional client
            ID are routed to clients.
            
            When set to `json` messages are parsed as JSON. Client messages are amended with an "id"
            field. Server messages are routed to clients based an optional "id" field. When set to
            `binary` messages are parsed according to gwsocket's strict mode. Unparseable messages
            may be dropped.
            
            See --server-frame and --client-frame for specifying framing independently.
            
            [default: binary when set, possible values: binary, json]

    -h, --help
            Print help information

        --joinmsg <MSG>
            Emit message to child on client connect (use #ID for id)

        --json
            Log JSON

        --leavemsg <MSG>
            Emit message to child on client disconnect (use #ID for id)

        --metrics
            Expose OpenMetrics endpoint at /metrics

        --oneshot
            Serve only once

        --passenv <LIST>
            List of envvars to pass to child
            
            [default: PATH,DYLD_LIBRARY_PATH]

        --server-frame=<MODE>
            Enable framing and routing for server originated messages
            
            See --frame for options.

        --staticdir <DIR>
            Serve static files from directory over HTTP

        --stats
            Expose stats endpoint at /<ROOM>/stats
            
            Exposed statistics can be queried individually at  /<ROOM>/stats/<STATISTIC>

        --tcp
            Connect to child using TCP instead of stdio. Use PORT to bind

        --tcpports <START:END>
            Port range for TCP
            
            [default: 9001:9999]

    -v
            Increase level of verbosity

    -V, --version
            Print version information

```