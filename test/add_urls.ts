
const sck = new WebSocket("ws://127.0.0.1:39093/ws")
const urls = [
  "https://www.youtube.com/watch?v=gOymiMEKWqQ",
  "https://www.youtube.com/watch?v=pTsKD-m5tgI",
  "https://docs.rs/ratatui/latest/ratatui/index.html",
  "https://www.npmjs.com/package/puppeteer",
  "https://www.youtube.com/watch?v=T61ZfLMJUMk&t=2412s",
]

sck.onopen = () => {
  for (let url of urls) {
    sck.send(JSON.stringify({
      "kind": "AddUrl",
      "url": url,
    }))
  }

  sck.close()
}
