// Based on https://codemirror.net/examples/collab/

import { ChangeSet, Text } from '@codemirror/state'
import { Update, rebaseUpdates } from '@codemirror/collab'

type Data = {
    t: 'Join' | 'pullUpdates' | 'pushUpdates' | 'getDocument'
    _from: string
    version: number
    updates: Update[]
    doc?: string
}

let updates: Update[] = []
let doc = Text.of(['Try typing here, and open another browser window!\n\n\n\n\n\n'])
let pending: ((value: any) => void)[] = []
import { createInterface } from 'node:readline'
const stdin = createInterface({ input: process.stdin, terminal: false })

stdin.on('line', (line: string) => {
    if (!line.startsWith('{')) {
        return
    }

    let data: Data = JSON.parse(line.trim().trim())
    if (data.t == 'pullUpdates') {
        if (data.version < updates.length) {
            resp({
                type: 'pullUpdatesResponse',
                updates: updates.slice(data.version),
                _to: data._from,
            })
        } else {
            pending.push((p) => resp({ ...p, _to: data._from }))
        }
    } else if (data.t == 'pushUpdates') {
        // Convert the JSON representation to an actual ChangeSet instance
        let received = data.updates.map((json) => ({
            clientID: json.clientID,
            changes: ChangeSet.fromJSON(json.changes),
        }))

        if (data.version != updates.length)
            received = rebaseUpdates(
                received,
                updates.slice(data.version)
            ) as Update[]

        for (let update of received) {
            updates.push(update)
            // console.error(updates);
            doc = update.changes.apply(doc)
        }

        resp({ type: 'pushUpdatesResponse', status: true, _to: data._from })

        if (received.length) {
            // Notify pending requests
            let json = received.map((update) => ({
                clientID: update.clientID,
                changes: update.changes.toJSON(),
            }))
            while (pending.length)
                pending.pop()!({ type: 'pullUpdatesResponse', updates: json })
        }
    } else if (data.t == 'Join' || data.t == 'getDocument') {
        resp({
            type: 'getDocumentResponse',
            version: updates.length,
            doc: doc.toString(),
            _to: data._from,
        })
    } else {
        console.error('Unknown message')
        console.error(data)
    }
})

function resp(value: any) {
    console.log(JSON.stringify(value))
}
