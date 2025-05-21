
const api = "http://127.0.0.1:39093/api/add-url"
const urls = [
  "https://www.youtube.com/watch?v=gOymiMEKWqQ",
  "https://www.youtube.com/watch?v=pTsKD-m5tgI",
  "https://docs.rs/ratatui/latest/ratatui/index.html",
  "https://www.npmjs.com/package/puppeteer",
  "https://www.youtube.com/watch?v=T61ZfLMJUMk&t=2412s",
  "https://www.youtube.com/watch?v=9gQvDmoU6yk",
  "https://wildtruth.net/",
  "https://www.youtube.com/watch?v=nXCRytg5RaY",
  "https://www.youtube.com/watch?v=pB8vKPKm4D8",
]

for(let url of urls) {
  const resp = await fetch(api, {
    method: "post",
    headers: {"content-type": "application/json"},
    body: JSON.stringify({ url })
  })

  if(!resp.ok) {
    console.log(resp.statusText)
    console.log(await resp.text())
    break
  }
}
