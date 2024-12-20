# ScaleSocket

[![Build status](https://github.com/scalesocket/scalesocket/actions/workflows/ci.yml/badge.svg)](https://github.com/scalesocket/scalesocket/actions)
[![Crates.io](https://img.shields.io/crates/v/scalesocket.svg)](https://crates.io/crates/scalesocket)

*ScaleSocket* is a collaborative websocket server and autoscaler. It's a simple way to build multiplayer backends.

![High level architecture diagram on ScaleSocket usage](https://github.com/scalesocket/scalesocket/blob/main/docs/_assets/diagram.svg?raw=true)


## About

ScaleSocket is a command line tool that lets you to wrap a backend script or binary, and serve it collaboratively over websockets.

Clients connect to *rooms* (a.k.a. channels) which have a unique URL (`wss://example.com/room1`). Connecting to a room spawns a new backend process.
Subsequent connections to the same room share the process.

The backend does not require any network code or room handling logic.

## Documentation

For full details and installation instructions, see the [documentation](https://www.scalesocket.org/man/).


## Features

* Share a backend process between websocket clients
* Proxy websocket traffic to normal TCP socket or stdio
* Route server messages to specific clients
* Serve static files
* Expose CGI [environment variables](https://www.rfc-editor.org/rfc/rfc3875.html) to backend process
* [OpenMetrics](https://github.com/OpenObservability/OpenMetrics) compatible
* Built-in lobby server for listing rooms


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

For more advanced usage and features, see [usage](https://www.scalesocket.org/man/usage).

## Alternatives and Comparison

See the [comparison](https://www.scalesocket.org/man/comparison).

## License

* Apache License, Version 2.0 ([LICENSE](https://github.com/scalesocket/scalesocket/blob/HEAD/LICENSE) or [www.apache.org](http://www.apache.org/licenses/LICENSE-2.0))

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
