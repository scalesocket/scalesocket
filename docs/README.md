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

ScaleSocket is a command line tool that lets you to wrap a script or binary, and serve it over websockets. Clients then connect to *rooms* (a.k.a. channels) which have a unique URL (`wss://example.com/exampleroom`). Connecting to a room spawns a new process of the wrapped binary. Subsequent connections to the same room share the process.

<div class="links">

[Quick Start](/man/README.md#quick-start)
[Installation](/man/installation.md)
[Online Demo](https://demo-multiplayer.scalesocket.org/)

</div>

## What can it be used for?

ScaleSocket is useful for building and prototyping multiplayer backends. It can be used for chat rooms, multiplayer games and real-time collaboration applications.

## What does it look like?

Below is an example websocket echo server in three lines of Python. No netcode required, just stdin and stdout.

{% capture code %}
```python
#!/usr/bin/python -u
from sys import stdin, stdout

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

A websocket server is running! Let's connect to it and send something to see that it works.

{% capture shell %}
```console
$ websocat --no-exit-on-zeromsg ws://127.0.0.1:5000/exampleroom
▶ world
◀ hello world
```
{% endcapture %}

{% include "_partials/window.md" content: shell, title: "", class: "terminal" %}

Note that multiple clients can share the room. We can also create a new room by connecting to a different route. This saves us from implementing lobby and room management logic. In fact, it also saves us from implementing any netcode at all for the backend.

For more advanced usage and features, see [usage](/man/usage).
