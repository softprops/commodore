//! Call rank and take command of [Slack](https://slack.com/) with rust at your helm

#[macro_use]
extern crate log;
extern crate hyper;
extern crate url;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod response;

pub use response::Response;
use regex::{Captures as RegexCaptures, Regex};

use hyper::Client;
use hyper::header::ContentType;
use hyper::server::{Handler as HyperHandler, Request, Response as HyperResponse};
use std::collections::HashMap;
use std::io::Read;

const DEFAULT_RESPONSE: &'static [u8] = b"ok";

fn params<R: Read>(read: &mut R) -> HashMap<String, String> {
    let mut buffer = String::new();
    read.read_to_string(&mut buffer).unwrap();
    let mut params = HashMap::new();
    for (k, v) in url::form_urlencoded::parse(buffer.as_bytes()) {
        params.insert(k, v);
    }
    params
}

/// Results for regex matchers that collect captures
pub type Captures<'a> = RegexCaptures<'a>;

/// Deferred response interface
pub trait Responder: Sync + Send {
    /// Calling respond should update
    /// the channel or reply to the user
    /// that issued the original command
    fn respond(&self, response: Response) -> ();
}

#[doc(hidden)]
pub struct DefaultResponder {
    response_url: String,
}

impl Responder for DefaultResponder {
    fn respond(&self, response: Response) {
        let client = Client::new();
        let _ = client.post(&self.response_url[..])
                      .header(ContentType::json())
                      .body(serde_json::to_string(&response).unwrap().as_bytes())
                      .send();
    }
}

/// Command handling interface
/// Implementation for Fn
pub trait Handler: Sync + Send {
    /// handles Slack commands. Optional captures resulting
    /// from matching are provided along with an interface
    /// for deferred responses
    fn handle(&self,
              cmd: &Command,
              caps: &Option<Captures>,
              responder: Box<Responder>)
              -> Option<Response>;

    /// provides a mean explicit coersion for
    /// disambiguating cases where a fn named `handle` is
    /// already defined in another trait for which another
    /// impl exists for the same type
    fn as_handler(&self) -> &Handler;
}

impl<F> Handler for F
    where F: Fn(&Command, &Option<Captures>, Box<Responder>) -> Option<Response>,
          F: Send + Sync
{
    fn handle(&self,
              cmd: &Command,
              caps: &Option<Captures>,
              responder: Box<Responder>)
              -> Option<Response> {
        self(cmd, caps, responder)
    }

    fn as_handler(&self) -> &Handler {
        self
    }
}

#[doc(hidden)]
pub struct TokenValidator<H: Handler + 'static> {
    handler: H,
    token: String,
}

impl<H: Handler + 'static> Handler for TokenValidator<H> {
    fn handle(&self,
              cmd: &Command,
              caps: &Option<Captures>,
              responder: Box<Responder>)
              -> Option<Response> {
        if cmd.token == self.token {
            self.handler.handle(cmd, caps, responder)
        } else {
            error!("cmd token ${:?} did not match handler token ${:?}",
                   cmd.token,
                   self.token);
            None
        }
    }

    fn as_handler(&self) -> &Handler {
        self
    }
}

/// Command matching interface
pub trait Matcher: Send + Sync {
    /// returns of tuple of optional captures and an indicator for
    /// whether or not the provided command is matched
    fn matches<'a>(&self, cmd: &'a Command) -> (Option<Captures<'a>>, bool);
}

impl<F> Matcher for F
    where F: Fn(&Command) -> (Option<Captures>, bool),
          F: Send + Sync
{
    fn matches<'a>(&self, cmd: &'a Command) -> (Option<Captures<'a>>, bool) {
        self(cmd)
    }
}

/// A direct command matcher
pub struct MatchCommand(pub String);

impl Matcher for MatchCommand {
    fn matches<'a>(&self, cmd: &'a Command) -> (Option<Captures<'a>>, bool) {
        (None, cmd.command == self.0)
    }
}

/// A matcher that assumes any text starting with
/// the provided string is a subcommand. i.e. /cmd help
pub struct MatchSubCommand(pub String);

impl Matcher for MatchSubCommand {
    fn matches<'a>(&self, cmd: &'a Command) -> (Option<Captures<'a>>, bool) {
        if cmd.text.starts_with(&self.0) {
            (None, true)
        } else {
            debug!("regex {:?} did not match cmd {:?}", self.0, cmd.command);
            (None, false)
        }
    }
}

/// A regex pattern matcher for command text.
/// Regex captures will be provided to the matched Handler
pub struct MatchText(pub Regex);

impl Matcher for MatchText {
    fn matches<'a>(&self, cmd: &'a Command) -> (Option<Captures<'a>>, bool) {
        if self.0.is_match(cmd.text.as_ref()) {
            (self.0.captures(cmd.text.as_ref()), true)
        } else {
            debug!("regex {:?} did not match cmd {:?}", self.0, cmd.command);
            (None, false)
        }
    }
}

#[doc(hidden)]
pub struct Route {
    handler: Box<Handler>,
    matcher: Box<Matcher>,
}

/// A command de-multiplexor
#[derive(Default)]
pub struct Mux {
    routes: Vec<Box<Route>>,
}

impl Mux {
    pub fn new() -> Mux {
        Mux { ..Default::default() }
    }

    /// Install routing for a Slack command, secret token, and target Handler
    pub fn command<C, T, H>(&mut self, cmd: C, token: T, handler: H)
        where C: Into<String>,
              T: Into<String>,
              H: Handler + 'static
    {
        self.matching(MatchCommand(cmd.into()),
                      TokenValidator {
                          handler: handler,
                          token: token.into(),
                      })
    }

    /// Install routing for a Slack command matcher and target Handler
    pub fn matching<M, H>(&mut self, matcher: M, handler: H)
        where M: Matcher + 'static,
              H: Handler + 'static
    {
        let route = Route {
            handler: Box::new(handler),
            matcher: Box::new(matcher),
        };
        self.route(route)
    }

    /// Install a command routing
    pub fn route(&mut self, route: Route) {
        self.routes.push(Box::new(route));
    }

    /// Attempts to return the first match result for a target Handler
    pub fn handler<'a>(&self, cmd: &'a Command) -> Option<(Option<Captures<'a>>, &Box<Handler>)> {
        for r in self.routes.iter() {
            if let (captures, true) = r.matcher.matches(cmd) {
                return Some((captures, &r.handler));
            }
        }
        None
    }
}

impl Handler for Mux {
    fn handle(&self,
              cmd: &Command,
              _: &Option<Captures>,
              responder: Box<Responder>)
              -> Option<Response> {
        if let &Some((ref captures, handler)) = &self.handler(&cmd) {
            debug!("cmd matched. attempting to handle cmd {:#?}", cmd);
            handler.handle(&cmd, &captures, responder)
        } else {
            debug!("no matching handlers for {:#?}", cmd);
            None
        }
    }

    fn as_handler(&self) -> &Handler {
        self
    }
}

/// A struct representation of a Slack Command
/// and the context from which it was triggered
#[derive(Default, Debug, PartialEq)]
pub struct Command {
    pub token: String,
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub user_name: String,
    pub command: String,
    pub text: String,
    pub response_url: String,
}

impl Command {
    pub fn from_params(params: HashMap<String, String>) -> Option<Command> {
        if let (Some(token),
                Some(team_id),
                Some(team_domain),
                Some(channel_id),
                Some(channel_name),
                Some(user_id),
                Some(user_name),
                Some(command),
                Some(text),
                Some(response_url)) = (params.get("token"),
                                       params.get("team_id"),
                                       params.get("team_domain"),
                                       params.get("channel_id"),
                                       params.get("channel_name"),
                                       params.get("user_id"),
                                       params.get("user_name"),
                                       params.get("command"),
                                       params.get("text"),
                                       params.get("response_url")) {
            Some(Command {
                token: token.clone(),
                team_id: team_id.clone(),
                team_domain: team_domain.clone(),
                channel_id: channel_id.clone(),
                channel_name: channel_name.clone(),
                user_id: user_id.clone(),
                user_name: user_name.clone(),
                command: command.clone(),
                text: text.clone(),
                response_url: response_url.clone(),
            })
        } else {
            None
        }
    }
}

impl HyperHandler for Mux {
    // https://api.slack.com/slash-commands
    fn handle(&self, req: Request, mut res: HyperResponse) {
        let (_, _, _, _, _, mut body) = req.deconstruct();

        // parse params
        let params = params(&mut body);
        // parse cmd
        if let Some(cmd) = Command::from_params(params) {
            debug!("rec cmd {:?}", cmd);
            let write = |bytes: &[u8], content_type: ContentType| {
                res.headers_mut().set(content_type);
                let _ = res.send(bytes);
            };
            let responder = DefaultResponder { response_url: cmd.response_url.clone() };
            if let Some(resp) = self.as_handler().handle(&cmd, &None, Box::new(responder)) {
                match serde_json::to_string(&resp) {
                    Ok(payload) => write(payload.as_bytes(), ContentType::json()),
                    _ => write(DEFAULT_RESPONSE, ContentType::plaintext()),
                };
            } else {
                write(DEFAULT_RESPONSE, ContentType::plaintext())
            };
        } else {
            error!("rec invalid cmd");
            let _ = res.send(DEFAULT_RESPONSE);
        }
        ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::regex::Regex;
    use std::collections::HashMap;

    #[test]
    fn matches_commands() {
        let cmd = Command { command: "/test".to_owned(), ..Default::default() };
        let (_, matched) = MatchCommand("/test".to_owned()).matches(&cmd);
        assert!(matched, "cmd did not match")
    }

    #[test]
    fn matches_text() {
        let cmd = Command { text: "/test hello world".to_owned(), ..Default::default() };
        let (captures, matched) = MatchText(Regex::new(r"(?P<greeting>\S+?) (?P<name>\S+?)$")
                                                .unwrap())
                                      .matches(&cmd);
        assert!(matched, "cmd did not match");
        match captures {
            Some(caps) => {
                assert_eq!(caps.name("greeting"), Some("hello"));
                assert_eq!(caps.name("name"), Some("world"));
            }
            _ => assert!(false, "expected captures"),
        }
    }

    #[test]
    fn extracts_commands() {
        let mut params = HashMap::new();
        params.insert("token".to_owned(), "test_token".to_owned());
        params.insert("team_id".to_owned(), "test_team".to_owned());
        params.insert("team_domain".to_owned(), "test_team_domain".to_owned());
        params.insert("channel_id".to_owned(), "test_channel_id".to_owned());
        params.insert("channel_name".to_owned(), "test_channel_name".to_owned());
        params.insert("user_id".to_owned(), "test_user_id".to_owned());
        params.insert("user_name".to_owned(), "test_user_name".to_owned());
        params.insert("command".to_owned(), "test_command".to_owned());
        params.insert("text".to_owned(), "test_text".to_owned());
        params.insert("response_url".to_owned(), "test_response_url".to_owned());
        match Command::from_params(params) {
            Some(cmd) => {
                assert_eq!(cmd,
                           Command {
                               token: "test_token".to_owned(),
                               team_id: "test_team".to_owned(),
                               team_domain: "test_team_domain".to_owned(),
                               channel_id: "test_channel_id".to_owned(),
                               channel_name: "test_channel_name".to_owned(),
                               user_id: "test_user_id".to_owned(),
                               user_name: "test_user_name".to_owned(),
                               command: "test_command".to_owned(),
                               text: "test_text".to_owned(),
                               response_url: "test_response_url".to_owned(),
                           })
            }
            _ => assert!(false, "failed to extract command"),
        }
    }
}
