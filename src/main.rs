mod connection_manager;
mod messages;
mod ws_connection;

use actix::{Actor, Addr};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use connection_manager::ConnectionManager;
use tokio::net::TcpStream;
use ws_connection::WsConn;

async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    data: web::Data<Addr<ConnectionManager>>,
) -> impl Responder {
    match TcpStream::connect("127.0.0.1:9000").await {
        Ok(tcp_stream) => {
            let connection_manager = data.get_ref().clone();
            let ws = WsConn::new(tcp_stream, connection_manager);
            ws::start(ws, &r, stream)
        }
        Err(e) => Ok({
            eprintln!("Failed to connect to TCP server: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection_manager = ConnectionManager::default().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(connection_manager.clone()))
            .route("/ws/", web::get().to(ws_index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
