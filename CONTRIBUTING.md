# Contributing

Please use the following guidelines when contributing to `scalesocket`:

- Format branch names as: `TYPE-NAME`:
  - The types `feat,fix,docs,style,refactor,perf,tests,chore,examples` are used.
- Format PR title as: `TYPE: DESCRIPTION`:
  - The types `Feature,Fix,Docs,Style,Refactor,Perf,Tests,Chore,Examples` are used.
- Run the tests (`cargo test`)
- Run the linters and formatters (`cargo fmt`) and clippy (`cargo clippy`). Requires a nightly compiler.

## Architecture

The implementation relies heavily on async tasks and channels.
The main async tasks and channels (Tx/Rx) are outlined in the diagram below.

```
┌────────────────────────────────────────┐
│   ╔════╗     ╔════╗                    │▒
│   ║ WS ║     ║ WS ║       Internet     │▒
│   ╚═╦══╝     ╚═╦══╝                    │▒
╞═╤═══╩══════════╩═════╤═════════════════╡▒
│ │  routes::handle()  │◀╌╌╌╌╌╌╌┐        │▒
│ └────────────────────┘        ╎        │▒
│           │                   ╎        │▒
│        EventTx            Websocket    │▒
│           │                   ╎        │▒
│           ▼                   ╎        │▒
│ ┌────────────────────┐        ╎        │▒
│ │  events::handle()  │        ╎        │▒
│ ├─────────┬──────────┤        ╎        │▒
│ │ spawn() │ attach() │        ╎        │▒
│ └────┬────┴────┬─────┘        ╎        │▒
│      │         │              ╎        │▒
│      │         │              ▼        │▒
│      │    ┌────┴─────────────────┐     │▒
│      │    │ connection::handle() │     │▒
│      │    └──────────────────────┘     │▒
│      │         ▲                       │▒
│      │         │                       │▒
│      │   FromProcessRx                 │▒
│      │    ToProcessTx                  │▒
│      │         │                       │▒
│      │         ▼                       │▒
│ ┌────┴──────────────┐                  │▒
│ │                   │ ◀╌╌╌╌╌╌─┐        │▒
│ │ process::handle() │         ╎        │▒
│ │                   │ FromProcessRxAny │▒
│ ├───────────────────┤  ToProcessTxAny  │▒
│ │                   │         ╎        │▒
│ │      spawn()      │         ▼        │▒
╞═╧═════════╦═════════╧══════════════════╡▒
│      ╔════╩════╗                       │▒
│      ║ Process ║             OS        │▒
│      ╚═════════╝                       │▒
└────────────────────────────────────────┘▒
 ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
```
