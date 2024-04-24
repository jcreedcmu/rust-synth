use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use tokio::sync::mpsc::{channel, Sender};

// Useful docs, example for app_data:
// https://docs.rs/actix-web/latest/actix_web/web/struct.Data.html
// https://github.com/actix/actix-web/discussions/2805

#[derive(Deserialize, Debug)]
pub enum WebAction {
  Quit,
  Drum,
}

#[derive(Deserialize, Debug)]
pub struct WebMessage {
  pub message: WebAction,
}

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

pub fn start<C>(k: C)
where
  C: Fn(&WebMessage) + Send + 'static,
{
  let (tx, mut rx) = channel::<WebMessage>(100);
  std::thread::spawn(move || {
    serve(tx).unwrap();
  });
  std::thread::spawn(move || loop {
    match rx.blocking_recv() {
      None => break,
      Some(msg) => k(&msg),
    }
  });
}
