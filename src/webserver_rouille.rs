// Copyright (c) 2016 The Rouille developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#[macro_use]
extern crate rouille;

use std::fs::File;
use std::thread;

use rouille::websocket;
use rouille::{Request, Response};

fn fallback(request: &Request) -> Response {
  let response = rouille::match_assets(request, "./public");
  if response.is_success() {
    return response;
  }
  Response::text("404 Not found").with_status_code(404)
}

fn main() {
  // This example demonstrates how to use websockets with rouille.

  // Small message so that people don't need to read the source code.
  // Note that like all examples we only listen on `localhost`, so you can't access this server
  // from another machine than your own.
  println!("Now listening on localhost:8000");

  rouille::start_server("localhost:8000", move |request| {
    router!(request,
        (GET) (/) => {
        let file = File::open("./public/index.html").unwrap();
        let response = Response::from_file("text/html", file);
        response
        },
        (GET) (/ws) => { start_ws(request) },
        _ => fallback(request)
    )
  });
}

fn start_ws(request: &Request) -> Response {
  let (response, websocket) = try_or_400!(websocket::start::<String>(&request, None));
  thread::spawn(move || {
    let ws = websocket.recv().unwrap();
    handle_ws(ws);
  });

  response
}

// Function run in a separate thread.
fn handle_ws(mut websocket: websocket::Websocket) {
  // We wait for a new message to come from the websocket.
  while let Some(message) = websocket.next() {
    match message {
      websocket::Message::Text(txt) => {
        // If the message is text, send it back with `send_text`.
        println!("received {:?} from a websocket", txt);
        websocket.send_text(&txt).unwrap();
      },
      websocket::Message::Binary(_) => {
        println!("received binary from a websocket");
      },
    }
  }
}
