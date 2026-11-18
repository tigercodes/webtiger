//! The models: windows and tabs, and what each one shows. Plain state with
//! no I/O: [`crate::App`] changes it and asks the shell for the rest.

use std::collections::{BTreeMap, BTreeSet};

use wt_core::{Address, IdMint, LoginSetId, Security, TabId, WebViewId, WindowId};
use wt_proto::v1::{
    AddressView, EngineKind, LoadState, TabView, WindowSnapshot, address_view, engine_event,
};
use wt_store::Space;

use crate::error::is_code;

/// Whether a tab is loading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Load {
    Idle,
    Loading,
    Failed,
}

/// A tab: a browsing context, with or without a web view behind it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tab {
    pub(crate) window: WindowId,
    pub(crate) web_view: Option<WebViewId>,
    /// The page that owns the web view.
    committed: Option<Address>,
    /// Where the *user* sent the tab, until a page commits or the load ends.
    /// A load a page starts by itself never sets this, so a page cannot put
    /// an address in the field that it has not yet been allowed to become.
    pending: Option<Address>,
    title: String,
    load: Load,
    progress: f64,
    can_go_back: bool,
    can_go_forward: bool,
    /// With `Load::Failed`: the code.
    error: String,
    engine: EngineKind,
    login_set: LoginSetId,
}

impl Tab {
    fn new(window: WindowId, engine: EngineKind, login_set: LoginSetId) -> Tab {
        Tab {
            window,
            web_view: None,
            committed: None,
            pending: None,
            title: String::new(),
            load: Load::Idle,
            progress: 0.0,
            can_go_back: false,
            can_go_forward: false,
            error: String::new(),
            engine,
            login_set,
        }
    }

    /// The user sent the tab to `address`.
    fn begin_user_load(&mut self, address: Address) {
        self.pending = Some(address);
        self.load = Load::Loading;
        self.progress = 0.0;
        self.error.clear();
    }

    /// The tab's load failed with a code.
    fn fail(&mut self, code: &str) {
        self.load = Load::Failed;
        self.progress = 0.0;
        self.error = if is_code(code) {
            code.to_owned()
        } else {
            "net.failed".to_owned()
        };
    }

    /// What a shell reported about the tab's web view.
    fn apply(&mut self, event: engine_event::Kind) {
        use engine_event::Kind;
        match event {
            Kind::NavigationStarted(_) => {
                self.load = Load::Loading;
                self.progress = 0.0;
                self.error.clear();
            }
            Kind::NavigationCommitted(committed) => {
                // An address the engine reports but the parser refuses is
                // shown as no address at all rather than as the old one.
                self.committed = Address::parse(&committed.url).ok();
                self.pending = None;
                self.title.clear();
            }
            Kind::NavigationFinished(_) => {
                // A load that failed stays failed. Some engines report a
                // finish after the failure, for the error page they show
                // themselves; only a new load clears a failure.
                if self.load != Load::Failed {
                    self.load = Load::Idle;
                    self.progress = 1.0;
                }
            }
            Kind::NavigationFailed(failed) => {
                if failed.code == "net.cancelled" {
                    // The user stopped it, or another load replaced it: the
                    // field goes back to the page that is still on screen.
                    self.load = Load::Idle;
                    self.progress = 0.0;
                    self.pending = None;
                } else {
                    self.fail(&failed.code);
                }
            }
            Kind::AddressChanged(changed) => {
                if let Ok(address) = Address::parse(&changed.url) {
                    self.committed = Some(address);
                }
            }
            Kind::TitleChanged(changed) => self.title = changed.title,
            Kind::ProgressChanged(changed) => {
                if changed.fraction.is_finite() {
                    self.progress = changed.fraction.clamp(0.0, 1.0);
                }
            }
            Kind::HistoryChanged(changed) => {
                self.can_go_back = changed.can_go_back;
                self.can_go_forward = changed.can_go_forward;
            }
            Kind::RendererGone(_) => self.fail("page.renderer_gone"),
        }
    }

    fn view(&self, id: TabId) -> TabView {
        let shown = self.pending.as_ref().or(self.committed.as_ref());
        TabView {
            tab_id: id.get(),
            web_view_id: self.web_view.map_or(0, WebViewId::get),
            address: Some(shown.map_or_else(AddressView::default, address_view)),
            title: self.title.clone(),
            load_state: match self.load {
                Load::Idle => LoadState::Idle,
                Load::Loading => LoadState::Loading,
                Load::Failed => LoadState::Failed,
            } as i32,
            progress: self.progress,
            can_go_back: self.can_go_back,
            can_go_forward: self.can_go_forward,
            error_code: self.error.clone(),
            engine: self.engine as i32,
            login_set_id: self.login_set.to_string(),
        }
    }
}

fn address_view(address: &Address) -> AddressView {
    AddressView {
        url: address.as_str().to_owned(),
        display: address.display(),
        host: address.host().unwrap_or_default().to_owned(),
        registrable_domain: address.registrable_domain().unwrap_or_default().to_owned(),
        security: match address.security() {
            Security::Secure => address_view::Security::Secure,
            Security::Insecure => address_view::Security::Insecure,
            Security::Local => address_view::Security::Local,
        } as i32,
    }
}

/// A web view the model has just asked for, and whether the shell has to
/// create it first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LoadTarget {
    pub(crate) web_view: WebViewId,
    pub(crate) create: bool,
    pub(crate) engine: EngineKind,
    pub(crate) login_set: LoginSetId,
}

/// Everything the app knows.
#[derive(Debug)]
pub(crate) struct State {
    space: Space,
    engine: EngineKind,
    /// One mint for windows, tabs and web views, so an id of one kind never
    /// equals an id of another.
    ids: IdMint,
    /// W0: a window has one tab.
    windows: BTreeMap<WindowId, TabId>,
    tabs: BTreeMap<TabId, Tab>,
    web_views: BTreeMap<WebViewId, TabId>,
    /// Windows whose snapshot may have changed since the last publish.
    dirty: BTreeSet<WindowId>,
    /// Windows closed since the last publish.
    closed: Vec<WindowId>,
}

impl State {
    pub(crate) fn new(space: Space, engine: EngineKind) -> State {
        State {
            space,
            engine,
            ids: IdMint::new(),
            windows: BTreeMap::new(),
            tabs: BTreeMap::new(),
            web_views: BTreeMap::new(),
            dirty: BTreeSet::new(),
            closed: Vec::new(),
        }
    }

    pub(crate) fn open_window(&mut self) -> (WindowId, TabId) {
        let window = WindowId(self.ids.mint());
        let tab = TabId(self.ids.mint());
        self.windows.insert(window, tab);
        self.tabs
            .insert(tab, Tab::new(window, self.engine, self.space.login_set));
        self.dirty.insert(window);
        (window, tab)
    }

    /// Closes a window. The web views to destroy, or `None` when there is no
    /// such window.
    pub(crate) fn close_window(&mut self, window: WindowId) -> Option<Vec<WebViewId>> {
        let tab = self.windows.remove(&window)?;
        let web_view = self.tabs.remove(&tab).and_then(|tab| tab.web_view);
        if let Some(web_view) = web_view {
            self.web_views.remove(&web_view);
        }
        self.dirty.remove(&window);
        self.closed.push(window);
        Some(web_view.into_iter().collect())
    }

    /// The user sent a tab to an address. Says which web view loads it and
    /// whether that web view is new, or `None` when there is no such tab.
    pub(crate) fn begin_user_load(
        &mut self,
        tab_id: TabId,
        address: Address,
    ) -> Option<LoadTarget> {
        let tab = self.tabs.get_mut(&tab_id)?;
        tab.begin_user_load(address);
        let (web_view, create) = match tab.web_view {
            Some(web_view) => (web_view, false),
            None => {
                let web_view = WebViewId(self.ids.mint());
                tab.web_view = Some(web_view);
                self.web_views.insert(web_view, tab_id);
                (web_view, true)
            }
        };
        self.dirty.insert(tab.window);
        Some(LoadTarget {
            web_view,
            create,
            engine: tab.engine,
            login_set: tab.login_set,
        })
    }

    /// The shell could not create `web_view`: the tab has none after all.
    pub(crate) fn web_view_failed(&mut self, web_view: WebViewId, code: &str) {
        let Some(tab_id) = self.web_views.remove(&web_view) else {
            return;
        };
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.web_view = None;
            tab.fail(code);
            self.dirty.insert(tab.window);
        }
    }

    /// The shell could not start a load in `web_view`.
    pub(crate) fn load_failed(&mut self, web_view: WebViewId, code: &str) {
        if let Some(tab) = self
            .web_views
            .get(&web_view)
            .and_then(|tab_id| self.tabs.get_mut(tab_id))
        {
            tab.fail(code);
            self.dirty.insert(tab.window);
        }
    }

    /// The web view of a tab: `Err(false)` for no such tab, `Err(true)` for
    /// a tab that has no web view yet.
    pub(crate) fn web_view_of(&self, tab: TabId) -> Result<WebViewId, bool> {
        self.tabs.get(&tab).ok_or(false)?.web_view.ok_or(true)
    }

    /// A shell's report about a web view. One the model no longer has (its
    /// window closed while the report was on its way) is ignored.
    pub(crate) fn apply(&mut self, web_view: WebViewId, event: engine_event::Kind) {
        let Some(tab) = self
            .web_views
            .get(&web_view)
            .and_then(|tab_id| self.tabs.get_mut(tab_id))
        else {
            return;
        };
        let before = tab.clone();
        tab.apply(event);
        if *tab != before {
            self.dirty.insert(tab.window);
        }
    }

    pub(crate) fn window_snapshot(&self, window: WindowId) -> Option<WindowSnapshot> {
        let tab_id = *self.windows.get(&window)?;
        let tab = self.tabs.get(&tab_id)?;
        Some(WindowSnapshot {
            window_id: window.get(),
            tab: Some(tab.view(tab_id)),
        })
    }

    pub(crate) fn take_dirty(&mut self) -> BTreeSet<WindowId> {
        std::mem::take(&mut self.dirty)
    }

    pub(crate) fn take_closed(&mut self) -> Vec<WindowId> {
        std::mem::take(&mut self.closed)
    }
}
