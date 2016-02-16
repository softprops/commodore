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
use hyper::server::{Handler, Request, Response as HyperResponse};
use std::collections::HashMap;
use std::io::Read;

fn params<R: Read>(read: &mut R) -> HashMap<String, String> {
    let mut buffer = String::new();
    read.read_to_string(&mut buffer).unwrap();
    let mut params = HashMap::new();
    for (k, v) in url::form_urlencoded::parse(buffer.as_bytes()) {
        params.insert(k, v);
    }
    params
}

pub type Captures<'a> = RegexCaptures<'a>;

/// Responds to commands after some deferred time
/// This allows command handlers to asyncronously
/// response to commands
pub trait Responder {
    fn respond(&self, response: Response) -> ();
}

pub struct DefaultResponder {
    response_url: String,
}


impl Responder for DefaultResponder {
    fn respond(&self, response: Response) {
        let client = Client::new();
        let _ = client.post(&self.response_url[..])
                      .body(serde_json::to_string(&response).unwrap().as_bytes())
                      .send();
    }
}

pub struct DiscardResponder;

impl Responder for DiscardResponder {
    fn respond(&self, _: Response) {}
}

/// A Handle handles matched commands
pub trait Handle: Sync + Send {
    fn handle(&self,
              cmd: &Command,
              caps: &Option<Captures>,
              responder: Box<Responder>)
              -> Option<Response>;
}

impl<F> Handle for F
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

pub struct TokenValidator<H: Handle + 'static> {
    handler: H,
    token: String,
}

impl<H: Handle + 'static> Handle for TokenValidator<H> {
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

/// A binding between a command handler and which commands its handles
pub struct Route {
    handler: Box<Handle>,
    matcher: Box<Matcher>,
}

///
/// A command de-multiplexor
///
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
              H: Handle + 'static
    {
        self.matching(MatchCommand(cmd.into()),
                      TokenValidator {
                          handler: handler,
                          token: token.into(),
                      })
    }

    pub fn matching<M, H>(&mut self, matcher: M, handler: H)
        where M: Matcher + 'static,
              H: Handle + 'static
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

    pub fn handler<'a>(&self, cmd: &'a Command) -> Option<(Option<Captures<'a>>, &Box<Handle>)> {
        for r in self.routes.iter() {
            if let (captures, true) = r.matcher.matches(cmd) {
                return Some((captures, &r.handler));
            }
        }
        None
    }

    pub fn handle(&self, cmd: &Command) {
        // set up responder
        let responder = DefaultResponder { response_url: cmd.response_url.clone() };
        // handle cmd
        if let &Some((ref captures, handler)) = &self.handler(&cmd) {
            info!("attempting to handle cmd ${:?}", cmd);
            handler.handle(&cmd, &captures, Box::new(responder));
        }
    }
}

#[derive(Default, Debug)]
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

impl Handler for Mux {
    // https://api.slack.com/slash-commands
    fn handle(&self, req: Request, res: HyperResponse) {
        let (_, _, _, _, _, mut body) = req.deconstruct();
        // parse params
        let params = params(&mut body);
        // parse cmd
        if let Some(cmd) = Command::from_params(params) {
            info!("rec cmd {:?}", cmd);
            self.handle(&cmd)
        } else {
            error!("rec invalid cmd")
        }
        let _ = res.send(b"ok");
        ()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::regex::Regex;

    #[test]
    fn matches_commands() {
        let cmd = Command { command: "/test".to_owned(), ..Default::default() };
        let (_, matched) = MatchCommand("/test".to_owned()).matches(&cmd);
        assert!(matched)
    }

    #[test]
    fn matches_regexes() {
        let cmd = Command { command: "/test".to_owned(), ..Default::default() };
        let (_, matched) = MatchRegex(Regex::new("(test)").unwrap()).matches(&cmd);
        assert!(matched)
    }
}
