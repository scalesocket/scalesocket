---
title: Websocket chat example using ScaleSocket
---

# Terminal Streaming example in Javascript

Stream a TUI application over websockets using [xterm.js](https://xtermjs.org/).

➡ [View demo online](https://demo-terminal.scalesocket.org) (not mobile-friendly)  
➡ [View full source on GitHub](https://github.com/scalesocket/scalesocket/tree/main/examples/terminal)  
➡ [Run the example](#running-the-example)

## Running the example

You can run this self-contained example using Docker.

```shell
git clone https://github.com/scalesocket/scalesocket
cd scalesocket/examples/
docker compose up --build terminal
```

Then open `http://localhost:5000/` in your browser.

## Frontend code

The frontend is a single html file, `index.html`, using [xterm.js](https://xtermjs.org/). It connects with websockets to the server and shows a chat interface.

The javascript defines the terminal windget and connects it to a websocket:

<!-- i'm sorry, this is ugly -->
```js
{%- capture code -%}
{% include "_partials/examples/terminal/index.html" %}
{%- endcapture -%}
{% assign parts = code |  replace: "    ", "" |  replace: "/script>", "script>" | split: '<script>'  %}
{{ parts[3] | strip}}
```

## Backend code

There is no backend code, besides running a terminal application. The backend in this example is a wrapped [snake](https://pkgs.alpinelinux.org/packages?name=bsd-games&repo=testing) process.

## Backend server

The backend is the ScaleSocket server.

We want to:
* let participants join rooms based on URL
* start a new `snake` process when a new user connects
* host a static html file

To do this, start ScaleSocket using:

```shell
scalesocket --addr 0.0.0.0:5000\
    --binary\
    --staticdir /var/www/public/\
    # launch using unbuffer to expose a pty
    bash -- -c 'TERM=xterm-color unbuffer -p nsnake'
```

Note that we are using the `--binary` flag to enable binary mode. Furthermore, [unbuffer(1)](https://linux.die.net/man/1/unbuffer) and `TERM` let snake work as-if it was run in an interactive terminal.

## How does it work?

The frontend connects to the server using websockets. The backend spins up a new `snake` process.

When the frontend terminal widget sends a key-press, ScaleSocket passes it directly to the *stdin* of `snake`.

Since `snake` writes the screen to the terminal using *stdout*, ScaleSocket forwards it back to all connected clients.
