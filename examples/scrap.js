const sck = new WebSocket("ws://127.0.0.1:39093/ws")

sck.onmessage = (data) => {
  const msg = JSON.parse(data)

  if (msg.kind === "NextWorkAnswer") {
    sck.send(JSON.stringify({
      "kind": "ScrapResult",
      "url": msg.url,
      "keywords": {
        "nutella": 3,
        "spiderman": 2,
      },
      "links": ["https://www.youtube.com/watch?v=tFxmGDGAJr4"],
    }))
  }
}

sck.send(JSON.stringify({
  "kind": "NextWork"
}))
