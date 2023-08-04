#!/usr/bin/python3
from contextlib import suppress
from json import JSONDecodeError, loads, dumps
from sys import stdin, stdout, stderr


def main():
    print("game server started", file=stderr)
    players = {}

    # receiving data is as easy as reading stdin
    stdin_events = map(parse_json, stdin)

    for event in stdin_events:
        t, id, data = parse_event(event)

        if t == "Join":
            players[id] = (150, 150)
            send_event("State", {"players": players}, to_id=id)
        elif t == "Leave":
            del players[id]
            send_event("Leave", {"leaver": id})
        elif t == "Input":
            players[id] = (data.get("x", 0), data.get("y", 0))
            send_event("State", {"players": players})


def send_event(t: str, data: dict, to_id: int = None):
    # sending data is as easy as printing
    print(dumps({"t": t, "data": data, "id": to_id}))


def parse_json(data: str):
    with suppress(JSONDecodeError):
        return loads(data)
    return None


def parse_event(event: dict):
    with suppress(KeyError):
        return event["t"], int(event["id"]), event.get("data")
    return None, None, None


if __name__ == "__main__":
    # ensure python output is not buffered
    stdout.reconfigure(line_buffering=True)
    main()
