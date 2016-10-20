# commodore

[![Build Status](https://travis-ci.org/softprops/commodore.svg?branch=master)](https://travis-ci.org/softprops/commodore) [![Coverage Status](https://coveralls.io/repos/github/softprops/commodore/badge.svg?branch=master)](https://coveralls.io/github/softprops/commodore?branch=master) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![crates.io](http://meritbadge.herokuapp.com/commodore)](https://crates.io/crates/commodore)

> call rank and take command of slack with rust at your helm

Commodore allows you to easily extend your [Slack](https://slack.com/) experience with [Rust](https://www.rust-lang.org/) via Slack's [Command API](https://api.slack.com/slash-commands).

## rust docs

Find them [here](https://softprops.github.io/commodore)

## usage

```rust
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
```

## responding

commodore supports a typed representation of slacks response structure. To
make creating instances of these convenient, builder instances are provided

```rust
extern crate commodore;

use commodore::{Attachment, Field, Response};

fn main() {
  let response = Response::builder()
    .text("hallo")
    .in_channel()
    .attach(
      Attachment::builder()
        .text("attached")
        .color("red")
        .field(
          Field {
            title: "foo".to_owned(),
            value: "value".to_owned(),
            short: false
          }
        )
        .build()
    ).build();
    println!("{:#?}", response);
}
```

Doug Tangren (softprops) 2016
