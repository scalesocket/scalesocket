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

See the [CLI Reference](/man/cli.md) and the `--tcp`, `--tcpports` and `--delay` arguments for details.

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
* Messages from the server, that contain a `_to` field will be routed to the specific client

In addition
* Messages from the server, that contain a `_meta` field set to `true` will be stored in the room metadata, and dropped

### Binary Framing

When `--frame=binary` is enabled, ScaleSocket is compatible with [gwsocket](https://gwsocket.io/). Messages will be parsed according to the [gwsocket strict mode](https://gwsocket.io/man#man-strict-mode), with the following rules:
* Messages from the client, with an invalid header are dropped
* Messages from the client must set the header type to `0x01` (text)
* Messages from the server, that contain a nonzero client `id` field will be routed to the specific client

See the [CLI Reference](/man/cli.md) and the `--frame`, `--serverframe` and `--clientframe` arguments for details.

## Join and Leave Messages

ScaleSocket can optionally send a message to the target when a client joins or leaves a room.

The messages support the variables:
* `#ID` eg. `123`
* `QUERY_XYZ` for each query parameter, `?xyz=`, in the connection URL.
* The [Environment variables](/man/advanced_usage.md#environment-variables)

For example, starting scalesocket with:

```console
$ scalesocket --joinmsg '{"type":"Join","_from":#ID}' ./example.sh
```

Sends the message `{"type":"Join","_from":123}` to the server when a new client joins. This is useful for keeping track of connected clients.


See the [CLI Reference](/man/cli.md) and the `--joinmsg` and `--leavemsg` arguments for details.

## Static File Hosting

ScaleSocket can serve static files from a directory.

See the [CLI Reference](/man/cli.md) and the `--staticdir` argument for details.