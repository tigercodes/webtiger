//! A server asks who is there before it answers (017): HTTP authentication
//! with a name and a password. The core decides whether the person is
//! asked, in the tab and under the server's name, and tells the shell the
//! answer. Nothing typed is kept here.

use wt_core::{Address, WebViewId};
use wt_proto::v1::{SignInAnswer, SignInRequested, engine_event, host_request};

use crate::app::App;
use crate::model::{AskedFor, Waited};

/// How much of a server's name for what it protects is kept, in
/// characters. It is only compared, never shown.
const REALM_LENGTH: usize = 200;

impl App {
    /// A shell's word that a server asks who is there. False when the event
    /// was not that.
    pub(crate) fn sign_in_event(&self, web_view: WebViewId, kind: &engine_event::Kind) -> bool {
        let engine_event::Kind::SignInRequested(asked) = kind else {
            return false;
        };
        let request = asked.request_id;
        // Said twice: the one it names is the one that waits.
        if self.state.borrow().is_asking(request, Waited::SignIn) {
            return true;
        }
        if !self.show_sign_in(web_view, asked) {
            // Nobody is asked: the server gets no answer.
            self.answer_sign_in(request, None);
        }
        true
    }

    /// Puts a server's question before the person, when the server is the
    /// tab's own (017 §3). False when it is given no answer with nobody
    /// asked.
    fn show_sign_in(&self, web_view: WebViewId, asked: &SignInRequested) -> bool {
        let server =
            self.state
                .borrow()
                .server(web_view, &asked.host, &asked.origin, &asked.heading);
        let Some(server) = server else {
            return false;
        };
        let confidential =
            Address::parse(&server.origin).is_ok_and(|server| server.is_trustworthy());
        let what = AskedFor::SignIn {
            realm: asked.realm.chars().take(REALM_LENGTH).collect(),
            confidential,
            again: asked.again,
        };
        self.state
            .borrow_mut()
            .ask(web_view, asked.request_id, server, Vec::new(), what)
    }

    /// Tells the shell how a server is answered: with a name and a
    /// password, or not at all.
    pub(crate) fn answer_sign_in(&self, request: u64, typed: Option<(&str, &str)>) {
        let (name, password) = typed.unwrap_or_default();
        let answering = self
            .host
            .request(host_request::Kind::SignInAnswer(SignInAnswer {
                request_id: request,
                accepted: typed.is_some(),
                name: name.to_owned(),
                password: password.to_owned(),
            }));
        // Asked already; a shell that no longer has the request has no
        // server to answer.
        tokio::task::spawn_local(async move {
            let _ = answering.await;
        });
    }

    /// A web view's load is over, or another began: the servers that asked
    /// who is there in it are given no answer.
    pub(crate) fn sign_ins_ended(&self, web_view: WebViewId) {
        let ended = self.state.borrow_mut().end_sign_ins(web_view);
        for settled in ended {
            self.settle(settled, false);
        }
    }
}
