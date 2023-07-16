# Scalesocket

[![Build status](https://github.com/scalesocket/scalesocket/actions/workflows/ci.yml/badge.svg)](https://github.com/scalesocket/scalesocket/actions)
[![Crates.io](https://img.shields.io/crates/v/scalesocket.svg)](https://crates.io/crates/scalesocket)

> *Scalesocket* is a websocket server and autoscaler. It is an easy way to build multiplayer backends.


## About

ScaleSocket lets you to wrap a script or binary, and serve it over websockets. Clients then connect to *rooms* at `wss://example.com/exampleroom`. Connecting to a room spawns a new process of the wrapped binary. Subsequent connections to the same room share the process.

For full details, see the [documentation](https://www.scalesocket.org/docs.html).


## Features

* Share a backend process between websocket clients
* Proxy websocket traffic to normal TCP socket or stdio
* Route server messages to specific clients
* Serve static files
* Expose CGI [environment variables](https://www.rfc-editor.org/rfc/rfc3875.html) to backend process
* [OpenMetrics](https://github.com/OpenObservability/OpenMetrics) compatible


## Quick Start

Create the file `example.sh` with the follow content:
```console
#!/bin/bash
echo '{"message": "hello world"}'
sleep 1
echo '{"message": "goodbye"}'
sleep 1
```

Make it executable:
```console
$ chmod u+x example.sh
```

Wrap it by starting the ScaleSocket server:
```console
$ scalesocket ./example.sh
```

Then connect to the websocket endpoint, for example using curl:
```console
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

For more advanced usage and features, see [features](/man/features.md).
