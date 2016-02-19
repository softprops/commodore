
/// A payload to reply to commands with
include!(concat!(env!("OUT_DIR"), "/response.rs"));

impl Response {
    /// returned a response which will displayed for the user that issued the command
    /// for formatting rules see [this doc](https://api.slack.com/docs/formatting)
    pub fn ephemeral<T>(text: T) -> Response
        where T: Into<String>
    {
        Response {
            text: text.into(),
            response_type: "ephemeral".to_owned(),
        }
    }
    /// return a response which will be displayed for anyone in the channel
    /// for formatting rules see [this doc](https://api.slack.com/docs/formatting)
    pub fn in_channel<T>(text: T) -> Response
        where T: Into<String>
    {
        Response {
            text: text.into(),
            response_type: "in_channel".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::serde_json;

    #[test]
    fn test_ephemeral_response() {
        match serde_json::to_string(&Response::ephemeral("test")) {
            Ok(json) => assert_eq!(json, r#"{"text":"test","response_type":"ephemeral"}"#),
            _ => assert!(false, "failed to serialize json"),
        }
    }

    #[test]
    fn test_in_channel_response() {
        match serde_json::to_string(&Response::in_channel("test")) {
            Ok(json) => assert_eq!(json, r#"{"text":"test","response_type":"in_channel"}"#),
            _ => assert!(false, "failed to serialize json"),
        }
    }
}
