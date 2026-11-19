//! The models: windows and tabs, and what each one shows. Plain state with
//! no I/O: [`crate::App`] changes it and asks the shell for the rest.

use std::collections::{BTreeMap, BTreeSet};

use wt_core::{Address, IdMint, LoginSetId, Security, TabId, WebViewId, WindowId};
use wt_proto::v1::{
    AddressView, EngineKind, LoadState, TabSummary, TabView, WindowSnapshot, address_view,
    engine_event,
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
    /// The tab whose page opened this one, while that tab is there to go
    /// back to (006).
    opener: Option<TabId>,
    /// A page opened this tab, and so a page may close it.
    opened_by_page: bool,
    /// A page has owned the web view at some time.
    had_page: bool,
    /// The click that opened this tab's window has been used to hand an
    /// address to the system (007): it is good for one.
    click_spent: bool,
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
            opener: None,
            opened_by_page: false,
            had_page: false,
            click_spent: false,
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
                self.had_page = true;
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
            // About windows, not about this tab's page: `State::apply` has
            // dealt with them. And an address for the system changes nothing
            // of the page that asked for it: the app has dealt with that.
            Kind::NewWindowRequested(_)
            | Kind::CloseRequested(_)
            | Kind::OutsideLinkRequested(_) => {}
        }
    }

    /// The address the tab shows: where the user sent it, until a page
    /// commits; otherwise the page that owns the web view.
    fn shown(&self) -> Option<&Address> {
        self.pending.as_ref().or(self.committed.as_ref())
    }

    /// The tab as a strip of tabs shows it.
    fn summary(&self, id: TabId) -> TabSummary {
        TabSummary {
            tab_id: id.get(),
            title: self.title.clone(),
            host: self
                .shown()
                .and_then(Address::host)
                .unwrap_or_default()
                .to_owned(),
            loading: self.load == Load::Loading,
        }
    }

    fn view(&self, id: TabId) -> TabView {
        let shown = self.shown();
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

/// A window: its tabs in the order they are shown, and the one in front. It
/// always has a tab (005, T1).
#[derive(Debug)]
struct Window {
    tabs: Vec<TabId>,
    front: TabId,
}

/// Everything the app knows.
#[derive(Debug)]
pub(crate) struct State {
    space: Space,
    engine: EngineKind,
    /// One mint for windows, tabs and web views, so an id of one kind never
    /// equals an id of another.
    ids: IdMint,
    windows: BTreeMap<WindowId, Window>,
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
        self.windows.insert(
            window,
            Window {
                tabs: vec![tab],
                front: tab,
            },
        );
        self.tabs
            .insert(tab, Tab::new(window, self.engine, self.space.login_set));
        self.dirty.insert(window);
        (window, tab)
    }

    /// Closes a window. The web views to destroy, or `None` when there is no
    /// such window.
    pub(crate) fn close_window(&mut self, window: WindowId) -> Option<Vec<WebViewId>> {
        let closing = self.windows.remove(&window)?;
        let mut web_views = Vec::new();
        for tab in closing.tabs {
            if let Some(web_view) = self.tabs.remove(&tab).and_then(|tab| tab.web_view) {
                self.web_views.remove(&web_view);
                web_views.push(web_view);
            }
        }
        self.dirty.remove(&window);
        self.closed.push(window);
        Some(web_views)
    }

    /// Opens an empty tab in a window: after `after` when that is one of the
    /// window's tabs, at the end otherwise; in front unless `background`.
    /// `None` when there is no such window.
    pub(crate) fn open_tab(
        &mut self,
        window_id: WindowId,
        after: Option<TabId>,
        background: bool,
    ) -> Option<TabId> {
        let window = self.windows.get_mut(&window_id)?;
        let tab = TabId(self.ids.mint());
        let place = after
            .and_then(|after| window.tabs.iter().position(|id| *id == after))
            .map_or(window.tabs.len(), |index| index + 1);
        window.tabs.insert(place, tab);
        if !background {
            window.front = tab;
        }
        self.tabs
            .insert(tab, Tab::new(window_id, self.engine, self.space.login_set));
        self.dirty.insert(window_id);
        Some(tab)
    }

    /// Closes a tab. Its web view to destroy, if it had one; or `None` when
    /// there is no such tab.
    ///
    /// When the tab was in front, the one after it comes forward, or the one
    /// before it when it was the last. A window always has a tab: an empty
    /// one takes the place of its only tab.
    pub(crate) fn close_tab(&mut self, tab_id: TabId) -> Option<Option<WebViewId>> {
        let tab = self.tabs.remove(&tab_id)?;
        if let Some(web_view) = tab.web_view {
            self.web_views.remove(&web_view);
        }
        if let Some(window) = self.windows.get_mut(&tab.window) {
            let index = window.tabs.iter().position(|id| *id == tab_id);
            window.tabs.retain(|id| *id != tab_id);
            if window.tabs.is_empty() {
                let fresh = TabId(self.ids.mint());
                self.tabs.insert(
                    fresh,
                    Tab::new(tab.window, self.engine, self.space.login_set),
                );
                window.tabs.push(fresh);
                window.front = fresh;
            } else if window.front == tab_id {
                // Back to the tab whose page opened it, when that is still
                // here. Otherwise what was after it, which is now where it
                // was; past the end, the last.
                let opener = tab.opener.filter(|opener| window.tabs.contains(opener));
                let next = index.unwrap_or(0).min(window.tabs.len() - 1);
                if let Some(next) = opener.or_else(|| window.tabs.get(next).copied()) {
                    window.front = next;
                }
            }
            self.dirty.insert(tab.window);
        }
        Some(tab.web_view)
    }

    /// Brings a tab in front in its window. Whether there is such a tab.
    pub(crate) fn activate_tab(&mut self, tab_id: TabId) -> bool {
        let Some(tab) = self.tabs.get(&tab_id) else {
            return false;
        };
        if let Some(window) = self.windows.get_mut(&tab.window)
            && window.front != tab_id
        {
            window.front = tab_id;
            self.dirty.insert(tab.window);
        }
        true
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

    /// A page asked for a new window, and the shell made `new` for it (006).
    /// With a gesture, from a tab that is here, the web view is adopted: a
    /// new tab has it, after the tab that asked and in front. Otherwise it
    /// is refused, and this returns it to be destroyed. An id the shell had
    /// no right to choose is not acted on at all.
    fn new_window(
        &mut self,
        asking: WebViewId,
        new: WebViewId,
        user_gesture: bool,
    ) -> Option<WebViewId> {
        if !new.is_the_shells() || self.web_views.contains_key(&new) {
            return None;
        }
        let refused = Some(new);
        if !user_gesture {
            return refused;
        }
        let Some(opener_id) = self.web_views.get(&asking).copied() else {
            return refused;
        };
        let Some(opener) = self.tabs.get(&opener_id) else {
            return refused;
        };
        let window_id = opener.window;
        let mut tab = Tab::new(window_id, opener.engine, opener.login_set);
        tab.web_view = Some(new);
        tab.opener = Some(opener_id);
        tab.opened_by_page = true;
        let Some(window) = self.windows.get_mut(&window_id) else {
            return refused;
        };
        let tab_id = TabId(self.ids.mint());
        let place = window
            .tabs
            .iter()
            .position(|id| *id == opener_id)
            .map_or(window.tabs.len(), |index| index + 1);
        window.tabs.insert(place, tab_id);
        window.front = tab_id;
        self.tabs.insert(tab_id, tab);
        self.web_views.insert(new, tab_id);
        self.dirty.insert(window_id);
        None
    }

    /// A page asked for its own window to be closed. A tab a page opened is
    /// closed, and its web view is returned to be destroyed; a tab the user
    /// opened is not a page's to close.
    fn close_requested(&mut self, web_view: WebViewId) -> Option<WebViewId> {
        let tab_id = *self.web_views.get(&web_view)?;
        if !self.tabs.get(&tab_id)?.opened_by_page {
            return None;
        }
        self.close_tab(tab_id).flatten()
    }

    /// Whether a web view is a window a page opened that has nothing to show
    /// and nowhere to go: no page has ever owned it, and the user has sent
    /// its tab nowhere.
    fn has_nothing_to_show(&self, web_view: WebViewId) -> bool {
        self.web_views
            .get(&web_view)
            .and_then(|tab_id| self.tabs.get(tab_id))
            .is_some_and(|tab| tab.opened_by_page && !tab.had_page && tab.pending.is_none())
    }

    /// Whether an address a web view's page asked for may be handed to the
    /// system (007), as far as who asked goes: the web view must be a tab's,
    /// and a person must have asked. A window a page opened exists only
    /// because somebody clicked, so the first address of one that has had no
    /// page counts as asked for, once.
    pub(crate) fn may_open_outside(&mut self, web_view: WebViewId, user_gesture: bool) -> bool {
        let carried = self.has_nothing_to_show(web_view);
        let Some(tab) = self
            .web_views
            .get(&web_view)
            .and_then(|tab_id| self.tabs.get_mut(tab_id))
        else {
            return false;
        };
        if user_gesture {
            return true;
        }
        if carried && !tab.click_spent {
            tab.click_spent = true;
            return true;
        }
        false
    }

    /// A shell's report about a web view. One the model no longer has (its
    /// window closed while the report was on its way) is ignored. Returns a
    /// web view the shell must now be told to destroy, when the report
    /// leaves one that no tab has.
    pub(crate) fn apply(
        &mut self,
        web_view: WebViewId,
        event: engine_event::Kind,
    ) -> Option<WebViewId> {
        match event {
            engine_event::Kind::NewWindowRequested(asked) => {
                return self.new_window(
                    web_view,
                    WebViewId(asked.new_web_view_id),
                    asked.user_gesture,
                );
            }
            engine_event::Kind::CloseRequested(_) => return self.close_requested(web_view),
            // The only place a window a page opened was going was refused,
            // and nothing else is loading in it: its tab is closed, as if
            // the window had not been opened. (A load that was stopped or
            // replaced ends as cancelled too, and closes nothing.)
            engine_event::Kind::NavigationFailed(ref failed)
                if failed.refused && self.has_nothing_to_show(web_view) =>
            {
                let tab_id = *self.web_views.get(&web_view)?;
                return self.close_tab(tab_id).flatten();
            }
            _ => {}
        }
        // A web view no tab has: nothing changes, and nothing is destroyed.
        let tab = self
            .web_views
            .get(&web_view)
            .and_then(|tab_id| self.tabs.get_mut(tab_id))?;
        let before = tab.clone();
        tab.apply(event);
        if *tab != before {
            self.dirty.insert(tab.window);
        }
        None
    }

    pub(crate) fn window_snapshot(&self, window_id: WindowId) -> Option<WindowSnapshot> {
        let window = self.windows.get(&window_id)?;
        let front = self.tabs.get(&window.front)?;
        Some(WindowSnapshot {
            window_id: window_id.get(),
            tab: Some(front.view(window.front)),
            tabs: window
                .tabs
                .iter()
                .filter_map(|id| self.tabs.get(id).map(|tab| tab.summary(*id)))
                .collect(),
        })
    }

    pub(crate) fn take_dirty(&mut self) -> BTreeSet<WindowId> {
        std::mem::take(&mut self.dirty)
    }

    pub(crate) fn take_closed(&mut self) -> Vec<WindowId> {
        std::mem::take(&mut self.closed)
    }
}
