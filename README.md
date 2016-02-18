# commodore

[![Build Status](https://travis-ci.org/softprops/commodore.svg?branch=master)](https://travis-ci.org/softprops/commodore) [![Coverage Status](https://coveralls.io/repos/github/softprops/commodore/badge.svg?branch=master)](https://coveralls.io/github/softprops/commodore?branch=master) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE)

> call rank and take command of slack with rust at your helm

Commodore allows you to easily extend your [Slack](https://slack.com/) expeience with [Rust](https://www.rust-lang.org/) via Slack's [Command API](https://api.slack.com/slash-commands).

## rust docs

Find them [here](https://softprops.github.io/commodore)

## usage

```rust
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate commodore;
extern crate hyper;

use commodore::{Captures, Command, Mux, Response, Responder};
use hyper::Server;
use std::thread;
use std::time::Duration;

pub fn main() {
    env_logger::init().unwrap();
    let addr = format!("0.0.0.0:{}", 4567);
    let mut mux = Mux::new();
    mux.command("/commodore",
                "secrettoken",
                |c: &Command, _: &Option<Captures>, responder: Box<Responder>| -> Option<Response> {
                    info!("handler recv cmd {:#?}", c);
                    thread::spawn(move || {
                        // simulate doing something important
                        thread::sleep(Duration::from_secs(3));
                        responder.respond(Response::builder("some time later").build());
                    });
                    Some(Response::builder("got it").build())
                });
    let srvc = Server::http(&addr[..])
                   .unwrap()
                   .handle(mux);
    println!("listening on {}", addr);
    srvc.unwrap();
}
```

Doug Tangren (softprops) 2016
