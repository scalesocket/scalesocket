// Naive TypeScript port of CodeMirror collaborative example
// See https://codemirror.net/examples/collab/ for details

import { createInterface } from 'node:readline'
import { ChangeSet, Text } from '@codemirror/state'
import { Update, rebaseUpdates } from '@codemirror/collab'

type Data = {
    t: 'Join' | 'pullUpdates' | 'pushUpdates' | 'getDocument'
    version: number
    updates: Update[]
    doc?: string
    _from: string
}
type Callback = (value: any) => void

let updates: Update[] = []
let doc = Text.of(['Try typing, and open another browser window!\n\n\n\n\n\n'])
let pending: Callback[] = []
const stdin = createInterface({ input: process.stdin, terminal: false })

// ScaleSocket reads stdin from client websockets
stdin.on('line', (line: string) => {
    if (!line.startsWith('{')) {
        return
    }
    recv(JSON.parse(line.trim()))
})

function recv(data: Data) {
    if (data.t == 'pullUpdates') {
        // Request to pull updates from this server
        if (data.version < updates.length) {
            resp({
                type: 'pullUpdatesResponse',
                updates: updates.slice(data.version),
                // Use ScaleSocket routing to reply to the correct client
                _to: data._from,
            })
        } else {
            pending.push((p) => resp({ ...p, _to: data._from }))
        }
    } else if (data.t == 'pushUpdates') {
        // Reques to push updates to this server
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
            doc = update.changes.apply(doc)
        }

        resp({
            type: 'pushUpdatesResponse',
            status: true,
            // Use ScaleSocket routing to reply to the correct client
            _to: data._from,
        })

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
        // New client or request for current document
        resp({
            type: 'getDocumentResponse',
            version: updates.length,
            doc: doc.toString(),
            // Use ScaleSocket routing to reply to the correct client
            _to: data._from,
        })
    } else {
        console.error('Unknown message')
        console.error(data)
    }
}

function resp(value: any) {
    // ScaleSocket sends stdout back to client(s)
    console.log(JSON.stringify(value))
}
