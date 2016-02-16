# commodore

[![Build Status](https://travis-ci.org/softprops/commodore.svg?branch=master)](https://travis-ci.org/softprops/commodore)

> call rank and take command of slack with rust at your helm

Commodore allows you to easily extend your [Slack](https://slack.com/) expeience with [Rust](https://www.rust-lang.org/) via Slack's [Command API](https://api.slack.com/slash-commands).

## usage

```rust
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
    mux.command("/commodore",
                "secrettoken",
                |c: &Command, _: &Option<Captures>, _: Box<Responder>| -> Option<Response> {
                    info!("handler recv cmd {:#?}", c);
                    None
                });
    let srvc = Server::http(&addr[..])
                   .unwrap()
                   .handle(mux);
    println!("listening on {}", addr);
    srvc.unwrap();
}
```

Doug Tangren (softprops) 2016
