# Scalesocket

[![Crates.io](https://img.shields.io/crates/v/scalesocket.svg)](https://crates.io/crates/scalesocket)

*Scalesocket* is a websocket server and autoscaler.

## Features

* Share a backend process between clients
* Proxy websocket traffic to normal TCP socket or stdio
* Route server messages to specific clients
* Serve static files
* Expose CGI [environment variables](https://www.rfc-editor.org/rfc/rfc3875.html) to backend process
* [OpenMetrics](https://github.com/OpenObservability/OpenMetrics) compatible


## Usage

```console
$ scalesocket --help
scalesocket 0.1.2
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

        --cmd-attach-delay <SECONDS>
            Delay before attaching to child [default: 1 for --tcp]

        --frame[=<MODE>...]
            Enable framing and routing for messages
            
            Client messages are amended with ID header. Server messages with optional client ID
            routed to clients.
            
            When set to `json` messages are parsed as JSON. Client messages are amended with an "id"
            field. Server messages are routed to clients based an optional "id" field. When set to
            `binary` messages are parsed according to gwsocket's strict mode. Unparseable messages
            are dropped.
            
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

        --passenv <LIST>
            List of envvars to pass to child
            
            [default: PATH,DYLD_LIBRARY_PATH]

        --staticdir <DIR>
            Serve static files from directory over HTTP

        --stats
            Expose stats endpoint at /<ROOM>/stats

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