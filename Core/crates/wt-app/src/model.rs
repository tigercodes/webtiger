//! The models: windows and tabs, and what each one shows. Plain state with
//! no I/O: [`crate::App`] changes it and asks the shell for the rest.

use std::collections::{BTreeMap, BTreeSet};

use wt_core::{
    ACTUAL_SIZE, Address, IdMint, LoginSetId, Security, TabId, WebViewId, WindowId, zoom_in,
    zoom_out,
};
use wt_proto::v1::{
    AddressView, EngineKind, FindView, LoadState, TabSummary, TabView, WindowSnapshot, ZoomView,
    address_view, engine_event,
};
use wt_store::{SavedTab, Space};

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
    /// The page the tab shows, as history has it (008).
    visit: Option<TabVisit>,
    /// The tab came back from the session and has not been loaded since
    /// (009): it has where it was (`pending`) and what it was called, and no
    /// web view. It is loaded when it comes in front.
    asleep: bool,
    /// The title the session keeps for the tab, and the address it is the
    /// title of: the first a page says, so that a page that changes its
    /// title all the time is written once.
    session_title: Option<(String, String)>,
    /// What is being looked for in the tab's page, and what the engine has
    /// said of it (010). `None` while the tab has no find.
    find: Option<FindView>,
    /// The zoom the tab's web view was last told, in per cent (012). A web
    /// view that was never told is at actual size.
    zoom: u32,
}

/// The visit a tab's page is in the store, and what has been written of it.
#[derive(Clone, Debug, PartialEq)]
struct TabVisit {
    id: i64,
    /// The address it was written with: the same address again, straight
    /// after, is the same visit.
    url: String,
    /// The title the store has for it.
    title: String,
    /// The page has said a title since the visit was written, and it has
    /// been written: later changes wait until the tab goes on.
    titled: bool,
    /// The last title the page said that was one. An engine clears the
    /// title as the next page arrives, sometimes before it says the page
    /// has committed: no title is not the title a page was left with.
    latest: String,
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
            visit: None,
            asleep: false,
            session_title: None,
            find: None,
            zoom: ACTUAL_SIZE,
        }
    }

    /// A tab as the session kept it (009): where it was and what it was
    /// called, asleep. `None` when what was kept is no address of the
    /// web's: that is an empty tab.
    fn restored(
        window: WindowId,
        engine: EngineKind,
        login_set: LoginSetId,
        saved: &SavedTab,
    ) -> Tab {
        let mut tab = Tab::new(window, engine, login_set);
        let Some(address) = Address::parse(&saved.url)
            .ok()
            .filter(|address| matches!(address.scheme(), "http" | "https"))
        else {
            return tab;
        };
        tab.session_title =
            (!saved.title.is_empty()).then(|| (address.as_str().to_owned(), saved.title.clone()));
        tab.title.clone_from(&saved.title);
        tab.pending = Some(address);
        tab.asleep = true;
        tab
    }

    /// The tab as the session keeps it (009): its address when that is the
    /// web's, and the first title its page said.
    fn saved(&mut self, front: bool) -> SavedTab {
        let url = self
            .shown()
            .filter(|address| matches!(address.scheme(), "http" | "https"))
            .map(|address| address.as_str().to_owned())
            .unwrap_or_default();
        if url.is_empty() {
            self.session_title = None;
            return SavedTab {
                url,
                title: String::new(),
                front,
            };
        }
        let kept = self
            .session_title
            .as_ref()
            .is_some_and(|(of, _)| *of == url);
        // While the tab is on its way somewhere, the title is still the
        // page's it is leaving.
        if !kept && self.pending.is_none() && !self.title.is_empty() {
            self.session_title = Some((url.clone(), self.title.clone()));
        }
        let title = self
            .session_title
            .as_ref()
            .filter(|(of, _)| *of == url)
            .map(|(_, title)| title.clone())
            .unwrap_or_default();
        SavedTab { url, title, front }
    }

    /// The user sent the tab to `address`.
    fn begin_user_load(&mut self, address: Address) {
        self.asleep = false;
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
                // What was found went with the page it was found in.
                self.find = None;
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
            // What the engine found is for the text now looked for (010).
            // With no find, it is a late word about one that has stopped.
            Kind::FindResult(result) => {
                if let Some(find) = &mut self.find {
                    find.answered = true;
                    find.found = result.found;
                    find.counted = result.counted;
                    find.matches = if result.counted { result.matches } else { 0 };
                    find.current = if result.counted {
                        result.current.min(result.matches)
                    } else {
                        0
                    };
                }
            }
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
            find: self.find.clone(),
            zoom: self.zoom_view(),
        }
    }

    /// The site of the page that owns the web view (012). `None` with no
    /// page, and for a page with no host.
    fn site(&self) -> Option<&str> {
        self.committed.as_ref()?.site()
    }

    /// What the page is zoomed to, for the snapshot: what its web view was
    /// last told. `None` while the tab has no page with a site.
    fn zoom_view(&self) -> Option<ZoomView> {
        self.web_view?;
        self.site()?;
        Some(ZoomView {
            percent: self.zoom,
            can_zoom_in: zoom_in(self.zoom).is_some(),
            can_zoom_out: zoom_out(self.zoom).is_some(),
        })
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
    /// The session (009): the tabs of its window as last worked out, which
    /// is what a window opened when there is none is given. `None` when
    /// nothing is kept between runs: fake mode.
    saved: Option<Vec<SavedTab>>,
    /// The window whose tabs are the session: the first opened, and the
    /// next one opened once that is closed.
    session_window: Option<WindowId>,
}

impl State {
    /// A state with no windows. `saved` is the session the store kept, to
    /// be given back to the first window; `None` keeps no session at all.
    pub(crate) fn new(space: Space, engine: EngineKind, saved: Option<Vec<SavedTab>>) -> State {
        State {
            saved,
            session_window: None,
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

    /// Opens a window. Its id, the tab in front, and whether its tabs came
    /// back from the session (009): they do when no window has the session,
    /// which is so for the first window of a run, and again once the window
    /// that had it is closed.
    pub(crate) fn open_window(&mut self) -> (WindowId, TabId, bool) {
        let window = WindowId(self.ids.mint());
        let has_session = self
            .session_window
            .is_some_and(|session| self.windows.contains_key(&session));
        let kept = match &self.saved {
            Some(saved) if !has_session => {
                self.session_window = Some(window);
                saved.clone()
            }
            _ => Vec::new(),
        };
        let mut tabs = Vec::new();
        let mut front = None;
        for saved in &kept {
            let id = TabId(self.ids.mint());
            self.tabs.insert(
                id,
                Tab::restored(window, self.engine, self.space.login_set, saved),
            );
            if saved.front {
                front = Some(id);
            }
            tabs.push(id);
        }
        let restored = !tabs.is_empty();
        if tabs.is_empty() {
            let id = TabId(self.ids.mint());
            self.tabs
                .insert(id, Tab::new(window, self.engine, self.space.login_set));
            tabs.push(id);
        }
        // A session with nobody in front (a store that was meddled with)
        // puts its first tab there: a window always has one.
        let front = front.unwrap_or(tabs[0]);
        self.windows.insert(window, Window { tabs, front });
        self.dirty.insert(window);
        (window, front, restored)
    }

    /// The tabs in front that are asleep, and where each was (009): each is
    /// to be loaded now, and is asleep no longer. The tab in front is never
    /// left asleep.
    pub(crate) fn tabs_to_wake(&mut self) -> Vec<(TabId, Address)> {
        let fronts: Vec<TabId> = self.windows.values().map(|window| window.front).collect();
        fronts
            .into_iter()
            .filter_map(|id| {
                let tab = self.tabs.get_mut(&id)?;
                if !tab.asleep {
                    return None;
                }
                tab.asleep = false;
                Some((id, tab.pending.clone()?))
            })
            .collect()
    }

    /// The session as it now is (009): the tabs of its window, in order.
    /// `None` when there is nothing to write: no session is kept, or its
    /// window is closed, which closes none of its tabs.
    pub(crate) fn session(&mut self) -> Option<Vec<SavedTab>> {
        self.saved.as_ref()?;
        let window = self.windows.get(&self.session_window?)?;
        let front = window.front;
        let ids = window.tabs.clone();
        let session: Vec<SavedTab> = ids
            .into_iter()
            .filter_map(|id| Some(self.tabs.get_mut(&id)?.saved(id == front)))
            .collect();
        self.saved = Some(session.clone());
        Some(session)
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

    /// Text is looked for in a tab's page (010), or with empty text the
    /// find stops. The web view to ask, as [`State::web_view_of`] says it.
    pub(crate) fn find(&mut self, tab_id: TabId, text: &str) -> Result<WebViewId, bool> {
        let tab = self.tabs.get_mut(&tab_id).ok_or(false)?;
        let web_view = tab.web_view.ok_or(true)?;
        let before = tab.find.clone();
        if text.is_empty() {
            tab.find = None;
        } else if tab.find.as_ref().is_none_or(|find| find.text != text) {
            // Other text: nothing is known of it yet. The same text again
            // goes to another match, and what is known still holds.
            tab.find = Some(FindView {
                text: text.to_owned(),
                ..FindView::default()
            });
        }
        if tab.find != before {
            self.dirty.insert(tab.window);
        }
        Ok(web_view)
    }

    /// The site of a tab's page, in its login set, and the zoom its web
    /// view was last told (012). `Err(false)` for no such tab, `Err(true)`
    /// for a tab with no page, or a page with no site.
    pub(crate) fn zoom_of(&self, tab: TabId) -> Result<(LoginSetId, String, u32), bool> {
        let tab = self.tabs.get(&tab).ok_or(false)?;
        tab.web_view.ok_or(true)?;
        let site = tab.site().ok_or(true)?;
        Ok((tab.login_set, site.to_owned(), tab.zoom))
    }

    /// A site was given a zoom in a login set: every tab there whose page
    /// is of the site has it. The web views to tell, in the order their
    /// tabs were opened; one that was told that zoom already is not told
    /// again.
    pub(crate) fn set_zoom(
        &mut self,
        login_set: LoginSetId,
        site: &str,
        percent: u32,
    ) -> Vec<WebViewId> {
        let mut to_tell = Vec::new();
        for tab in self.tabs.values_mut() {
            let Some(web_view) = tab.web_view else {
                continue;
            };
            if tab.login_set != login_set || tab.site() != Some(site) || tab.zoom == percent {
                continue;
            }
            tab.zoom = percent;
            self.dirty.insert(tab.window);
            to_tell.push(web_view);
        }
        to_tell
    }

    /// The page a web view now shows, for its zoom: its login set, its site
    /// when it has one, and the zoom the web view was last told.
    pub(crate) fn zoom_at(&self, web_view: WebViewId) -> Option<(LoginSetId, Option<String>, u32)> {
        let tab = self.tab_of(web_view)?;
        Some((tab.login_set, tab.site().map(str::to_owned), tab.zoom))
    }

    /// A web view is told a zoom.
    pub(crate) fn zoomed(&mut self, web_view: WebViewId, percent: u32) {
        let Some(tab) = self.tab_of_mut(web_view) else {
            return;
        };
        tab.zoom = percent;
        let window = tab.window;
        self.dirty.insert(window);
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

    fn tab_of(&self, web_view: WebViewId) -> Option<&Tab> {
        self.web_views
            .get(&web_view)
            .and_then(|tab_id| self.tabs.get(tab_id))
    }

    fn tab_of_mut(&mut self, web_view: WebViewId) -> Option<&mut Tab> {
        self.web_views
            .get(&web_view)
            .and_then(|tab_id| self.tabs.get_mut(tab_id))
    }

    /// The login set a window's history is: its Space's.
    pub(crate) fn history_of(&self, window: WindowId) -> Option<LoginSetId> {
        self.windows
            .contains_key(&window)
            .then_some(self.space.login_set)
    }

    /// The visit to write for the page a web view now shows (008): its login
    /// set and address. `None` when the page is not a visit: it has no
    /// address of the web's, or it is the visit the tab is already at (the
    /// same address again, straight after).
    pub(crate) fn visit_to_record(&self, web_view: WebViewId) -> Option<(LoginSetId, String)> {
        let tab = self.tab_of(web_view)?;
        let address = tab.committed.as_ref()?;
        if !matches!(address.scheme(), "http" | "https") {
            return None;
        }
        let url = address.as_str();
        if tab.visit.as_ref().is_some_and(|visit| visit.url == url) {
            return None;
        }
        Some((tab.login_set, url.to_owned()))
    }

    /// The store has written the page a web view shows as the visit `id`.
    pub(crate) fn visit_recorded(&mut self, web_view: WebViewId, id: i64, url: String) {
        if let Some(tab) = self.tab_of_mut(web_view) {
            tab.visit = Some(TabVisit {
                id,
                url,
                title: String::new(),
                titled: false,
                latest: String::new(),
            });
        }
    }

    /// The title to write for a web view's visit, and the visit: when the
    /// page has said its first title since the visit was written, or, with
    /// `leaving`, when the tab goes on and the last title the page said is
    /// no longer the one written. A page that changes its title all the time
    /// costs the store two writes. No title is never written.
    pub(crate) fn title_to_write(
        &mut self,
        web_view: WebViewId,
        leaving: bool,
    ) -> Option<(i64, String)> {
        let tab = self.tab_of_mut(web_view)?;
        let said = tab.title.clone();
        let visit = tab.visit.as_mut()?;
        if !said.is_empty() {
            visit.latest = said;
        }
        if visit.latest.is_empty() || visit.title == visit.latest || (visit.titled && !leaving) {
            return None;
        }
        visit.title.clone_from(&visit.latest);
        visit.titled = true;
        Some((visit.id, visit.latest.clone()))
    }

    /// Every tab forgets which visit its page is: history was forgotten, and
    /// a title must not be written to a visit that is gone, nor a reload be
    /// taken for a visit that is no longer there.
    pub(crate) fn visits_forgotten(&mut self) {
        for tab in self.tabs.values_mut() {
            tab.visit = None;
        }
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
