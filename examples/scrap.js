const host = "http://127.0.0.1:39093"
const resp = await fetch(host +  "/api/next-work", {method: "post"})
const msg = await resp.json()

fetch(host + "/api/scrap-result", {
  method: "post",
  headers: {"content-type": "application/json"},
  body: JSON.stringify({
    "url": msg.url,
    "keywords": {
      "nutella": 3,   // the keyword and how often it did occoure
      "spiderman": 2,
    },
    // the links found on the url
    "links": ["https://www.youtube.com/watch?v=tFxmGDGAJr4"],
  })
})
