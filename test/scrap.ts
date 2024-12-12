import { bytesToHex } from "@noble/curves/abstract/utils"
import { secp256k1 } from "@noble/curves/secp256k1"

const prikey = secp256k1.utils.randomPrivateKey()
const pubkey = secp256k1.getPublicKey(prikey)
const pubkeyHex = bytesToHex(pubkey)
const ws = new WebSocket("ws://127.0.0.1:39093/ws")

ws.onopen = () => {
  ws.send(JSON.stringify({
    "kind": "NextWork",
    "pubkey": pubkeyHex,
  }))
}

ws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data)
  
  console.log(msg)
  if (msg.kind === "NextWorkAnswer") {
    // sck.send(JSON.stringify({
    //   "kind": "ScrapResult",
    //   "url": msg.url,
    //   "keywords": {
    //     "nutella": 3,
    //     "spiderman": 2,
    //   },
    //   "links": ["https://www.youtube.com/watch?v=tFxmGDGAJr4"],
    // }))
  }

  ws.close()
}
