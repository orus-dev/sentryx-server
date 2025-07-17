const ws = new WebSocket('ws://localhost:5273');

ws.onmessage = (msg) => console.log("Received:", msg.data);
ws.onopen = () => ws.send("my-key");