---
hide_side_table_of_contents: true
title: "Command Line Arguments"
---
# Command Line Arguments

```console
$ scalesocket --help
A websocket server and autoscaler

Usage: scalesocket [OPTIONS] <CMD> [-- <ARGS>...]

Arguments:
  <CMD>
          Command to wrap

  [ARGS]...
          Arguments to command

Options:
      --addr <ADDR>
          Interface to bind to
          
          [default: 0.0.0.0:9000]

  -b, --binary
          Set scalesocket to experimental binary mode

      --json
          Log JSON

      --joinmsg <MSG>
          Emit message to child on client connect (use #ID for id)

      --leavemsg <MSG>
          Emit message to child on client disconnect (use #ID for id)

      --metrics
          Expose OpenMetrics endpoint at /metrics

      --oneshot
          Serve only once

      --passenv <LIST>
          List of envvars to pass to child
          
          [default: PATH,DYLD_LIBRARY_PATH]

      --frame[=<MODE>...]
          Enable framing and routing for all messages
          
          Client messages are tagged with an ID header (u32). Server messages with optional client ID are routed to clients.
          
          When set to `json` messages are parsed as JSON. Client messages are amended with an "_from" field. Server messages are routed to clients based an optional "_to" field. When set to `binary` messages are parsed according to gwsocket's strict mode. Unparseable messages may be dropped.
          
          See --server-frame and --client-frame for specifying framing independently.
          
          [default: binary when set, possible values: binary, json]

      --client-frame=<MODE>
          Enable framing and routing for client originated messages
          
          See --frame for options.

      --server-frame=<MODE>
          Enable framing and routing for server originated messages
          
          See --frame for options.

      --staticdir <DIR>
          Serve static files from directory over HTTP

      --api
          Expose room metadata API under /api/
          
          The exposed endpoints are:
          * /api/rooms/          - list rooms
          * /api/<ROOM>/         - get room metadata
          * /api/<ROOM>/<METRIC> - get room individual metric

      --tcpports <START:END>
          Port range for TCP
          
          [default: 9001:9999]

      --tcp
          Connect to child using TCP instead of stdio. Use PORT to bind

  -v...
          Increase level of verbosity

      --cmd-attach-delay <SECONDS>
          Delay before attaching to child [default: 1 for --tcp]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```