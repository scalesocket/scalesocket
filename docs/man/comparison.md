---
hide_side_table_of_contents: true
---
# Alternatives

Depending on your needs, these alternatives may suit your needs.

|                                                Application|Language|Spawn Technology|Channels|Production ready|Two-way connection|Shared backend|Usecase|
|:----|:----|:----|:----|:----|:----|:----|:----|
|[ScaleSocket](https:/scalesocket.org/)|rust|processes|stdio, tcp|✓|✓|✓|two-way websocket server|
|[spawner](https://github.com/drifting-in-space/spawner)|rust|docker, kubernetes|http, ws| |✓|✓|two-way websocket server|
|[websocketd](http://websocketd.com/)|go|processes|stdio|✓|✓| |two-way websocket server|
|[websocat](https://github.com/vi/websocat)|rust|processes|stdio, tcp|✓|✓| |all-round tool|
|[gwsocket](https://gwsocket.io/)|c| |stdio, pipes|✓|✓|✓|two-way websocket stream|
|[websockify](https://github.com/novnc/websockify)|various|processes|tcp|✓|✓|✓|websocket to TCP proxy|
|[agones](https://agones.dev/site/docs/overview/)|go|kubernetes|tcp, udp|✓|✓|✓|gameserver scaler|
|                                                FastCGI|c|processes|stdio|✓| |✓|dynamic websites in 2000's|
|                                                inetd|c|processes|stdio|✓| | |dynamic websites in 1990's|

Also some servers, such as NGINX and Caddy support spawning processes and CGI with plugins.