//! A scripted web engine: the pages it "serves" are a table, and loading one
//! produces the events a real engine reports, in the order 001 §5 asks every
//! adapter to report them.

use std::collections::{BTreeMap, BTreeSet};

use wt_proto::v1::{
    Ack, AddressChanged, CloseRequested, DialogKind, DialogRequested, DownloadFailed,
    DownloadFinished, DownloadProgress, DownloadRequested, EngineEvent, EngineKind, Error,
    FindResult, HistoryChanged, HostPong, LinkOpenRequested, NavigationCommand,
    NavigationCommitted, NavigationFailed, NavigationFinished, NavigationStarted,
    NewWindowRequested, OutsideLinkRequested, PermissionKind, PermissionRequested,
    PermissionWithdrawn, ProgressChanged, RendererGone, SearchRequested, SignInRequested,
    TitleChanged, engine_event, host_request, host_response,
};

/// A page of the scripted web.
#[derive(Clone, Debug, Default)]
pub struct FakePage {
    pub title: String,
    /// The server sends the browser on to this address.
    pub redirect: Option<String>,
    /// The load fails with this code instead.
    pub fail: Option<String>,
    /// The server's certificate is not trusted (026): the load fails with
    /// this code, and the engine holds the certificate, until it is told to
    /// let it through for the web view's login set.
    pub certificate: Option<String>,
}

/// A web view the fake shell holds.
#[derive(Clone, Debug)]
pub struct FakeWebView {
    pub engine: EngineKind,
    pub login_set: String,
    pub ephemeral: bool,
    /// The addresses committed, oldest first.
    pub history: Vec<String>,
    /// Where in `history` the web view is.
    pub index: usize,
}

/// The engine behind a [`crate::FakeShell`].
#[derive(Debug)]
pub struct FakeEngine {
    pages: BTreeMap<String, FakePage>,
    /// The web views that exist, by id.
    pub web_views: BTreeMap<u64, FakeWebView>,
    /// The engines this shell has. `WebViewCreate` for another is refused.
    pub engines: Vec<EngineKind>,
    /// How many web views this shell has made unasked: each is named by the
    /// next id of the shell's own.
    own_web_views: u64,
    /// The addresses this shell was asked to open with the system, in order.
    /// It opens nothing.
    pub opened_outside: Vec<String>,
    /// Nothing on this device opens an outside link: `OpenOutside` is
    /// answered with "system.no_handler".
    pub nothing_opens_outside: bool,
    /// What this shell was asked to find, in order: the web view, the text
    /// (empty to stop), and whether backwards. It finds nothing by itself:
    /// a test reports what a page would have.
    pub finds: Vec<(u64, String, bool)>,
    /// Every zoom a web view was asked for, in order (012): the web view and
    /// the per cent.
    pub zooms: Vec<(u64, u32)>,
    /// Every download the shell was asked to start, in order (013): the
    /// shell's name for it and the name to save it under.
    pub download_starts: Vec<(u64, String)>,
    /// Every download the shell was asked to drop, in order.
    pub download_cancels: Vec<u64>,
    /// Downloads the shell will say it does not have when asked to start
    /// them.
    pub downloads_not_started: Vec<u64>,
    /// How this shell was told to answer the pages that asked to use
    /// something (014), in order: its name for the request, and whether it
    /// was granted.
    pub permission_answers: Vec<(u64, bool)>,
    /// How this shell was told to answer the pages that said something or
    /// asked (016), in order: its name for the dialog, whether it was
    /// accepted, and the text.
    pub dialog_answers: Vec<(u64, bool, String)>,
    /// How this shell was told to answer the servers that asked who is
    /// there (017), in order: its name for the question, and the name and
    /// password, or `None` for no answer.
    pub sign_in_answers: Vec<(u64, Option<(String, String)>)>,
    /// The certificates this shell lets through (026): each a login set and
    /// the origin whose certificate it is.
    trusted: BTreeSet<(String, String)>,
    /// The origin whose certificate each web view last refused.
    refused: BTreeMap<u64, String>,
    /// Every web view this shell was told to let a certificate through for,
    /// in order.
    pub trust_asked: Vec<u64>,
    /// This shell keeps no certificate it refuses: it says so with the
    /// failure, and is told nothing it can act on.
    pub holds_no_certificate: bool,
    /// The files this shell was asked to open with the system (030), in
    /// order: the name and the kind. It opens nothing.
    pub opened_files: Vec<(String, String)>,
    /// Nothing on this device opens a file: `OpenFile` is answered with
    /// "system.no_handler".
    pub nothing_opens_files: bool,
    /// Files that are no longer where they were saved: `OpenFile` for one of
    /// these names is answered with "download.missing".
    pub missing_files: Vec<String>,
}

impl FakeEngine {
    pub(crate) fn new(engines: Vec<EngineKind>) -> FakeEngine {
        FakeEngine {
            pages: BTreeMap::new(),
            web_views: BTreeMap::new(),
            engines,
            own_web_views: 0,
            opened_outside: Vec::new(),
            nothing_opens_outside: false,
            finds: Vec::new(),
            zooms: Vec::new(),
            download_starts: Vec::new(),
            download_cancels: Vec::new(),
            downloads_not_started: Vec::new(),
            permission_answers: Vec::new(),
            dialog_answers: Vec::new(),
            sign_in_answers: Vec::new(),
            trusted: BTreeSet::new(),
            refused: BTreeMap::new(),
            trust_asked: Vec::new(),
            holds_no_certificate: false,
            opened_files: Vec::new(),
            nothing_opens_files: false,
            missing_files: Vec::new(),
        }
    }

    /// The report of what a find found (010): whether the text is there,
    /// and for an engine that counts, how many matches and which is shown.
    #[must_use]
    pub fn find_result(id: u64, found: bool, counted: Option<(u32, u32)>) -> EngineEvent {
        let (matches, current) = counted.unwrap_or_default();
        event(
            id,
            engine_event::Kind::FindResult(FindResult {
                found,
                counted: counted.is_some(),
                matches,
                current,
            }),
        )
    }

    /// The report of a page trying to go to an address that is not a
    /// browser's, which the shell refused in the web view (007).
    #[must_use]
    pub fn outside_link(id: u64, url: &str, user_gesture: bool) -> EngineEvent {
        event(
            id,
            engine_event::Kind::OutsideLinkRequested(OutsideLinkRequested {
                url: url.to_owned(),
                user_gesture,
            }),
        )
    }

    /// A page in the web view `asking` asks for a new window (006). As a
    /// real engine demands, the shell makes the web view at once, in the
    /// asking web view's login set and under an id of its own. The new web
    /// view's id, and the report of it.
    pub fn ask_for_window(
        &mut self,
        asking: u64,
        url: &str,
        user_gesture: bool,
    ) -> (u64, EngineEvent) {
        self.own_web_views += 1;
        let new = (1 << 63) | self.own_web_views;
        let (engine, login_set, ephemeral) = self
            .web_views
            .get(&asking)
            .map_or((EngineKind::Unspecified, String::new(), false), |asker| {
                (asker.engine, asker.login_set.clone(), asker.ephemeral)
            });
        self.web_views.insert(
            new,
            FakeWebView {
                engine,
                login_set,
                ephemeral,
                history: Vec::new(),
                index: 0,
            },
        );
        let report = event(
            asking,
            engine_event::Kind::NewWindowRequested(NewWindowRequested {
                new_web_view_id: new,
                url: url.to_owned(),
                user_gesture,
            }),
        );
        (new, report)
    }

    /// A web view goes to an address by itself, as a window a page opened
    /// does, or a page that follows a link: nobody asked the shell to load
    /// it. What the engine then reports.
    pub fn goes_to(&mut self, id: u64, url: &str) -> Vec<EngineEvent> {
        self.load(id, url)
    }

    /// The report of a page asking for its own window to be closed.
    #[must_use]
    pub fn close_requested(id: u64) -> EngineEvent {
        event(id, engine_event::Kind::CloseRequested(CloseRequested {}))
    }

    /// The report of a page's title.
    #[must_use]
    pub fn title(id: u64, title: &str) -> EngineEvent {
        event(
            id,
            engine_event::Kind::TitleChanged(TitleChanged {
                title: title.to_owned(),
            }),
        )
    }

    /// Serves a page with a title at an address.
    pub fn page(&mut self, url: &str, title: &str) {
        self.pages.insert(
            url.to_owned(),
            FakePage {
                title: title.to_owned(),
                ..FakePage::default()
            },
        );
    }

    /// Makes an address redirect to another.
    pub fn redirect(&mut self, from: &str, to: &str) {
        self.pages.insert(
            from.to_owned(),
            FakePage {
                redirect: Some(to.to_owned()),
                ..FakePage::default()
            },
        );
    }

    /// A page whose server's certificate is not trusted (026): it fails with
    /// `code`, the engine holding the certificate, until the engine is told
    /// to let it through for the login set of the web view that asks; then
    /// it loads, with `title`.
    pub fn untrusted(&mut self, url: &str, title: &str, code: &str) {
        self.pages.insert(
            url.to_owned(),
            FakePage {
                title: title.to_owned(),
                certificate: Some(code.to_owned()),
                ..FakePage::default()
            },
        );
    }

    /// This shell no longer holds what its web views refused: told to let a
    /// certificate through, it says it has none ("certificate.unknown").
    pub fn forgets_what_it_refused(&mut self) {
        self.refused.clear();
    }

    /// The server at `url` has another certificate now: what was let through
    /// for it is let through no more (026 §3: one certificate, and no other).
    pub fn certificate_changes(&mut self, url: &str) {
        if let Some(origin) = origin_of(url) {
            self.trusted.retain(|(_, trusted)| *trusted != origin);
        }
    }

    /// Makes an address fail to load with a code.
    pub fn fail(&mut self, url: &str, code: &str) {
        self.pages.insert(
            url.to_owned(),
            FakePage {
                fail: Some(code.to_owned()),
                ..FakePage::default()
            },
        );
    }

    /// Answers a host request as a shell would, and says what the engine
    /// then reports.
    pub(crate) fn answer(
        &mut self,
        request: host_request::Kind,
    ) -> (host_response::Kind, Vec<EngineEvent>) {
        use host_request::Kind;
        let ack = host_response::Kind::Ack(Ack {});
        match request {
            Kind::Ping(ping) => (
                host_response::Kind::Pong(HostPong {
                    payload: ping.payload,
                }),
                Vec::new(),
            ),
            Kind::WebViewCreate(create) => {
                if !self.engines.contains(&create.engine()) {
                    return (error("engine.unavailable"), Vec::new());
                }
                let engine = create.engine();
                let login_set = create.login_set.unwrap_or_default();
                self.web_views.insert(
                    create.web_view_id,
                    FakeWebView {
                        engine,
                        login_set: login_set.id,
                        ephemeral: login_set.ephemeral,
                        history: Vec::new(),
                        index: 0,
                    },
                );
                (ack, Vec::new())
            }
            Kind::WebViewLoad(load) => {
                if !self.web_views.contains_key(&load.web_view_id) {
                    return (error("webview.unknown"), Vec::new());
                }
                (ack, self.load(load.web_view_id, &load.url))
            }
            Kind::WebViewTrustCertificate(trust) => {
                let Some(view) = self.web_views.get(&trust.web_view_id) else {
                    return (error("webview.unknown"), Vec::new());
                };
                self.trust_asked.push(trust.web_view_id);
                // What this web view last refused, for its login set: that
                // origin's certificate is let through from now.
                let Some(origin) = self.refused.get(&trust.web_view_id).cloned() else {
                    return (error("certificate.unknown"), Vec::new());
                };
                self.trusted.insert((view.login_set.clone(), origin));
                (ack, Vec::new())
            }
            Kind::WebViewCommand(command) => {
                if !self.web_views.contains_key(&command.web_view_id) {
                    return (error("webview.unknown"), Vec::new());
                }
                let events = self.command(command.web_view_id, command.command());
                (ack, events)
            }
            Kind::WebViewDestroy(destroy) => {
                self.web_views.remove(&destroy.web_view_id);
                (ack, Vec::new())
            }
            Kind::WebViewFind(find) => {
                if !self.web_views.contains_key(&find.web_view_id) {
                    return (error("webview.unknown"), Vec::new());
                }
                self.finds
                    .push((find.web_view_id, find.text, find.backwards));
                (ack, Vec::new())
            }
            Kind::WebViewSetZoom(zoom) => {
                if !self.web_views.contains_key(&zoom.web_view_id) {
                    return (error("webview.unknown"), Vec::new());
                }
                self.zooms.push((zoom.web_view_id, zoom.percent));
                (ack, Vec::new())
            }
            Kind::DownloadStart(start) => {
                if self.downloads_not_started.contains(&start.download_id) {
                    return (error("download.unknown"), Vec::new());
                }
                self.download_starts
                    .push((start.download_id, start.file_name));
                (ack, Vec::new())
            }
            Kind::DownloadCancel(cancel) => {
                self.download_cancels.push(cancel.download_id);
                (ack, Vec::new())
            }
            Kind::DialogAnswer(answer) => {
                self.dialog_answers
                    .push((answer.request_id, answer.accepted, answer.text));
                (ack, Vec::new())
            }
            Kind::SignInAnswer(answer) => {
                let typed = answer.accepted.then_some((answer.name, answer.password));
                self.sign_in_answers.push((answer.request_id, typed));
                (ack, Vec::new())
            }
            Kind::PermissionAnswer(answer) => {
                self.permission_answers
                    .push((answer.request_id, answer.granted));
                (ack, Vec::new())
            }
            Kind::OpenOutside(open) => {
                if self.nothing_opens_outside {
                    return (error("system.no_handler"), Vec::new());
                }
                self.opened_outside.push(open.url);
                (ack, Vec::new())
            }
            Kind::OpenFile(open) => {
                if self.missing_files.contains(&open.file_name) {
                    return (error("download.missing"), Vec::new());
                }
                if self.nothing_opens_files {
                    return (error("system.no_handler"), Vec::new());
                }
                self.opened_files.push((open.file_name, open.mime_type));
                (ack, Vec::new())
            }
        }
    }

    /// The events of loading an address: redirects followed, then a commit
    /// or a failure.
    fn load(&mut self, id: u64, url: &str) -> Vec<EngineEvent> {
        let mut events = vec![event(
            id,
            engine_event::Kind::NavigationStarted(NavigationStarted {
                url: url.to_owned(),
            }),
        )];
        let mut at = url.to_owned();
        for _ in 0..8 {
            match self.pages.get(&at).and_then(|page| page.redirect.clone()) {
                Some(next) => at = next,
                None => break,
            }
        }
        let page = self.pages.get(&at).cloned();
        // A certificate that is not trusted (026): refused, and held, unless
        // this web view's login set was told to let it through.
        let login_set = self
            .web_views
            .get(&id)
            .map(|view| view.login_set.clone())
            .unwrap_or_default();
        let untrusted = page
            .as_ref()
            .and_then(|page| page.certificate.clone())
            .zip(origin_of(&at))
            .filter(|(_, origin)| !self.trusted.contains(&(login_set.clone(), origin.clone())));
        if let Some((code, origin)) = untrusted {
            let held = !self.holds_no_certificate;
            if held {
                self.refused.insert(id, origin);
            }
            events.push(event(
                id,
                engine_event::Kind::NavigationFailed(NavigationFailed {
                    code,
                    url: at,
                    refused: false,
                    certificate_held: held,
                }),
            ));
            return events;
        }
        let failure = match &page {
            None => Some("net.dns".to_owned()),
            Some(page) => page.fail.clone(),
        };
        if let Some(code) = failure {
            events.push(event(
                id,
                engine_event::Kind::NavigationFailed(NavigationFailed {
                    code,
                    url: at,
                    refused: false,
                    certificate_held: false,
                }),
            ));
            return events;
        }
        if let Some(view) = self.web_views.get_mut(&id) {
            view.history.truncate(if view.history.is_empty() {
                0
            } else {
                view.index + 1
            });
            view.history.push(at.clone());
            view.index = view.history.len() - 1;
        }
        events.extend(self.arrive(id, &at, page.map(|page| page.title).unwrap_or_default()));
        events
    }

    /// The events of a page taking over the web view.
    fn arrive(&self, id: u64, url: &str, title: String) -> Vec<EngineEvent> {
        let (back, forward) = self.web_views.get(&id).map_or((false, false), |view| {
            (view.index > 0, view.index + 1 < view.history.len())
        });
        vec![
            event(
                id,
                engine_event::Kind::NavigationCommitted(NavigationCommitted {
                    url: url.to_owned(),
                }),
            ),
            event(
                id,
                engine_event::Kind::HistoryChanged(HistoryChanged {
                    can_go_back: back,
                    can_go_forward: forward,
                }),
            ),
            event(id, engine_event::Kind::TitleChanged(TitleChanged { title })),
            event(
                id,
                engine_event::Kind::ProgressChanged(ProgressChanged { fraction: 1.0 }),
            ),
            event(
                id,
                engine_event::Kind::NavigationFinished(NavigationFinished {}),
            ),
        ]
    }

    fn command(&mut self, id: u64, command: NavigationCommand) -> Vec<EngineEvent> {
        let Some(view) = self.web_views.get_mut(&id) else {
            return Vec::new();
        };
        match command {
            NavigationCommand::Back if view.index > 0 => view.index -= 1,
            NavigationCommand::Forward if view.index + 1 < view.history.len() => view.index += 1,
            NavigationCommand::Reload if !view.history.is_empty() => {}
            _ => return Vec::new(),
        }
        let url = view.history[view.index].clone();
        let title = self
            .pages
            .get(&url)
            .map(|page| page.title.clone())
            .unwrap_or_default();
        let mut events = vec![event(
            id,
            engine_event::Kind::NavigationStarted(NavigationStarted { url: url.clone() }),
        )];
        events.extend(self.arrive(id, &url, title));
        events
    }

    /// A web view's page asks to use something of the device's (014): the
    /// shell names the request `request`, and says who asks.
    #[must_use]
    pub fn permission_requested(
        id: u64,
        request: u64,
        origin: &str,
        kinds: &[PermissionKind],
    ) -> EngineEvent {
        event(
            id,
            engine_event::Kind::PermissionRequested(PermissionRequested {
                request_id: request,
                origin: origin.to_owned(),
                kinds: kinds.iter().map(|kind| *kind as i32).collect(),
            }),
        )
    }

    /// A web view's page says something, or asks (016): the shell names it
    /// `request`, and says who called.
    #[must_use]
    pub fn dialog_requested(
        id: u64,
        request: u64,
        origin: &str,
        kind: DialogKind,
        message: &str,
        default_text: &str,
    ) -> EngineEvent {
        event(
            id,
            engine_event::Kind::DialogRequested(DialogRequested {
                request_id: request,
                origin: origin.to_owned(),
                kind: kind as i32,
                message: message.to_owned(),
                default_text: default_text.to_owned(),
            }),
        )
    }

    /// The person asked for a link of a web view's page in a new tab (018),
    /// as the shell says it.
    #[must_use]
    pub fn link_open_requested(
        id: u64,
        url: &str,
        background: bool,
        user_gesture: bool,
    ) -> EngineEvent {
        event(
            id,
            engine_event::Kind::LinkOpenRequested(LinkOpenRequested {
                url: url.to_owned(),
                background,
                user_gesture,
            }),
        )
    }

    /// The person asked to search for words of a web view's page (023), as
    /// the shell says it.
    #[must_use]
    pub fn search_requested(id: u64, text: &str, user_gesture: bool) -> EngineEvent {
        event(
            id,
            engine_event::Kind::SearchRequested(SearchRequested {
                text: text.to_owned(),
                user_gesture,
            }),
        )
    }

    /// A server asks who is there in a web view (017), as the shell says
    /// it.
    #[must_use]
    pub fn sign_in_requested(id: u64, asked: SignInRequested) -> EngineEvent {
        event(id, engine_event::Kind::SignInRequested(asked))
    }

    /// A web view's page no longer waits for an answer to a request.
    #[must_use]
    pub fn permission_withdrawn(id: u64, request: u64) -> EngineEvent {
        event(
            id,
            engine_event::Kind::PermissionWithdrawn(PermissionWithdrawn {
                request_id: request,
            }),
        )
    }

    /// A web view's page asked for a download (013), which the shell names
    /// `download`.
    #[must_use]
    pub fn download_requested(
        id: u64,
        download: u64,
        url: &str,
        suggested: &str,
        mime_type: &str,
        bytes: u64,
    ) -> EngineEvent {
        event(
            id,
            engine_event::Kind::DownloadRequested(DownloadRequested {
                download_id: download,
                url: url.to_owned(),
                suggested_filename: suggested.to_owned(),
                mime_type: mime_type.to_owned(),
                expected_bytes: bytes,
                user_asked: false,
            }),
        )
    }

    /// The person chose in a menu to save a file of a web view's page
    /// (022): a download nobody's page asked for.
    #[must_use]
    pub fn download_chosen(id: u64, download: u64, url: &str, suggested: &str) -> EngineEvent {
        event(
            id,
            engine_event::Kind::DownloadRequested(DownloadRequested {
                download_id: download,
                url: url.to_owned(),
                suggested_filename: suggested.to_owned(),
                mime_type: String::new(),
                expected_bytes: 0,
                user_asked: true,
            }),
        )
    }

    /// A download has got so far.
    #[must_use]
    pub fn download_progress(download: u64, received: u64, expected: u64) -> EngineEvent {
        // A download's later reports are about the download: no web view.
        event(
            0,
            engine_event::Kind::DownloadProgress(DownloadProgress {
                download_id: download,
                received_bytes: received,
                expected_bytes: expected,
            }),
        )
    }

    /// A download's file is whole, under `name`.
    #[must_use]
    pub fn download_finished(download: u64, name: &str) -> EngineEvent {
        event(
            0,
            engine_event::Kind::DownloadFinished(DownloadFinished {
                download_id: download,
                file_name: name.to_owned(),
            }),
        )
    }

    /// A download ended without its file.
    #[must_use]
    pub fn download_failed(download: u64, code: &str) -> EngineEvent {
        event(
            0,
            engine_event::Kind::DownloadFailed(DownloadFailed {
                download_id: download,
                code: code.to_owned(),
            }),
        )
    }

    /// The report of a page changing its address without a load.
    #[must_use]
    pub fn address_changed(id: u64, url: &str) -> EngineEvent {
        event(
            id,
            engine_event::Kind::AddressChanged(AddressChanged {
                url: url.to_owned(),
            }),
        )
    }

    /// The report of a page starting a load by itself.
    #[must_use]
    pub fn navigation_started(id: u64, url: &str) -> EngineEvent {
        event(
            id,
            engine_event::Kind::NavigationStarted(NavigationStarted {
                url: url.to_owned(),
            }),
        )
    }

    /// The report of a page taking over the web view.
    #[must_use]
    pub fn navigation_committed(id: u64, url: &str) -> EngineEvent {
        event(
            id,
            engine_event::Kind::NavigationCommitted(NavigationCommitted {
                url: url.to_owned(),
            }),
        )
    }

    /// The report of a load finishing.
    #[must_use]
    pub fn navigation_finished(id: u64) -> EngineEvent {
        event(
            id,
            engine_event::Kind::NavigationFinished(NavigationFinished {}),
        )
    }

    /// The report of a load failing.
    #[must_use]
    pub fn navigation_failed(id: u64, code: &str, url: &str) -> EngineEvent {
        event(
            id,
            engine_event::Kind::NavigationFailed(NavigationFailed {
                code: code.to_owned(),
                url: url.to_owned(),
                refused: false,
                certificate_held: false,
            }),
        )
    }

    /// The report of a navigation the shell refused on the core's verdict,
    /// with nothing else loading in the web view.
    #[must_use]
    pub fn navigation_refused(id: u64, url: &str) -> EngineEvent {
        event(
            id,
            engine_event::Kind::NavigationFailed(NavigationFailed {
                code: "net.cancelled".to_owned(),
                url: url.to_owned(),
                refused: true,
                certificate_held: false,
            }),
        )
    }

    /// The report of a web view's process dying.
    #[must_use]
    pub fn renderer_gone(id: u64) -> EngineEvent {
        event(id, engine_event::Kind::RendererGone(RendererGone {}))
    }

    /// The report of a load's progress.
    #[must_use]
    pub fn progress(id: u64, fraction: f64) -> EngineEvent {
        event(
            id,
            engine_event::Kind::ProgressChanged(ProgressChanged { fraction }),
        )
    }
}

/// The scheme, host and port of an address, as far as its path: whose
/// certificate a connection to it presents.
fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    (!authority.is_empty()).then(|| format!("{scheme}://{authority}"))
}

fn event(id: u64, kind: engine_event::Kind) -> EngineEvent {
    EngineEvent {
        web_view_id: id,
        kind: Some(kind),
    }
}

fn error(code: &str) -> host_response::Kind {
    host_response::Kind::Error(Error {
        code: code.to_owned(),
        field: String::new(),
    })
}
