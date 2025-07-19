const ws = new WebSocket("ws://localhost:5273");

/*
pub enum Method {
    Install(App),
    SetEnabled(String, bool),
    Edit(String, App),
    Uninstall(String),
    Toggle(String, bool),
    Start(String),
    Stop(String),
    Restart(String),
}
*/

ws.onmessage = (msg) => console.log("Received:", msg.data);
ws.onopen = () => {
  ws.send("my-key");

  // ws.send(
  //   JSON.stringify({
  //     Install: {
  //       repo: "git@github.com:osui-rs/osui.git",
  //       branch: "master",
  //       install_command: "",
  //       run_command: "cargo run",
  //     }
  //   })
  // );

  // ws.send(
  //   JSON.stringify({
  //     SetEnabled: ["osui-rs/osui", true]
  //   })
  // );

  ws.send(
    JSON.stringify({
      Start: 0
    })
  );

  // ws.send(
  //   JSON.stringify({
  //     Uninstall: "git@github.com:osui-rs/osui.git"
  //   })
  // );
};
