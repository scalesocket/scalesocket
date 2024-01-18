# Advanced Usage

## Environment Variables

ScaleSocket can optionally expose CGI [environment variables](https://www.rfc-editor.org/rfc/rfc3875.html) to the target.


The supported environment variables are:
* `QUERY_STRING` eg. `foo=bar&baz=qux`
* `REMOTE_ADDR` eg. `127.0.0.1:1234`
* `ROOM` eg. `exampleroom`
* `PORT` for binding in TCP mode
* Any environment variables specified with `--passenv`

See the [CLI Reference](/man/cli.md) and the `--passenv` argument for details.

## HTTP Endpoints

### Metrics Endpoint

ScaleSocket can expose an [OpenMetrics](https://openmetrics.io/) and [Prometheus](https://prometheus.io/) compatible endpoint for scraping metrics.

The tracked metrics are:
* `scalesocket_websocket_connections` with the label `room`
* `scalesocket_websocket_connections_total` with the label `room`

See the [CLI Reference](/man/cli.md) and the `--metrics` flag for details.

### Metadata Endpoint

ScaleSocket can expose a JSON endpoint for retrieving rooms and their metadata.

When `--framing` is set to JSON, additional metadata can be set by the server using messages with a `_meta: true` field.
This is useful for building a lobby or room list.

See the [CLI Reference](/man/cli.md) and the `--api` flag for details.

### Health Endpoint

ScaleSocket exposes a standard `/health` endpoint for checking readiness.

## Delimiters

ScaleSocket parses output intpo messages by splitting output at newline (`\n`), except when `--binary` mode is enabled.

To alter this behaviour, the `--delimiters` argument can be used to specify a set of custom delimiters.
For example `--delimiters=,` will split comma-separated output by the target into messages.

To set the delimiter to the null-byte (`\0`), use `--null`. This is useful for outputting multiline messages from the target, for example as a backend for [htmx](https://htmx.org/).

## Caching

ScaleSocket can optionally cache server sent messages, and send them to new clients when they join a room.

Two types of caching are supported: `all` and `tagged`. Supported cache sizes are `1`, `8` and `64`.

This is an experimental feature.

### Caching All Messages

When `--cache=all:8` is enabled, the last 8 messages sent by the server will be cached.

This is useful for enabling a chat history.

### Caching Tagged Messages

When `--framing` is set to JSON, and `--cache=tagged:8` is enabled, only messages with a `_cache: true` field will be cached.

This is useful for maintaining a state that is sent quickly to clients when they join a room.

See the [CLI Reference](/man/cli.md) and the `--cache` and `--cachepersist` arguments for details.
