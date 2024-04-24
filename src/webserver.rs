use crate::reduce::add_ugen_state;
use crate::state::State;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc::{channel, Receiver, Sender};

// Useful docs, example for app_data:
// https://docs.rs/actix-web/latest/actix_web/web/struct.Data.html
// https://github.com/actix/actix-web/discussions/2805

#[derive(Deserialize, Debug)]
enum WebAction {
  Quit,
  Drum,
}

#[derive(Deserialize, Debug)]
struct WebMessage {
  message: WebAction,
}

type Duplex<T> = (Sender<T>, Receiver<T>);

#[post("/api/action")]
async fn action(
  message: web::Json<WebMessage>,
  tx: actix_web::web::Data<Sender<WebMessage>>,
) -> impl Responder {
  tx.send(message.into_inner()).await.unwrap();
  HttpResponse::Ok().body("{}")
}

#[actix_web::main]
pub async fn serve(tx: Sender<WebMessage>) -> std::io::Result<()> {
  HttpServer::new(move || {
    App::new()
      .app_data(actix_web::web::Data::new(tx.clone()))
      .service(action)
      .service(actix_files::Files::new("/", "./public").index_file("index.html"))
  })
  .bind(("127.0.0.1", 8000))?
  .run()
  .await
}

fn reduce_web_message(m: &WebMessage, s: &mut State) {
  match m.message {
    WebAction::Drum => {
      let ugen = s.new_drum(1000.0);
      add_ugen_state(s, ugen);
    },
    WebAction::Quit => {
      s.going = false;
    },
  }
}

pub fn start(sg: Arc<Mutex<State>>) {
  let (tx, mut rx) = channel::<WebMessage>(100);
  std::thread::spawn(move || {
    serve(tx).unwrap();
  });
  std::thread::spawn(move || loop {
    match rx.blocking_recv() {
      None => break,
      Some(msg) => {
        let mut s: MutexGuard<State> = sg.lock().unwrap();
        reduce_web_message(&msg, &mut s);
      },
    }
  });
}
