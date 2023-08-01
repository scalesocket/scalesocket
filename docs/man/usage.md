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
Alternatively, the messages can be sent to the target using TCP.

See the [CLI Reference](/man/cli.md) and the `--tcp`, `--tcpports` and `--cmd-attach-delay` arguments for details.

## Rooms

Clients connecting to the server specify a room in the connection URL.

Connecting to a room spawns a new process of the wrapped binary or script. Subsequent connections to the same room share the same process.

The room ID is the first path component of the URL. For example `wss://example.com/exampleroom`.

## Framing and Routing Messages

ScaleSocket can optionally parse, tag and route messages.

### JSON Framing

When `--frame=json` is enabled, messages will be parsed and routed, with the following rules:
* Messages from the client, that are not valid JSON are dropped
* Messages from the client are tagged with an `id` field
* Messages from the server, that contain an `id` field will be routed to the specific client

### Binary Framing

When `--frame=binary` is enabled, ScaleSocket is compatible with [gwsocket](https://gwsocket.io/). Messages will be parsed according to the [gwsocket strict mode](https://gwsocket.io/man#man-strict-mode), with the following rules:
* Messages from the client, with an invalid header are dropped
* Messages from the client must set the header type to `0x01` (text)
* Messages from the server, that contain an `id` field will be routed to the specific client

See the [CLI Reference](/man/cli.md) and the `--frame`, `--server-frame` and `--client-frame` arguments for details.

## Join and Leave Messages

ScaleSocket can optionally send a message to the target when a client joins or leaves a room. An `#ID` placeholder will be replaced with the client's ID.

```console
$ scalesocket --frame=json --joinmsg '{"t":"Join","id":#ID}' --leavemsg '{"t":"Leave"}' ./example.sh
```

See the [CLI Reference](/man/cli.md) and the `--joinmsg` and `--leavemsg` arguments for details.

## Metrics Endpoint

ScaleSocket can expose an [OpenMetrics](https://openmetrics.io/) and [Prometheus](https://prometheus.io/) compatible endpoint for scraping metrics.

The tracked metrics are:
* `connections` with the label `room`
* `unique_connections` with the label `room` (TODO)

See the [CLI Reference](/man/cli.md) and the `--metrics` flag for details.

## Stats Endpoint

ScaleSocket can expose a JSON endpoint for retrieving stats for rooms.

See the [CLI Reference](/man/cli.md) and the `--stats` flag for details.

## Static File Hosting

ScaleSocket can serve static files from a directory.

See the [CLI Reference](/man/cli.md) and the `--staticdir` argument for details.