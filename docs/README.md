---
hide_side_table_of_contents: true
hide_navigation: true
---

<div class="hero">

<div class="banner">

# Websocket server and autoscaler

Simple way to build multiplayer backends.

</div>

## What is ScaleSocket?

ScaleSocket lets you to wrap a script or binary, and serve it over websockets. Clients then connect to *rooms* which have an unique URL (`wss://example.com/exampleroom`). Connecting to a room spawns a new process of the wrapped binary. Subsequent connections to the same room share the process.

<div class="links">

[Quick Start](/man/README.md#quick-start)
[Installation](/man/installation.md)
[Online Demo](https://demo-multiplayer.scalesocket.org/)

</div>

## ScaleSocket vs. Websocketd

ScaleSocket is inteded to be a drop-in replacement for [websocketd](http://websocketd.com/). It supports many of the same features, but allows websocket connections to share a single backend process.

For a full comparison of alternatives, see [comparison](/man/comparison.md).

## What can it be used for?

ScaleSocket is useful for building and prototyping multiplayer backends. It can be used for chat rooms, multiplayer games and real-time collaboration applications.

## What does it look like?

Below is an example websocket echo server in three lines of Python. No netcode reqiured, just stdin and stdout.

{% capture code %}
```python
from sys import stdin

for message in stdin:
    print(f"hello {message}")
```
{% endcapture %}

{% include "_partials/window.md" content: code, title: "example.py", class: "" %}

{% capture shell %}
```console
$ scalesocket --addr 0.0.0.0:5000 ./example.py
[INFO] running at 0.0.0.0 on port 5000
```
{% endcapture %}

{% include "_partials/window.md" content: shell, title: "", class: "terminal" %}

A websocket server is now running.
Clients can connect to rooms at `ws://localhost:5000/<room id>`.
In this example, messages sent by the server will be forwarded to all participants in the room.

This saves us from implementing lobby and room management logic. In fact, it also saves us from implementing any netcode at all for the backend.

For more advanced usage and features, see [usage](/man/usage.md).
