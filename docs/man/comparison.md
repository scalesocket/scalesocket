---
hide_side_table_of_contents: true
---
# Alternatives

A comparison of tools offering similar functionality.
The table highlights which tools support bidirectional communication with a shared backend instance.

|                                                Application|Language|Spawn Technology|Channels|Production ready|Bidirectional connection|Shared backend|Usecase|
|:----|:----|:----|:----|:----|:----|:----|:----|
|[ScaleSocket](https:/scalesocket.org/)|rust|processes|stdio, tcp|✓|✓|✓|bidirectional websocket server|
|[spawner](https://github.com/drifting-in-space/spawner)|rust|docker, kubernetes|http, ws| |✓|✓|bidirectional websocket server|
|[websocketd](http://websocketd.com/)|go|processes|stdio|✓|✓| |bidirectional websocket server|
|[websocat](https://github.com/vi/websocat)|rust|processes|stdio, tcp|✓|✓| |all-round tool|
|[gwsocket](https://gwsocket.io/)|c| |stdio, pipes|✓|✓|✓|bidirectional websocket stream|
|[websockify](https://github.com/novnc/websockify)|various|processes|tcp|✓|✓|✓|websocket to TCP proxy|
|[agones](https://agones.dev/site/docs/overview/)|go|kubernetes|tcp, udp|✓|✓|✓|gameserver scaler|
|                                                FastCGI|c|processes|stdio|✓| |✓|dynamic websites in 2000's|
|                                                inetd|c|processes|stdio|✓| | |dynamic websites in 1990's|

Also some servers, such as NGINX and Caddy support spawning processes and CGI with plugins.