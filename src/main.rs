extern crate hyper;
extern crate commodore;

use commodore::{Mux, Listener};
use hyper::server::Server;

fn main() {
    let addr = "0.0.0.0:6789";
    let mux: Mux = Default::default();
    let _ = Server::http(addr)
        .unwrap()
        .handle(Listener {
          mux: mux
        });
}
