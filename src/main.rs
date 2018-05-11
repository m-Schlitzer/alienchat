extern crate actix;
extern crate actix_web;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate openssl;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate uuid;

#[macro_use]
extern crate serde_derive;

use actix::{Addr, Arbiter, Syn};
use actix_web::{http, middleware, server::HttpServer, App, HttpResponse};
use std::env;
//use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[macro_use]
mod chatserver;
mod cmdarguments;
mod controller;
mod external_data_source;
mod mock_data;
mod role;
mod room;
mod user;
mod websocket;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        cmdarguments::check_args(args);
    }

    // Enable logger
    env::set_var("RUST_LOG", "info");
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
            App::with_state(websocket::WsChatSessionState { addr: server.clone() })
                .middleware(middleware::Logger::default())
                 // redirect to websocket.html
                .resource("/", |r| r.method(http::Method::GET).f(|_| {
                    HttpResponse::Found()
                        .header("LOCATION", "/ws")
                        .finish()
                }))
                // websocket route
                .resource("/ws/", |r| r.route().f(websocket::chat_route))
    })
    .bind("0.0.0.0:1888").expect("Cannot bind to 0.0.0.0:8080")
//    .start_ssl(builder).unwrap();
    .start();

    println!("Server is up and running!");
    let _ = sys.run();
}
