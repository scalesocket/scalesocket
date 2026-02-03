import { createInterface } from 'readline';


const state = {
  'clip': { x: 125, y: 125 },
  'note': { x: 725, y: 200 },
  'globe': { x: 625, y: 700 },
  'floppies': { x: 225, y: 520 },
}

const send = (data: Object) => {
  console.log(JSON.stringify(data))
}

const onReceive = (e: string) => {
  const { t: type, ...data } = JSON.parse(e)
  switch (type) {
    case 'Join':
      console.error('Someone joined');
      send({ t: 'State', state })
      return
    case 'Move':
      if (data.key in state) {
        state[data.key] = data.position
        send({ t: 'State', state })
      }
      return
  }
}

// Messages are received by reading lines from stdin
createInterface({ input: process.stdin })
  .on('line', (line: string) => onReceive(line.trim()));
