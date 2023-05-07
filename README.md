# Scalesocket

[![Build status](https://github.com/scalesocket/scalesocket/actions/workflows/ci.yml/badge.svg)](https://github.com/scalesocket/scalesocket/actions)
[![Crates.io](https://img.shields.io/crates/v/scalesocket.svg)](https://crates.io/crates/scalesocket)

> *Scalesocket* is a websocket server and autoscaler. It is an easy way to build multiplayer backends.


## About

Scalesocket enables you to wrap a script or binary, and serve it over websockets. Clients then connect to *rooms* at `wss://example.com/exampleroom`. Connecting to a room spawns a new process of the wrapped binary. Subsequent connections to the same room share the process.

For full details, see the [documentation](https://www.scalesocket.org/docs.html).


## Features

* Share a backend process between websocket clients
* Proxy websocket traffic to normal TCP socket or stdio
* Route server messages to specific clients
* Serve static files
* Expose CGI [environment variables](https://www.rfc-editor.org/rfc/rfc3875.html) to backend process
* [OpenMetrics](https://github.com/OpenObservability/OpenMetrics) compatible


## Usage

Create the file `example.sh` with the follow content:
```console,ignore
#!/bin/bash
echo '{"message": "hello world"}'
sleep 1
echo '{"message": "goodbye"}'
sleep 1
```

Make it executable:
```console,ignore
$ chmod u+x example.sh
```

Wrap it by starting the ScaleSocket server:
```console,ignore
$ scalesocket ./example.sh
```

Then connect to the websocket endpoint, for example using curl:
```console,ignore
$ curl --include \
       --no-buffer \
       --http1.1 \
       --header "Connection: Upgrade" \
       --header "Upgrade: websocket" \
       --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
       --header "Sec-WebSocket-Version: 13" \
       http://localhost:9000/exampleroom
�{"message": "hello world"}�{"message": "goodbye"}%
```


## Command line arguments

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

        --cmd-attach-delay <SECONDS>
            Delay before attaching to child [default: 1 for --tcp]

        --frame[=<MODE>...]
            Enable framing and routing for messages
            
            Client messages are amended with ID header (u32). Server messages with optional client
            ID are routed to clients.
            
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

        --oneshot
            Serve only once

        --passenv <LIST>
            List of envvars to pass to child
            
            [default: PATH,DYLD_LIBRARY_PATH]

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
