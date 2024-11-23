---
hide_side_table_of_contents: true
---

# Alternatives and Comparison

## ScaleSocket vs. Websocketd

ScaleSocket is inteded to be a drop-in replacement for [websocketd](http://websocketd.com/).
It supports many of the same features, but allows websocket connections to share a single backend process.

ScaleSocket fully embraces the concept of rooms, and allows routing messages to specific clients.

Additionally ScaleSocket exposes APIs for fetching room metadata, and metrics, which are useful for building lobbies and monitoring.

## ScaleSocket vs Socket.io

ScaleSocket can be used as a replacement for [Socket.io](https://socket.io/) in some cases.
It supports some of the same features, such as rooms and message routing.

The difference lies in how they are integrated with the backend code.
Socket.io is a (frontend and) backend library that is used by the backend code.
It requires the backend to handle connections and room logic.


ScaleSocket is a command line tool, that is used to "wrap"the backend without touching the code.
It is intended to somplify the backend so that it does not need to handle connections or room logic.

## Other Alternatives

A comparison of tools offering similar functionality.
The table highlights which tools support bidirectional communication with a shared backend instance.

|                                                Application|Language|Spawn Technology|Channels|Bidirectional connection|Shared backend|Use case|
|:----|:----|:----|:----|:----|:----|:----|
|[ScaleSocket](https:/scalesocket.org/)|rust|processes|stdio, tcp|✓|✓|bidirectional websocket server|
|[plane](https://github.com/drifting-in-space/plane)|rust|docker|http, ws|✓|✓|bidirectional websocket server|
|[websocketd](http://websocketd.com/)|go|processes|stdio|✓| |bidirectional websocket server|
|[websocat](https://github.com/vi/websocat)|rust|processes|stdio, tcp|✓| |all-round tool|
|[gwsocket](https://gwsocket.io/)|c| |stdio, pipes|✓|✓|bidirectional websocket stream|
|[wsbroad](https://github.com/vi/wsbroad/)|rust| | |✓|✓|websocket broadcaster|
|[websockify](https://github.com/novnc/websockify)|various|processes|tcp|✓|✓|websocket to TCP proxy|
|[agones](https://agones.dev/site/docs/overview/)|go|kubernetes|tcp, udp|✓|✓|gameserver scaler|
|                                                FastCGI|c|processes|stdio| |✓|dynamic websites in 2000's|
|                                                inetd|c|processes|stdio| | |dynamic websites in 1990's|

Also some servers, such as NGINX and Caddy support spawning processes and CGI with plugins.
