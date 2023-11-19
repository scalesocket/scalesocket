---
title: Multipayer game example using ScaleSocket
---

# Multiplayer game Example in Javascript & Python

A simple example. An game that has a backend in Python and frontend in Javascript, using [pixi.js](https://pixijs.com/).

➡ [View demo online](https://demo-multiplayer.scalesocket.org)  
➡ [View full source on GitHub](https://github.com/scalesocket/scalesocket/tree/main/examples/multiplayer)  
➡ [Run the example](#running-the-example)

## Running the example

You can run this self-contained example using Docker.

```shell
git clone https://github.com/scalesocket/scalesocket
cd scalesocket/examples/
docker compose up --build multiplayer
```

Then open `http://localhost:5000/` in your browser.


## Frontend code

The frontend consists of three files `index.html`, `client.js` and `bunny.png`. [Pixi.js](https://www.pixijs.com/) is used to simplify drawing and managing sprites.

The `index.html` file loads the game and connects to the server using websockets.

```html
{% include "_partials/examples/multiplayer/index.html" %}
```

The actual frontend logic is in `client.js`. We receive message from the websocket and update sprites (players) based on it. When the player clicks on the screen, we send the input to the server.

```js
// client.js
{% include "_partials/examples/multiplayer/client.js" %}
```

## Backend code

The backend is a single file, `server.py`. It reads stdin in a loop, and updates state based on incoming messages.

```python
{% include "_partials/examples/multiplayer/server.py" %}
```

## Backend server

The backend is the ScaleSocket server.

We want to:
* let players join rooms based on URL
* start a new `server.py` process when a new user connects
* host the static files

To do this, start ScaleSocket using:

```shell
scalesocket --addr 0.0.0.0:5000\
    --staticdir /var/www/public/ \
    --json\
    server.py
```

## How does it work?

The frontend connects to the server using websockets. The server spins up a new `server.py` process.

When the frontend sends a input, ScaleSocket passes it directly to the *stdin* of `server.py`.

The input is read in a loop by mapping json parser over the `stdin` which is a generator in Python.

```py
# excerpt from server.py
from sys import stdin

def parse_json(data: str):
    with suppress(JSONDecodeError):
        return loads(data)
    return None

stdin_events = map(parse_json, stdin)

for event in stdin_events:
    # do something with the event
    # in our case the frontend sends
    # {"t": "Input", "data": {"x": 123, "y": 456}}
    # ...
```

The events update the local state on the backend, and the backend sends the new state to *stdout*, by printing the state.

```py
# excerpt from server.py
from json import dumps

# ...

# sending is as easy as printing
print(dumps({"t": t, "data": data}))

# optionally, we may specify a receiver by using "_to"
print(dumps({"t": t, "data": data, "_to": to_id}))
```