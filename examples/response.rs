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
