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
//use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod actors;
mod controller;
mod external_data_source;
mod mock_data;
mod role;
mod room;
mod user;

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
    let server: Addr<Syn, _> = Arbiter::start(|_| actors::chatserver::ChatServer::default());

    fn user_resources(app: App) -> App {
        app.resource("/user", |r| {
            r.get().f(|_| HttpResponse::Ok());
            r.post().f(|_| HttpResponse::Created());
            r.put().f(|_| HttpResponse::Ok());
            r.delete().f(|_| HttpResponse::Ok());
        })
    }

    fn room_resources(app: App) -> App {
        app.resource("/room", |r| {
            r.get().f(|_| HttpResponse::Ok());
            r.post().f(|_| HttpResponse::Created());
            r.put().f(|_| HttpResponse::Ok());
            r.delete().f(|_| HttpResponse::Ok());
        })
    }

    HttpServer::new(
        move || { vec![
            App::new()
                .prefix("/api/v1")
                .configure(user_resources)
                .configure(room_resources)
                .boxed(),
            // Websocket sessions state
            App::with_state(actors::websocket::WsChatSessionState { addr: server.clone() })
                .middleware(middleware::Logger::default())
                 // redirect to websocket.html
                .resource("/", |r| r.method(http::Method::GET).f(|_| {
                    HttpResponse::Found()
                        .header("LOCATION", "/ws")
                        .finish()
                }))
                // websocket route
                .resource("/ws/", |r| r.route().f(actors::websocket::chat_route))
                .boxed()
        ]
    })
    .bind("0.0.0.0:1888").expect("Cannot bind to 0.0.0.0:1888")
//    .start_ssl(builder).unwrap();
    .start();

    println!("Server is up and running!");
    let _ = sys.run();
}
