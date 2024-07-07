use crate::connection_manager::ConnectionManager;
use crate::messages::{Connect, Disconnect, WsMessage};
use actix::prelude::*;
use actix_web_actors::ws;
use bytes::{Bytes, BytesMut};
use futures::sink::SinkExt;
use futures::stream::StreamExt; // Assurez-vous d'importer StreamExt
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{split, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConn {
    hb: Instant,
    id: Uuid,
    writer: Option<Arc<Mutex<FramedWrite<WriteHalf<TcpStream>, BytesCodec>>>>,
    connection_manager: Addr<ConnectionManager>,
}

impl WsConn {
    pub fn new(connection_manager: Addr<ConnectionManager>) -> Self {
        Self {
            hb: Instant::now(),
            id: Uuid::new_v4(),
            writer: None,
            connection_manager,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WsConn started");
        self.hb(ctx);

        let addr = ctx.address();
        self.connection_manager.do_send(Connect {
            addr: addr.recipient(),
            self_id: self.id,
        });

        // Connect to TCP server
        TcpStream::connect("127.0.0.1:9000")
            .into_actor(self)
            .map(|res, act, ctx| match res {
                Ok(stream) => {
                    let (reader, writer) = split(stream);
                    act.writer = Some(Arc::new(Mutex::new(FramedWrite::new(
                        writer,
                        BytesCodec::new(),
                    ))));
                    let framed_reader = FramedRead::new(reader, BytesCodec::new());
                    TcpStreamHandler::create_and_start(framed_reader, ctx.address());
                    println!("Connected to TCP server");
                }
                Err(err) => {
                    println!("Failed to connect to TCP server: {}", err);
                    ctx.stop();
                }
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("WsConn stopping");
        self.connection_manager.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => {
                println!("Received binary message: {:?}", bin); //
                if let Some(writer) = &self.writer {
                    let writer = Arc::clone(writer);
                    actix::spawn(async move {
                        let mut writer_guard = writer.lock().unwrap();
                        let bytes = Bytes::from(bin);
                        if let Err(e) = writer_guard.send(bytes).await {
                            println!("Error sending binary message: {:?}", e); // Log ajoutée
                        } else {
                            println!("Binary message routed"); // Log ajoutée
                        }
                    });
                }
            }
            Ok(ws::Message::Text(text)) => {
                println!("Received text message: {}", text);
                if let Some(writer) = &self.writer {
                    let writer = Arc::clone(writer);
                    actix::spawn(async move {
                        let mut writer_guard = writer.lock().unwrap();
                        let text_with_newline = format!("{}\n", text);
                        let bytes = Bytes::from(text_with_newline.into_bytes());
                        if let Err(e) = writer_guard.send(bytes).await {
                            println!("Error sending text message: {:?}", e); // Log ajoutée
                        } else {
                            println!("Text message routed"); // Log ajoutée
                        }
                    });
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

pub struct TcpStreamHandler {
    framed: Option<FramedRead<ReadHalf<TcpStream>, BytesCodec>>,
    ws_conn: Addr<WsConn>,
}

impl TcpStreamHandler {
    pub fn create_and_start(
        framed: FramedRead<ReadHalf<TcpStream>, BytesCodec>,
        ws_conn: Addr<WsConn>,
    ) -> Addr<Self> {
        let handler = Self {
            framed: Some(framed),
            ws_conn,
        };
        handler.start()
    }
}

impl Actor for TcpStreamHandler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("TcpStreamHandler started"); // Log ajoutée
        if let Some(framed) = self.framed.take() {
            ctx.add_stream(framed);
        }
    }
}

pub struct AddStreamMessage(pub FramedRead<ReadHalf<TcpStream>, BytesCodec>);

impl Message for AddStreamMessage {
    type Result = ();
}

impl Handler<AddStreamMessage> for TcpStreamHandler {
    type Result = ();

    fn handle(&mut self, msg: AddStreamMessage, ctx: &mut Self::Context) {
        ctx.add_stream(msg.0);
    }
}

impl StreamHandler<Result<BytesMut, std::io::Error>> for TcpStreamHandler {
    fn handle(&mut self, msg: Result<BytesMut, std::io::Error>, ctx: &mut Self::Context) {
        match msg {
            Ok(data) => {
                let text = String::from_utf8_lossy(&data).to_string();
                println!("Received text from TCP stream: {}", text); // Log ajoutée
                self.ws_conn.do_send(WsMessage(text));
            }
            Err(e) => {
                eprintln!("Failed to read from TCP stream: {:?}", e);
                ctx.stop();
            }
        }
    }
}
