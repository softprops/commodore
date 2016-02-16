

/// A payload to reply to commands with
/// for formatting rules see [this doc](https://api.slack.com/docs/formatting)
/// for attachments see [this doc](https://api.slack.com/docs/attachments)
include!(concat!(env!("OUT_DIR"), "/rep.rs"));

impl Response {
    pub fn builder<T>(text: T) -> ResponseBuilder
        where T: Into<String>
    {
        ResponseBuilder::new(text)
    }
}

/// builder interface for responses
#[derive(Default)]
pub struct ResponseBuilder {
    text: String,
    response_type: String,
}

impl ResponseBuilder {
    /// constructs a new reponse builder, by default with
    /// the text of an `ephemeral` response
    pub fn new<T>(text: T) -> ResponseBuilder
        where T: Into<String>
    {
        ResponseBuilder {
            text: text.into(),
            response_type: "ephemeral".to_owned(),
        }
    }

    pub fn ephemeral(&mut self) -> &mut ResponseBuilder {
        self.response_type = "ephemeral".to_owned();
        self
    }

    pub fn in_channel(&mut self) -> &mut ResponseBuilder {
        self.response_type = "in_channel".to_owned();
        self
    }

    pub fn build(&self) -> Response {
        Response {
            text: self.text.clone(),
            response_type: self.response_type.clone(),
        }
    }
}
