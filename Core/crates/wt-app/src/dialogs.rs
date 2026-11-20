//! A page says something, or asks (016): `alert`, `confirm` and `prompt`.
//! The core decides whether the person sees it, in the tab and under the
//! site's name, and tells the shell the page's answer.

use wt_core::{Address, WebViewId};
use wt_proto::v1::{DialogAnswer, DialogKind, DialogRequested, engine_event, host_request};

use crate::app::App;
use crate::model::{AskedFor, Asker, Waited};

/// How much of what a page says is shown, in characters (016 §3). A page
/// can say as much as it likes; a prompt is not a page.
pub const DIALOG_WORDS: usize = 2_000;

/// What a page said, as much of it as is shown: whole characters.
fn cut(words: &str) -> String {
    words.chars().take(DIALOG_WORDS).collect()
}

impl App {
    /// A shell's word that a page says something or asks. False when the
    /// event was not that.
    pub(crate) fn dialog_event(&self, web_view: WebViewId, kind: &engine_event::Kind) -> bool {
        let engine_event::Kind::DialogRequested(asked) = kind else {
            return false;
        };
        let request = asked.request_id;
        // Said twice: the one it names is the one that waits.
        if self.state.borrow().is_asking(request, Waited::Dialog) {
            return true;
        }
        if !self.show_dialog(web_view, asked) {
            // Nothing is shown: the page is answered as if the person had
            // closed it.
            self.answer_dialog(request, false, "");
        }
        true
    }

    /// Puts what a page says before the person, when it may be (016 §3).
    /// False when it is dismissed with nothing shown.
    fn show_dialog(&self, web_view: WebViewId, asked: &DialogRequested) -> bool {
        let kind = asked.kind();
        if kind == DialogKind::Unspecified {
            return false;
        }
        let Some(speaker) = self.state.borrow().speaker(web_view) else {
            return false;
        };
        // Who speaks must be the page itself: a frame from another site
        // does not put words before the person under this one's name.
        let origin = Address::parse(&asked.origin)
            .ok()
            .and_then(|from| from.origin());
        if origin.as_deref() != Some(speaker.origin.as_str()) || speaker.silenced {
            return false;
        }
        let what = AskedFor::Dialog {
            kind,
            message: cut(&asked.message),
            default_text: if kind == DialogKind::Prompt {
                cut(&asked.default_text)
            } else {
                String::new()
            },
            may_silence: speaker.spoke_before,
        };
        let asker = Asker {
            login_set: speaker.login_set,
            origin: speaker.origin,
            site: speaker.site,
        };
        let mut state = self.state.borrow_mut();
        let shown = state.ask(web_view, asked.request_id, asker, Vec::new(), what);
        if shown {
            state.spoke(web_view);
        }
        shown
    }

    /// Tells the shell the page's answer: OK, with the text for a prompt,
    /// or as if the person had closed it.
    pub(crate) fn answer_dialog(&self, request: u64, accepted: bool, text: &str) {
        let text = if accepted {
            text.to_owned()
        } else {
            String::new()
        };
        let answering = self
            .host
            .request(host_request::Kind::DialogAnswer(DialogAnswer {
                request_id: request,
                accepted,
                text,
            }));
        // Asked already; a shell that no longer has the request has no
        // page to answer.
        tokio::task::spawn_local(async move {
            let _ = answering.await;
        });
    }
}
