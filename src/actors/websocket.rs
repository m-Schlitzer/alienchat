extern crate actix;

use actix::prelude::*;
use actix_web::{ws, Error, HttpRequest, HttpResponse};
use actors::chatserver;
use std::time::Instant;

// This is our websocket route state, this state is shared with all route instances
// via `HttpContext::state()`
pub struct WsChatSessionState {
    pub addr: Addr<Syn, chatserver::ChatServer>,
}

// Entry point for our route
pub fn chat_route(req: HttpRequest<WsChatSessionState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
            name: None,
        },
    )
}

pub struct WsChatSession {
    // unique session id
    id: usize,
    // Client must send ping at least once per 10 seconds, otherwise we drop connection.
    hb: Instant,
    // joined room
    room: String,
    // peer name
    name: Option<String>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self, WsChatSessionState>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared across all
        // routes within application
        let addr: Addr<Syn, _> = ctx.address();
        ctx.state()
            .addr
            .send(chatserver::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                actix::fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server
        ctx.state()
            .addr
            .do_send(chatserver::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<chatserver::SessionMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: chatserver::SessionMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsChatSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Pong(_msg) => self.hb = Instant::now(),
            ws::Message::Text(text) => {
                let m = text.trim();

                let msg = if let Some(ref name) = self.name {
                    format!("{}: {}", name, m)
                } else {
                    m.to_owned()
                };
                // send message to chat server
                ctx.state().addr.do_send(chatserver::Message {
                    id: self.id,
                    msg: msg,
                    room: self.room.clone(),
                })
            }
            ws::Message::Binary(_bin) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}
