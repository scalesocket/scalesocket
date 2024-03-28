export class Socket extends WebSocket {
    constructor(uri) {
        super(uri);
        this.addEventListener('message', (e) => {
            const message = this.parseJSON(e.data);
            if (message !== null) {
                const event = new CustomEvent(`message.${message.type}`, { detail: message });
                this.dispatchEvent(event);
            }
        });
    }

    sendJSON(type, data) {
        this.send(JSON.stringify({ t: type, ...data }));
    }

    requestJSON(type, data) {
        const promise = new Promise((resolve, reject) => {
            this.addEventListener(`message.${type}Response`, (message) => resolve(message.detail), true);
        });
        this.sendJSON(type, data);
        return promise;
    }

    parseJSON(data) {
        if (data === undefined || !data.startsWith("{")) {
            return null;
        }
        return JSON.parse(data.trim());
    }
}
