function go() {
  const button = document.getElementById('action');
  button.onmousedown = async () => {
	 const response = await fetch('/api/action', {
		method: 'POST',
		headers: {
        "Content-Type": "application/json",
		},
		body: JSON.stringify({message: 123}),
	 });
	 const json = await response.text();
	 console.log(json);
  }
}

go();
