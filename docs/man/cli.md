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

      --cache <[TYPE:]SIZE>
          Cache server message history for room and replay it to new clients
          
          The cache buffer retains the last <SIZE> chunks, determined by <TYPE>.
          
          When set to `all`, all server messages are cached.
          When set to `tagged`, only server messages with `_cache: true` are cached.

      --cachepersist
          Preserve server message history for room even after last client disconnects

      --delay <SECONDS>
          Delay before attaching to child [default: 1 for --tcp]

      --joinmsg <MSG>
          Emit message to child on client connect (use #ID for id)

      --json
          Enable JSON framing with default join and leave messages
          
          This option is equivalent to --frame=json --joinmsg '{"t":"Join","_from":#ID}' --leavemsg '{"t":"Leave","_from":#ID}'

      --leavemsg <MSG>
          Emit message to child on client disconnect (use #ID for id)

      --log <FMT>
          Log format
          
          [default: text, possible values: text, json]

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
          
          When set to `json`, messages are parsed as JSON. Client messages are amended with an "_from" field. Server messages are routed to clients based an optional "_to" field.
          
          Server messages with `_meta: true` will be dropped, and stored as room metadata accessible via the API.
          
          When set to `gwsocket`, messages are parsed according to gwsocket's strict mode. Unparseable messages may be dropped.
          
          See --serverframe and --clientframe for specifying framing independently.
          
          [default: gwsocket when set, possible values: gwsocket, json]

      --clientframe=<MODE>
          Enable framing and routing for client originated messages
          
          See --frame for options.

      --serverframe=<MODE>
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

      --tcp
          Connect to child using TCP instead of stdio. Use PORT to bind

      --tcpports <START:END>
          Port range for TCP
          
          [default: 9001:9999]

  -v...
          Increase level of verbosity

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```