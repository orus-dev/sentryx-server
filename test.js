const ws = new WebSocket("ws://localhost:5273");

ws.onmessage = (msg) => console.log("Received:", msg.data);
ws.onopen = () => {
  ws.send("my-key");
  ws.send(
    JSON.stringify({
      repo: "git@github.com:osui-rs/osui.git",
      branch: "master",
      install_command: "",
      run_command: "cargo run",
    })
  );
};
