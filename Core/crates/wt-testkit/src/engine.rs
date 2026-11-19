//! A scripted web engine: the pages it "serves" are a table, and loading one
//! produces the events a real engine reports, in the order 001 §5 asks every
//! adapter to report them.

use std::collections::BTreeMap;

use wt_proto::v1::{
    Ack, AddressChanged, CloseRequested, EngineEvent, EngineKind, Error, HistoryChanged, HostPong,
    NavigationCommand, NavigationCommitted, NavigationFailed, NavigationFinished,
    NavigationStarted, NewWindowRequested, ProgressChanged, RendererGone, TitleChanged,
    engine_event, host_request, host_response,
};

/// A page of the scripted web.
#[derive(Clone, Debug, Default)]
pub struct FakePage {
    pub title: String,
    /// The server sends the browser on to this address.
    pub redirect: Option<String>,
    /// The load fails with this code instead.
    pub fail: Option<String>,
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
}

impl FakeEngine {
    pub(crate) fn new(engines: Vec<EngineKind>) -> FakeEngine {
        FakeEngine {
            pages: BTreeMap::new(),
            web_views: BTreeMap::new(),
            engines,
            own_web_views: 0,
        }
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
