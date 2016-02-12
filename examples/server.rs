#[macro_use]
extern crate log;
extern crate env_logger;

extern crate commodore;
extern crate hyper;

use commodore::{Captures, Command, Mux, Response, Responder};
use hyper::Server;

pub fn main() {
    env_logger::init().unwrap();
    let addr = format!("0.0.0.0:{}", 4567);
    let mut mux = Mux::new();
    mux.command("/commadore",
                "secrettoken",
                |_: &Command, _: &Option<Captures>, _: Box<Responder>| -> Option<Response> {
                    None
                });
    let srvc = Server::http(&addr[..])
                   .unwrap()
                   .handle(mux);
    println!("listening on {}", addr);
    srvc.unwrap();
}
