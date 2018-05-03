extern crate actix;
extern crate actix_web;
extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate openssl;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;

use actix::*;
use actix_web::{http, server::HttpServer, middleware, ws, App, Error, HttpRequest, HttpResponse};
//use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::time::Instant;

#[macro_use]

// This is our websocket route state, this state is shared with all route instances
// via `HttpContext::state()`
struct WsChatSessionState {
    addr: Addr<Syn, server::ChatServer>,
}

// Entry point for our route
fn chat_route(req: HttpRequest<WsChatSessionState>) -> Result<HttpResponse, Error> {
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

struct WsChatSession {
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
            .send(server::Connect {
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
        ctx.state().addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::SessionMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::SessionMessage, ctx: &mut Self::Context) {
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
                ctx.state().addr.do_send(server::Message {
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
mod chatserver;

fn main() {
    // Enable logger
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Enable ssl and get certs
    //    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    //    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    //    builder.set_certificate_chain_file("cert.pem").unwrap();

    let sys = actix::System::new("chat");

    //Start chat server actor in seperate thread
    let server: Addr<Syn, _> = Arbiter::start(|_| chatserver::ChatServer::default());

    HttpServer::new(
        move || {
            // Websocket sessions state
            let state = WsChatSessionState { addr: server.clone() };

            App::with_state(state)
                .middleware(middleware::Logger::default())
                 // redirect to websocket.html
                .resource("/", |r| r.method(http::Method::GET).f(|_| {
                    HttpResponse::Found()
                        .header("LOCATION", "/ws")
                        .finish()
                }))
                // websocket route
                .resource("/ws/", |r| r.route().f(chat_route))

    })
    .bind("0.0.0.0:1888").expect("Cannot bind to 0.0.0.0:8080")
//    .start_ssl(builder).unwrap();
    .start();

    println!("Server is up and running!");
    let _ = sys.run();
}
