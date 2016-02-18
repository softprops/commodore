//! call rank and take command of slack with rust at your helm

#[macro_use]
extern crate log;
extern crate hyper;
extern crate url;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod rep;

pub use rep::Response;
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

/// Capture results for regex matchers that collect captures
pub type Captures<'a> = RegexCaptures<'a>;

/// Allows for responding ot commands after some delayed period of time
pub trait Responder: Sync + Send {
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

/// Handles matched commands
pub trait Handler: Sync + Send {
    fn handle(&self,
              cmd: &Command,
              caps: &Option<Captures>,
              responder: Box<Responder>)
              -> Option<Response>;
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
            None
        }
    }
}

/// A matcher matches commands
pub trait Matcher: Send + Sync {
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
///
pub struct MatchCommand(String);

impl Matcher for MatchCommand {
    fn matches<'a>(&self, cmd: &'a Command) -> (Option<Captures<'a>>, bool) {
        (None, cmd.command == self.0)
    }
}

/// A regex pattern matcher
///
pub struct MatchRegex(Regex);

impl Matcher for MatchRegex {
    fn matches<'a>(&self, cmd: &'a Command) -> (Option<Captures<'a>>, bool) {
        if self.0.is_match(cmd.command.as_ref()) {
            (self.0.captures(cmd.command.as_ref()), true)
        } else {
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

    ///
    pub fn route(&mut self, route: Route) {
        self.routes.push(Box::new(route));
    }

    pub fn handler<'a>(&self, cmd: &'a Command) -> Option<(Option<Captures<'a>>, &Box<Handler>)> {
        for r in self.routes.iter() {
            if let (captures, true) = r.matcher.matches(cmd) {
                return Some((captures, &r.handler));
            }
        }
        None
    }

    pub fn handle(&self, cmd: &Command) -> Option<Response> {
        // set up responder
        let responder = DefaultResponder { response_url: cmd.response_url.clone() };
        // handle cmd
        if let &Some((ref captures, handler)) = &self.handler(&cmd) {
            info!("attempting to handle cmd ${:?}", cmd);
            handler.handle(&cmd, &captures, Box::new(responder))
        } else {
            None
        }
    }
}

/// A struct representation of a Slack Command and
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
            info!("rec cmd {:?}", cmd);
            let write = |bytes: &[u8], content_type: ContentType| {
                res.headers_mut().set(content_type);
                let _ = res.send(bytes);
            };
            if let Some(resp) = self.handle(&cmd) {
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
    fn matches_regexes() {
        let cmd = Command { command: "/test hello world".to_owned(), ..Default::default() };
        let (captures, matched) = MatchRegex(Regex::new(r"(?P<greeting>\S+?) (?P<name>\S+?)$")
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
            _ => assert!(false, "failed to exact command"),
        }
    }
}
