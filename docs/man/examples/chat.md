---
title: Websocket chat example using ScaleSocket
---

# Chat Example in Javascript

The most trivial example. A chat based on wrapping [cat(1)](https://linux.die.net/man/1/cat) without any backend code.

➡ [View full source on GitHub](https://github.com/scalesocket/scalesocket/tree/main/examples/chat)  
➡ [Run the example](#running-the-example)

## Running the example

You can run this self-contained example using Docker.

```shell
git clone https://github.com/scalesocket/scalesocket
cd scalesocket/examples/
docker compose up --build chat
```

Then open `http://localhost:5000/` in your browser.

## Frontend code

The frontend is a single html file, `index.html`, with some javascript. It connects with websockets to the backend and shows a chat interface.

```html
{% include "_partials/examples/chat/index.html" %}
```

## Backend code

There is no backend code. The backend is a wrapped `cat` process.

## Backend server

The backend is the ScaleSocket server.

We want to:
* let participants join rooms based on URL
* start a new `cat` process when a new user connects
* host a static html file

To do this, start ScaleSocket using:

```shell
scalesocket --addr 0.0.0.0:5000\
    --staticdir /var/www/public/\
    --frame=json\
    cat
```

## How does it work?

The frontend connects to the backend using websockets. The backend spins up a new `cat` process.

When the frontend sends a chat message, ScaleSocket passes it directly to the *stdin* of `cat`.

Since `cat` echoes all input it receives, the reply to *stdout* is the message itself, which ScaleSocket sends back to all connected clients.
