use crate::messages::{Connect, Disconnect, WsMessage};
use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ConnectionManager {
    sessions: HashMap<Uuid, Recipient<WsMessage>>,
}

impl Default for ConnectionManager {
    fn default() -> ConnectionManager {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for ConnectionManager {
    type Context = Context<Self>;
}

impl Handler<Connect> for ConnectionManager {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.insert(msg.self_id, msg.addr);
    }
}

impl Handler<Disconnect> for ConnectionManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}
