use crate::reduce::add_ugen_state;
use crate::state::State;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

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

fn do_thing(m: &WebMessage, s: &mut State) {
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

#[post("/api/action")]
async fn action(
  message: web::Json<WebMessage>,
  extra: actix_web::web::Data<Arc<Mutex<State>>>,
) -> impl Responder {
  let mut s = extra.lock().unwrap();
  do_thing(&message, &mut s);
  println!("got: {:?}", message);
  HttpResponse::Ok().body("{}")
}

#[actix_web::main]
pub async fn serve(sg: Arc<Mutex<State>>) -> std::io::Result<()> {
  HttpServer::new(move || {
    App::new()
      .app_data(actix_web::web::Data::new(sg.clone()))
      .service(action)
      .service(actix_files::Files::new("/", "./public").index_file("index.html"))
  })
  .bind(("127.0.0.1", 8000))?
  .run()
  .await
}
