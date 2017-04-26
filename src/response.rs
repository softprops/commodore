
/// A payload to reply to commands with
#[derive(Debug, Default, Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub response_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Attachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretext: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_icon: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer_icon: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<usize>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Field {
    pub title: String,
    pub value: String,
    pub short: bool,
}


impl Response {
    /// returned a response which will displayed for the user that issued the command
    /// for formatting rules see [this doc](https://api.slack.com/docs/formatting)
    pub fn ephemeral<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Response {
            text: Some(text.into()),
            response_type: "ephemeral".to_owned(),
            attachments: vec![],
        }
    }
    /// return a response which will be displayed for anyone in the channel
    /// for formatting rules see [this doc](https://api.slack.com/docs/formatting)
    pub fn in_channel<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Response {
            text: Some(text.into()),
            response_type: "in_channel".to_owned(),
            attachments: vec![],
        }
    }

    /// returns a builder interfaces for constructing instances
    /// of responses
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }
}

#[derive(Default)]
pub struct ResponseBuilder {
    text: Option<String>,
    response_type: String,
    attachments: Vec<Attachment>,
}

impl ResponseBuilder {
    pub fn new() -> ResponseBuilder {
        ResponseBuilder {
            response_type: "ephemeral".to_owned(),
            ..Default::default()
        }
    }

    pub fn text<T>(&mut self, text: T) -> &mut ResponseBuilder
    where
        T: Into<String>,
    {
        self.text = Some(text.into());
        self
    }

    pub fn ephemeral(&mut self) -> &mut ResponseBuilder {
        self.response_type = "ephemeral".to_owned();
        self
    }

    pub fn in_channel(&mut self) -> &mut ResponseBuilder {
        self.response_type = "in_channel".to_owned();
        self
    }

    pub fn attach(&mut self, at: Attachment) -> &mut ResponseBuilder {
        self.attachments.push(at);
        self
    }

    pub fn build(&self) -> Response {
        Response {
            text: self.text.clone(),
            response_type: self.response_type.clone(),
            attachments: self.attachments.iter().cloned().collect(),
        }
    }
}

/// see https://api.slack.com/docs/message-attachments
impl Attachment {
    pub fn new() -> Attachment {
        Attachment { ..Default::default() }
    }
    pub fn builder() -> AttachmentBuilder {
        AttachmentBuilder::new()
    }
}

#[derive(Default)]
pub struct AttachmentBuilder {
    text: Option<String>,
    color: Option<String>,
    fallback: Option<String>,
    pretext: Option<String>,

    title: Option<String>,
    title_link: Option<String>,

    author_name: Option<String>,
    author_link: Option<String>,
    author_icon: Option<String>,

    image_url: Option<String>,
    thumb_url: Option<String>,

    footer: Option<String>,
    footer_icon: Option<String>,

    ts: Option<usize>,

    fields: Vec<Field>,
}

impl AttachmentBuilder {
    pub fn new() -> AttachmentBuilder {
        AttachmentBuilder { ..Default::default() }
    }

    pub fn text<S>(&mut self, txt: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.text = Some(txt.into());
        self
    }

    pub fn color<S>(&mut self, color: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.color = Some(color.into());
        self
    }

    pub fn fallback<S>(&mut self, fallback: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.fallback = Some(fallback.into());
        self
    }

    pub fn pretext<S>(&mut self, pretext: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.pretext = Some(pretext.into());
        self
    }

    pub fn title<S>(&mut self, title: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    pub fn author_name<S>(&mut self, author_name: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.author_name = Some(author_name.into());
        self
    }

    pub fn author_link<S>(&mut self, author_link: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.author_link = Some(author_link.into());
        self
    }

    pub fn author_icon<S>(&mut self, author_icon: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.author_icon = Some(author_icon.into());
        self
    }

    pub fn image_url<S>(&mut self, image_url: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.image_url = Some(image_url.into());
        self
    }

    pub fn thumb_url<S>(&mut self, thumb_url: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.thumb_url = Some(thumb_url.into());
        self
    }

    pub fn footer<S>(&mut self, f: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.footer = Some(f.into());
        self
    }

    pub fn footer_icon<S>(&mut self, f: S) -> &mut AttachmentBuilder
    where
        S: Into<String>,
    {
        self.footer_icon = Some(f.into());
        self
    }

    pub fn field(&mut self, f: Field) -> &mut AttachmentBuilder {
        self.fields.push(f);
        self
    }

    pub fn ts(&mut self, s: usize) -> &mut AttachmentBuilder {
        self.ts = Some(s);
        self
    }

    pub fn build(&self) -> Attachment {
        Attachment {
            text: self.text.clone(),
            color: self.color.clone(),
            fallback: self.fallback.clone(),
            pretext: self.pretext.clone(),

            title: self.title.clone(),
            title_link: self.title_link.clone(),

            author_name: self.author_name.clone(),
            author_link: self.author_link.clone(),
            author_icon: self.author_icon.clone(),

            image_url: self.image_url.clone(),
            thumb_url: self.thumb_url.clone(),

            footer: self.footer.clone(),
            footer_icon: self.footer_icon.clone(),

            ts: self.ts.clone(),
            fields: self.fields.iter().cloned().collect(),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use super::super::serde_json;

    #[test]
    fn test_builder_default() {
        match serde_json::to_string(&Response::builder().build()) {
            Ok(json) => assert_eq!(json, r#"{"response_type":"ephemeral"}"#),
            _ => assert!(false, "failed to serialize json"),
        }
    }

    #[test]
    fn test_builder_options() {
        let res = Response::builder()
            .text("foo")
            .in_channel()
            .attach(
                Attachment::builder()
                    .text("attached")
                    .color("red")
                    .build(),
            )
            .build();
        match serde_json::to_string(&res) {
            Ok(json) => assert_eq!(
                json,
                r#"{"text":"foo","response_type":"in_channel","attachments":[{"text":"attached","color":"red"}]}"#
            ),
            _ => assert!(false, "failed to serialize json"),
        }

    }

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
