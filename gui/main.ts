function go() {

  const ws = new WebSocket('/ws/')
  ws.onopen = () => {
    console.log('ws opened on browser')
  }

  ws.onmessage = message => {
    console.log(`message received`, message.data);
  }

  const action = document.getElementById('action')!;
  action.onmousedown = async () => {
    ws.send(JSON.stringify({ message: "Drum" }));
  }

  const quit = document.getElementById('quit')!;
  quit.onmousedown = async () => {
    ws.send(JSON.stringify({ message: "Quit" }));
  }

}

go();
