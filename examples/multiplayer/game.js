export class Game {
    constructor(app, ws, resources) {
        this.app = app;
        this.ws = ws;
        this.resources = resources;
        this.state = new GameState(this.app.stage, resources);
    }

    sendMessage(t, data) {
        // send outgoing message
        this.ws.send(JSON.stringify({ t, data }));
    }

    recvMessage(t, data) {
        // handle incoming message
        if (t == "State") {
            this.state.updatePlayers(data.players);
        } else if (t == "Leave") {
            this.state.removePlayer(data.leaver);
        }
    }

    async init() {
        // set websocket listener
        this.ws.addEventListener('message', e => {
            const { t, data } = JSON.parse(e.data);
            this.recvMessage(t, data);
        });

        // set click listener
        this.app.stage.interactive = true;
        this.app.stage.hitArea = this.app.screen;
        this.app.stage.on('pointerdown', (e) => {
            this.sendMessage('Input', { x: e.global.x, y: e.global.y });
        });

        // set interpolation to run on every tick
        PIXI.Ticker.shared.add(dt => this.state.interpolate(dt));
    }
}

class GameState {
    /** @type {Record<number, {sprite: any, pos: any}>} */
    players = {}

    constructor(stage, resources) {
        this.stage = stage;
        this.resources = resources;
    }

    updatePlayers(players) {
        for (const [id, pos] of Object.entries(players)) {
            this.updatePlayer(id, pos);
        }
    }

    updatePlayer(id, pos) {
        if (id in this.players) {
            this.players[id].pos = pos;
        } else {
            this.addPlayer(id, pos);
        }
    }

    addPlayer(id, pos) {
        const [x, y] = pos;
        const sprite = new PIXI.Sprite(new PIXI.Texture(this.resources.bunny.baseTexture));
        sprite.anchor.set(0.5);
        sprite.position.set(x, y);

        this.players[id] = { pos, sprite };
        this.stage.addChild(sprite);
    }


    removePlayer(id) {
        console.log(id)
        if (id in this.players) {
            console.log('removing player');
            const sprite = this.players[id].sprite;
            this.stage.removeChild(sprite);
            sprite.destroy();
            delete this.players[id];
        }
    }

    interpolate(dt) {
        for (const { pos, sprite } of Object.values(this.players)) {
            const [x, y] = pos;
            const dx = (x - sprite.x) / 3;
            const dy = (y - sprite.y) / 3;
            sprite.x += dx;
            sprite.y += dy;
        }
    }
}
