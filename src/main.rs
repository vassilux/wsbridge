mod connection_manager;
mod messages;
mod ws_connection;
use actix::{Actor, Addr};
use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use connection_manager::ConnectionManager;
use ws_connection::WsConn;

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

async fn websocket(
    r: HttpRequest,
    stream: web::Payload,
    data: web::Data<Addr<ConnectionManager>>,
) -> impl Responder {
    println!("starting websocket ");
    let connection_manager = data.get_ref().clone();
    let ws = WsConn::new(connection_manager);
    ws::start(ws, &r, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection_manager = ConnectionManager::default().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(connection_manager.clone()))
            .service(web::resource("/").to(index))
            .route("/ws", web::get().to(websocket))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
