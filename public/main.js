function go() {
  const action = document.getElementById('action');
  action.onmousedown = async () => {
	 const response = await fetch('/api/action', {
		method: 'POST',
		headers: {
        "Content-Type": "application/json",
		},
		body: JSON.stringify({message: "Drum"}),
	 });
	 const json = await response.text();
	 console.log(json);
  }

  const quit = document.getElementById('quit');
  quit.onmousedown = async () => {
	 const response = await fetch('/api/action', {
		method: 'POST',
		headers: {
        "Content-Type": "application/json",
		},
		body: JSON.stringify({message: "Quit"}),
	 });
	 const json = await response.text();
	 console.log(json);
  }

}

go();
