import * as PIXI from "pixi.js";

 /** @typedef {(PIXI.Sprite & { targetPosition?: PIXI.Point })} Item */

export class Whiteboard {
  /**
    * @param {WebSocket} ws
    * @param {PIXI.Container} viewport
    */
  constructor(ws, viewport) {
    this.viewport = viewport;
    this.ws = ws;

    /** @type {Record<string, Item>} */
    this.items = {};
    this.selectedKey = null;

    // Canvas event handlers
    this.viewport.on('pointermove', (e) => this.onPointerMove(e));
    this.viewport.on('pointerup', (e) => this.onPointerUp(e));

    // WebSocket handlers
    this.ws.onmessage = (e) => this.onMessage(e);
    this.sendMessageThrottled = throttle(msg => this.sendMessage(msg), 300);

    // Animation loop
    PIXI.Ticker.shared.add(() => this.interpolate());
  }

  /**
    * Interpolates sprites towards their target position.
    */
  interpolate() {
    const items = Object.values(this.items)
      .filter(item => item.targetPosition)
      .filter(item => item.label !== this.selectedKey)
      .map(({ position, targetPosition }) => ({
        position, x: position.x, y: position.y, nx: targetPosition.x, ny: targetPosition.y
      }));

    for (const { position, x, y, nx, ny } of items) {
      position.set(x + (nx - x) * 0.01, y + (ny - y) * 0.01);
    }
  }

  sendMessage(/**@type {Object}*/ data) {
    this.ws.send(JSON.stringify(data));
  }

  onMessage(/**@type {MessageEvent}*/ e) {
    const { t: type, ...payload } = JSON.parse(e.data);
    switch (type) {
      case 'State':
        void this.updateState(payload.state);
        return
      default:
        console.error(`Unknown message type ${type}`);
        return
    }
  }

  async updateState(/**@type {Record<string, PIXI.Point>}*/ newState) {
    // Add new items and update existing ones
    for (const [key, position] of Object.entries(newState)) {
      if (!(key in this.items)) {
        this.items[key] = await this.addItem(key, position);
      } else if (this.selectedKey !== key) {
        this.items[key].targetPosition = position;
      }
    }
    // Remove deleted items
    for (const key of Object.keys(this.items)) {
      if (!(key in newState)) {
        this.viewport.removeChild(this.items[key]);
        this.items[key].destroy();
        delete this.items[key];
      }
    }
  }

  onPointerMove(/**@type {PIXI.FederatedMouseEvent}*/ e) {
    const key = this.selectedKey
    if (key && key in this.items) {
      // @ts-ignore
      const position = this.viewport.toWorld(e.global)

      this.items[key].position.set(position.x, position.y);
      this.items[key].targetPosition = undefined;
      this.sendMessageThrottled({ t: 'Move', key, position })
    }
  }

  onPointerUp(/**@type {PIXI.FederatedMouseEvent}*/ e) {
    const key = this.selectedKey;
    if (key) {
      const { x, y } = this.items[key];
      this.sendMessage({ t: 'Move', key, position: { x, y } })
      this.selectedKey = null;
      this.items[key].cursor = 'grab';
      this.viewport.drag({ pressDrag: true });
    }
  }

  /**
   * @param {string} key
   * @param {PIXI.Point} position
   */
  async addItem(key, position) {
    const texture = await PIXI.Assets.load(`${key}.png`);
    /** @type {Item} */
    const item = new PIXI.Sprite(texture);

    item.anchor.set(0.5);
    item.eventMode = 'static';
    item.cursor = 'grab';
    item.label = key;
    item.position.set(position.x, position.y);
    item.targetPosition = undefined;

    item.on('pointerdown', () => {
      this.selectedKey = key;
      item.cursor = 'grabbing';
      this.viewport.drag({ pressDrag: false})
    })

    this.viewport.addChild(item);
    return item
  }
}

/**
 * @param {Function} func
 * @param {number} limit
 */
function throttle(func, limit) {
  let waiting = false;
  return (...args) => {
    if (!waiting) {
      func(...args);
      waiting = true;
      setTimeout(() => waiting = false, limit);
    }
  };
}
