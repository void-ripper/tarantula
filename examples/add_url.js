
const host = "http://127.0.0.1:39093"
const resp = await fetch(host + "/api/add-url", {
  method: "post",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({
    "url": "https://www.youtube.com/watch?v=gOymiMEKWqQ"
  })
})

console.log(resp.statusText)
console.log(await resp.text())
