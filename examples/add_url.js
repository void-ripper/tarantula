
const sck = new WebSocket("ws://127.0.0.1:39093/ws")

sck.send(JSON.stringify({
  "kind": "AddUrl",
  "url": "https://www.youtube.com/watch?v=gOymiMEKWqQ"
}))
