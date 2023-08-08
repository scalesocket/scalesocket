# Usage

## Wrapping a Binary or Script

The command to spawn the target is specified as the argument to the `scalesocket` command.
The target can be a script or a binary.

```console
$ scalesocket ./example.sh
```

### Passing Arguments

If the target requires arguments, they can be specified after a `--` separator.

```console
$ scalesocket ./example.sh -- --arg1 --arg2
```

### STDIO and TCP Modes

By default, incoming websocket messages are written to the target's *stdin*.
The target's *stdout* is sent back to the websocket client.
Alternatively, the messages can be sent to the target over a TCP socket.

When using TCP mode, the target must be configured to bind to the port specified by the environment variable `PORT`.

See the [CLI Reference](/man/cli.md) and the `--tcp`, `--tcpports` and `--cmd-attach-delay` arguments for details.

## Rooms

Clients connecting to the server specify a room in the connection URL path.
The room ID is the first path component of the URL. For example `wss://example.com/exampleroom`.

Connecting to a room spawns a new process of the wrapped binary or script. Subsequent connections to the same room share the same process.

## Framing and Routing Messages

ScaleSocket can optionally parse, tag and route messages.

### JSON Framing

When `--frame=json` is enabled, messages will be parsed and routed, with the following rules:
* Messages from the client, that are not valid JSON are dropped
* Messages from the client are tagged with an `_from` field
* Messages from the server, that contain an `_to` field will be routed to the specific client

### Binary Framing

When `--frame=binary` is enabled, ScaleSocket is compatible with [gwsocket](https://gwsocket.io/). Messages will be parsed according to the [gwsocket strict mode](https://gwsocket.io/man#man-strict-mode), with the following rules:
* Messages from the client, with an invalid header are dropped
* Messages from the client must set the header type to `0x01` (text)
* Messages from the server, that contain a nonzero client `id` field will be routed to the specific client

See the [CLI Reference](/man/cli.md) and the `--frame`, `--server-frame` and `--client-frame` arguments for details.

## Join and Leave Messages

ScaleSocket can optionally send a message to the target when a client joins or leaves a room.

The messages support the variables:
* `#ID` eg. `123`
* `QUERY_XYZ` for each query parameter, `?xyz=`, in the connection URL.
* The [Environment variables](#environment-variables)

For example, starting scalesocket with:

```console
$ scalesocket --joinmsg '{"type":"Join","_from":#ID}' ./example.sh
```

Sends the message `{"type":"Join","_from":123}` to the server when a new client joins. This is useful for keeping track of connected clients.


See the [CLI Reference](/man/cli.md) and the `--joinmsg` and `--leavemsg` arguments for details.

## Environment Variables

ScaleSocket can optionally expose CGI [environment variables](https://www.rfc-editor.org/rfc/rfc3875.html) to the target.


The supported environment variables are:
* `QUERY_STRING` eg. `foo=bar&baz=qux`
* `REMOTE_ADDR` eg. `127.0.0.1:1234`
* `ROOM` eg. `exampleroom`
* `PORT` for binding in TCP mode
* Any environment variables specified with `--passenv`

See the [CLI Reference](/man/cli.md) and the `--passenv` argument for details.

## Endpoints

### Metrics Endpoint

ScaleSocket can expose an [OpenMetrics](https://openmetrics.io/) and [Prometheus](https://prometheus.io/) compatible endpoint for scraping metrics.

The tracked metrics are:
* `scalesocket_websocket_connections` with the label `room`
* `scalesocket_websocket_connections_total` with the label `room`

See the [CLI Reference](/man/cli.md) and the `--metrics` flag for details.

### Metadata Endpoint

ScaleSocket can expose a JSON endpoint for retrieving rooms and their metadata.

This is useful for building a lobby or room list.

See the [CLI Reference](/man/cli.md) and the `--api` flag for details.

### Health Endpoint

ScaleSocket exposes a standard `/health` endpoint for checking readiness.

## Static File Hosting

ScaleSocket can serve static files from a directory.

See the [CLI Reference](/man/cli.md) and the `--staticdir` argument for details.