#[macro_use]
extern crate log;
extern crate env_logger;

extern crate commodore;
extern crate hyper;

use commodore::{Captures, Command, Mux, Responder, Response};
use hyper::Server;
use std::thread;
use std::time::Duration;

pub fn main() {
    env_logger::init().unwrap();
    let mut mux = Mux::new();
    mux.command("/commodore", "secrettoken", |c: &Command,
                 _: &Option<Captures>,
                 responder: Box<Responder>|
                 -> Option<Response> {
        info!("handler recv cmd {:#?}", c);
        thread::spawn(move || {
            // simulate doing something important
            thread::sleep(Duration::from_secs(3));
            responder.respond(Response::ephemeral("some time later"));
        });
        Some(Response::ephemeral("got it"))
    });
    let svc = Server::http("0.0.0.0:4567")
                   .unwrap()
                   .handle(mux)
                   .unwrap();
    println!("listening on {}", svc.socket);
}
