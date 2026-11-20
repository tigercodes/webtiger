//! A page asks to use something of the device's (014): the core answers
//! from what was allowed before, or asks the person in the tab, and
//! remembers a yes. The engine is told how to answer the page.

use std::time::Duration;

use tokio::time::Instant;
use wt_core::{Address, LoginSetId, TabId, WebViewId, WindowId};
use wt_proto::v1::{
    Ack, AnswerPrompt, ForgetPermissions, PermissionAnswer, PermissionKind, PermissionRequested,
    PermissionsPage, QueryPermissions, SitePermissions, engine_event, forget_permissions,
    host_request, response,
};

use crate::app::App;
use crate::error::AppError;
use crate::model::{AskedFor, Settled, Waited};

/// How long a prompt must have been before the person to be answered (014
/// §3). A press that was on its way to the page when the prompt appeared
/// under it, and the second press of two, must not answer for the person.
pub const PROMPT_DELAY: Duration = Duration::from_millis(500);

/// The name a kind is written by in the store. `None` for what is no kind.
fn name(kind: PermissionKind) -> Option<&'static str> {
    match kind {
        PermissionKind::Camera => Some("camera"),
        PermissionKind::Microphone => Some("microphone"),
        PermissionKind::Location => Some("location"),
        PermissionKind::Downloads => Some("downloads"),
        PermissionKind::Unspecified => None,
    }
}

/// The kind a name in the store is. `None` for a name this build does not
/// know: a later build's.
fn kind(name: &str) -> Option<PermissionKind> {
    match name {
        "camera" => Some(PermissionKind::Camera),
        "microphone" => Some(PermissionKind::Microphone),
        "location" => Some(PermissionKind::Location),
        "downloads" => Some(PermissionKind::Downloads),
        _ => None,
    }
}

/// What a request asks for: each kind once, in the kinds' own order. `None`
/// when it asks for nothing, or for something the core does not know how
/// to ask about.
fn kinds_asked(asked: &[i32]) -> Option<Vec<PermissionKind>> {
    let mut kinds = Vec::new();
    for number in asked {
        let kind = PermissionKind::try_from(*number).ok()?;
        name(kind)?;
        // To download is not an engine's to ask for: the core asks, before a
        // site's first download (015).
        if kind == PermissionKind::Downloads {
            return None;
        }
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds.sort_by_key(|kind| *kind as i32);
    (!kinds.is_empty()).then_some(kinds)
}

/// An origin as the core writes one, from what a shell or a screen says.
fn origin_of(text: &str) -> Option<String> {
    Address::parse(text).ok()?.origin()
}

impl App {
    /// A shell's word about a page's request. False when the event was not
    /// one.
    pub(crate) fn permission_event(&self, web_view: WebViewId, kind: &engine_event::Kind) -> bool {
        match kind {
            engine_event::Kind::PermissionRequested(asked) => {
                self.permission_requested(web_view, asked);
            }
            engine_event::Kind::PermissionWithdrawn(withdrawn) => {
                self.state
                    .borrow_mut()
                    .withdraw(web_view, withdrawn.request_id);
            }
            _ => return false,
        }
        true
    }

    /// A page asked: it is answered from what is known, or the person is
    /// asked (014 §3).
    fn permission_requested(&self, web_view: WebViewId, asked: &PermissionRequested) {
        let request = asked.request_id;
        // Said twice: the request it names is the one that waits.
        if self.state.borrow().is_asking(request, Waited::Permission) {
            return;
        }
        let Some(decided) = self.decide(web_view, asked) else {
            // Nothing is known that answers it: the person is asked. A tab
            // that is asking as much as it may asks no more.
            return;
        };
        self.answer_page(request, decided);
    }

    /// What the core can say of a request by itself: granted, or refused.
    /// `None` when the person has been asked.
    fn decide(&self, web_view: WebViewId, asked: &PermissionRequested) -> Option<bool> {
        let Some(kinds) = kinds_asked(&asked.kinds) else {
            return Some(false);
        };
        let Some(asker) = self.state.borrow().asker(web_view) else {
            return Some(false);
        };
        // Who asks must be the page itself: not a frame from elsewhere, and
        // not a page the tab has left since.
        if origin_of(&asked.origin).as_deref() != Some(asker.origin.as_str()) {
            return Some(false);
        }
        if self.state.borrow().refused_here(web_view, &kinds) {
            return Some(false);
        }
        let allowed = self.allowed(asker.login_set, &asker.origin);
        if kinds.iter().all(|kind| allowed.contains(kind)) {
            return Some(true);
        }
        let request = asked.request_id;
        let asking =
            self.state
                .borrow_mut()
                .ask(web_view, request, asker, kinds, AskedFor::Permission);
        (!asking).then_some(false)
    }

    /// What an origin was allowed in a login set. A store that refuses has
    /// allowed nothing: the person is asked.
    pub(crate) fn allowed(&self, login_set: LoginSetId, origin: &str) -> Vec<PermissionKind> {
        self.store
            .borrow()
            .allowed(login_set, origin)
            .unwrap_or_default()
            .iter()
            .filter_map(|name| kind(name))
            .collect()
    }

    /// What waited for the person has its answer: the page that asked is
    /// told, the download that waited starts or is dropped (015), or the
    /// page that spoke is answered as if the person had closed it (016).
    pub(crate) fn settle(&self, settled: Settled, granted: bool) {
        match settled.waited {
            Waited::Permission => self.answer_page(settled.request, granted),
            Waited::Download if granted => self.start_waiting(settled.request),
            Waited::Download => self.drop_waiting(settled.request),
            Waited::Dialog => self.answer_dialog(settled.request, false, ""),
            Waited::SignIn => self.answer_sign_in(settled.request, None),
        }
    }

    /// Tells the shell how to answer the page that asked.
    fn answer_page(&self, request: u64, granted: bool) {
        let answering = self
            .host
            .request(host_request::Kind::PermissionAnswer(PermissionAnswer {
                request_id: request,
                granted,
            }));
        // Asked already; this only waits for the shell's word, which changes
        // nothing: a shell that no longer has the request has no page to
        // answer.
        tokio::task::spawn_local(async move {
            let _ = answering.await;
        });
    }

    /// The page a web view showed is gone, by a new page or by its
    /// renderer: what it was asking is refused.
    pub(crate) fn prompts_ended(&self, web_view: WebViewId) {
        let ended = self.state.borrow_mut().end_prompts(web_view);
        for settled in ended {
            self.settle(settled, false);
        }
    }

    /// A window's snapshot is about to be given to its subscribers: the
    /// prompt in it, if it has one, is before the person from now. One that
    /// was shown already has been since it first was.
    pub(crate) fn prompt_shown(&self, window: WindowId, prompt: Option<u64>) {
        let mut shown = self.prompts_shown.borrow_mut();
        match prompt {
            Some(prompt) => {
                if shown.get(&window).map(|(id, _)| *id) != Some(prompt) {
                    shown.insert(window, (prompt, Instant::now()));
                }
            }
            None => {
                shown.remove(&window);
            }
        }
    }

    /// The person's answer to the prompt a tab shows.
    pub(crate) fn answer_prompt(&self, answer: &AnswerPrompt) -> Result<response::Kind, AppError> {
        let tab = TabId(answer.tab_id);
        let window = self
            .state
            .borrow()
            .window_of(tab)
            .ok_or_else(|| AppError::new("tab.unknown").field("tab_id"))?;
        // Only what a window was shown can be answered, and not in the
        // moment it appeared.
        match self.prompts_shown.borrow().get(&window) {
            Some((prompt, since)) if *prompt == answer.prompt_id => {
                if since.elapsed() < PROMPT_DELAY {
                    return Err(AppError::new("prompt.too_soon").field("prompt_id"));
                }
            }
            _ => return Err(AppError::new("prompt.unknown").field("prompt_id")),
        }
        let answered = self
            .state
            .borrow_mut()
            .answer_prompt(tab, answer.prompt_id, answer.allow);
        let (login_set, asking) = answered.map_err(|shown| {
            if shown {
                AppError::new("prompt.unknown").field("prompt_id")
            } else {
                AppError::new("tab.unknown").field("tab_id")
            }
        })?;
        // A server that asked who is there (017) has the name and the
        // password, or no answer; and so has every other question of that
        // server and realm that waits in the login set. Nothing of it is
        // remembered here.
        if let Some(realm) = asking.sign_in() {
            let typed = answer
                .allow
                .then_some((answer.text.as_str(), answer.secret.as_str()));
            self.answer_sign_in(asking.request, typed);
            let others = self
                .state
                .borrow_mut()
                .settle_signed_in(login_set, &asking.origin, realm);
            for other in others {
                self.answer_sign_in(other.request, typed);
            }
            return Ok(response::Kind::Ack(Ack {}));
        }
        // What a page said or asked (016) is answered by itself: nothing
        // is allowed by it, and nothing remembered but that it spoke.
        if let Some(may_silence) = asking.dialog() {
            let silence = answer.silence && may_silence;
            let accepted = answer.allow && !silence;
            self.answer_dialog(asking.request, accepted, &answer.text);
            if silence {
                let silenced = self.state.borrow_mut().silence(tab);
                for other in silenced {
                    self.settle(other, false);
                }
            }
            return Ok(response::Kind::Ack(Ack {}));
        }
        self.settle(asking.settled(), answer.allow);
        let settled = if !answer.allow {
            self.state.borrow_mut().settle_refused(tab)
        } else if asking.origin.is_empty() {
            // Nothing of the web's to remember it by: allowed this once.
            Vec::new()
        } else {
            let names: Vec<&str> = asking.kinds.iter().filter_map(|kind| name(*kind)).collect();
            // A store that refuses remembers nothing: the page has its
            // answer all the same, and the person is asked again next time.
            let _ = self
                .store
                .borrow_mut()
                .allow(login_set, &asking.origin, &names);
            let mut allowed = self.allowed(login_set, &asking.origin);
            allowed.extend(asking.kinds.iter().copied());
            self.state
                .borrow_mut()
                .settle_allowed(login_set, &asking.origin, &allowed)
        };
        // What else waited for the same answer has it.
        for other in settled {
            self.settle(other, answer.allow);
        }
        Ok(response::Kind::Ack(Ack {}))
    }

    /// The login set whose permissions a window shows: its Space's.
    fn permissions_of(&self, window: u64) -> Result<LoginSetId, AppError> {
        self.state
            .borrow()
            .history_of(WindowId(window))
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))
    }

    /// What the sites of a window's login set were allowed, in the order of
    /// their names as shown.
    pub(crate) fn query_permissions(
        &self,
        query: &QueryPermissions,
    ) -> Result<response::Kind, AppError> {
        let login_set = self.permissions_of(query.window_id)?;
        let all = self
            .store
            .borrow()
            .all_allowed(login_set)
            .map_err(|error| AppError::new(error.code()))?;
        let mut sites: Vec<SitePermissions> = all
            .into_iter()
            .filter_map(|allowed| {
                // A kind only a later build knows is not this build's to
                // show; an origin with none it knows is not listed.
                let kinds: Vec<i32> = allowed
                    .kinds
                    .iter()
                    .filter_map(|name| kind(name))
                    .map(|kind| kind as i32)
                    .collect();
                if kinds.is_empty() {
                    return None;
                }
                let site = Address::parse(&allowed.origin)
                    .map_or_else(|_| allowed.origin.clone(), |named| named.display());
                Some(SitePermissions {
                    origin: allowed.origin,
                    site,
                    kinds,
                })
            })
            .collect();
        sites.sort_by(|a, b| a.site.cmp(&b.site).then_with(|| a.origin.cmp(&b.origin)));
        Ok(response::Kind::PermissionsPage(PermissionsPage { sites }))
    }

    /// Takes back what one origin was allowed in a window's login set, or
    /// what every origin was. It stops nothing that is running.
    pub(crate) fn forget_permissions(
        &self,
        forget: &ForgetPermissions,
    ) -> Result<response::Kind, AppError> {
        let login_set = self.permissions_of(forget.window_id)?;
        let forgotten = match &forget.what {
            Some(forget_permissions::What::Origin(origin)) if !origin.is_empty() => {
                // An origin that is none of the web's was allowed nothing.
                match origin_of(origin) {
                    Some(origin) => self
                        .store
                        .borrow_mut()
                        .forget_permissions(login_set, &origin)
                        .map(|_| ()),
                    None => Ok(()),
                }
            }
            Some(forget_permissions::What::Everything(true)) => {
                self.store.borrow_mut().forget_all_permissions(login_set)
            }
            _ => return Err(AppError::new("permission.nothing_named").field("what")),
        };
        forgotten.map_err(|error| AppError::new(error.code()))?;
        Ok(response::Kind::Ack(Ack {}))
    }
}
