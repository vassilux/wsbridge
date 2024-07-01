use crate::connection_manager::ConnectionManager;
use crate::messages::{Connect, Disconnect, WsMessage};
use actix::prelude::*;
use actix_web_actors::ws;
use futures::sink::SinkExt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{split, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConn {
    hb: Instant,
    id: Uuid,
    writer: Option<Arc<Mutex<FramedWrite<WriteHalf<TcpStream>, LinesCodec>>>>,
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
                        LinesCodec::new(),
                    ))));
                    let framed_reader = FramedRead::new(reader, LinesCodec::new());
                    TcpStreamHandler::start(framed_reader, ctx.address());
                }
                Err(err) => {
                    println!("Failed to connect to TCP server: {}", err);
                    ctx.stop();
                }
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
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
                if let Some(writer) = &self.writer {
                    let writer = Arc::clone(writer);
                    actix::spawn(async move {
                        let mut writer_guard: std::sync::MutexGuard<
                            FramedWrite<WriteHalf<TcpStream>, LinesCodec>,
                        > = writer.lock().unwrap();
                        let _ = writer_guard.send(bin.into()).await;
                    });
                }
            }
            Ok(ws::Message::Text(text)) => {
                if let Some(writer) = &self.writer {
                    let writer = Arc::clone(writer);
                    actix::spawn(async move {
                        let mut writer = writer.lock().unwrap();
                        let _ = writer.send(text.into()).await;
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
    framed: FramedRead<ReadHalf<TcpStream>, LinesCodec>,
    ws_conn: Addr<WsConn>,
}

impl TcpStreamHandler {
    pub fn start(
        framed: FramedRead<ReadHalf<TcpStream>, LinesCodec>,
        ws_conn: Addr<WsConn>,
    ) -> Addr<Self> {
        Self::create(|ctx| {
            ctx.add_stream(framed);
            Self { framed, ws_conn }
        })
    }
}

impl Actor for TcpStreamHandler {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, LinesCodecError>> for TcpStreamHandler {
    fn handle(&mut self, msg: Result<String, LinesCodecError>, ctx: &mut Self::Context) {
        match msg {
            Ok(data) => {
                self.ws_conn.do_send(WsMessage(data));
            }
            Err(e) => {
                eprintln!("Failed to read from TCP stream: {:?}", e);
                ctx.stop();
            }
        }
    }
}
