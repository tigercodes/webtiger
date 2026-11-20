//! The app: what a shell's requests and reports do to the models.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use tokio::time::Instant;
use wt_core::{
    ACTUAL_SIZE, Address, Clock, Input, SearchEngine, TabId, Timestamp, WebViewId, WindowId,
    classify, opens_outside, searched_words, zoom_in, zoom_out,
};
use wt_proto::v1::{
    Ack, ActivateTab, AddressSuggestion, AddressSuggestions, CloseTab, CloseWindow, EngineEvent,
    EngineKind, FindInPage, ForgetHistory, HistoryEntry, HistoryPage, LinkOpenRequested, LoginSet,
    NavigationCommand, OpenOutside, OpenTab, OutsideLinkRequested, QueryHistory, SearchRequested,
    SubmitAddress, SuggestAddresses, TabCommand, TabOpened, WebViewCommand, WebViewCreate,
    WebViewDestroy, WebViewFind, WebViewLoad, WebViewSetZoom, WindowOpened, WindowSnapshot,
    ZoomChange, ZoomPage, engine_event, forget_history, host_request, request, response,
};
use wt_store::{SavedTab, Space, Store};

use crate::downloads::{Downloading, Wanted};
use crate::error::AppError;
use crate::host::{Host, HostFuture};
use crate::model::{LoadTarget, State};

/// How many visits a page of history holds when the shell does not say
/// (008).
pub const HISTORY_PAGE: u32 = 100;

/// How many places are offered for what is typed when the shell does not
/// say (011).
pub const SUGGESTIONS: u32 = 6;

/// How long after a web view had an address handed to the system before it
/// may have another (007). A click's permission lasts a few seconds in an
/// engine, and a page can ask many times within it: one click opens one
/// thing.
pub const OUTSIDE_LINK_INTERVAL: Duration = Duration::from_secs(1);

/// A request's answer, when it is ready. Whatever the request changes in the
/// models it changes before this is returned, so requests take effect in the
/// order they arrive; the future is only the wait for the shell.
pub type AppFuture = Pin<Box<dyn Future<Output = Result<response::Kind, AppError>>>>;

/// What the app is started with.
#[derive(Clone)]
pub struct AppOptions {
    /// The engine new tabs use: the shell's first.
    pub engine: EngineKind,
    /// Nobody is there to be asked where searches go (020): the engine that
    /// stands in is taken as chosen. Fake mode, unless it asks for a first
    /// run.
    pub search_settled: bool,
    /// Fake mode: what every new window's first tab opens.
    pub fake_start: Option<String>,
    /// Where the app reads the time: a visit's (008).
    pub clock: Rc<dyn Clock>,
    /// Whether a window's tabs are kept between runs (009). Not in fake
    /// mode, where every window opens the same way.
    pub session: bool,
}

/// The models and what changes them.
#[derive(Clone)]
pub struct App {
    pub(crate) state: Rc<RefCell<State>>,
    pub(crate) host: Rc<dyn Host>,
    /// Where searches go (020), and whether the person chose it.
    pub(crate) search: Rc<Cell<(&'static SearchEngine, bool)>>,
    fake_start: Option<Rc<str>>,
    /// When each web view last had an address handed to the system, for
    /// those that had one within [`OUTSIDE_LINK_INTERVAL`].
    outside: Rc<RefCell<BTreeMap<WebViewId, Instant>>>,
    /// The store, for what browsing writes: history (008) and the session
    /// (009).
    pub(crate) store: Rc<RefCell<Store>>,
    pub(crate) clock: Rc<dyn Clock>,
    space: wt_core::SpaceId,
    /// The session as the store has it: written again only when it differs.
    written: Rc<RefCell<Vec<SavedTab>>>,
    /// The downloads that are coming (013), by the shell's name for each.
    pub(crate) downloads: Rc<RefCell<BTreeMap<u64, Downloading>>>,
    /// When each web view last had a download started, for those that had
    /// one within [`DOWNLOAD_INTERVAL`].
    pub(crate) downloaded: Rc<RefCell<BTreeMap<WebViewId, Instant>>>,
    /// The downloads that wait while the person is asked whether their site
    /// may download (015), by the shell's name for each.
    pub(crate) waiting: Rc<RefCell<BTreeMap<u64, Wanted>>>,
    /// The prompt each window's snapshot last showed, and since when (014):
    /// only what was shown, and for a moment, can be answered.
    pub(crate) prompts_shown: Rc<RefCell<BTreeMap<WindowId, (u64, Instant)>>>,
}

impl App {
    /// An app with no windows, whose tabs open in `space`, a Space of
    /// `store`'s.
    #[must_use]
    pub fn new(
        options: AppOptions,
        space: Space,
        store: Rc<RefCell<Store>>,
        host: Rc<dyn Host>,
    ) -> App {
        // A session that cannot be read is no session: the window opens
        // empty, and what it then holds is what is kept.
        let kept = options
            .session
            .then(|| store.borrow().saved_tabs(space.id).unwrap_or_default());
        // Whatever was fetching the downloads the store still calls coming
        // is gone (013 §5). A store that refuses leaves them as they are.
        // The clock is read only when there is one to end: nothing else in
        // a start takes a time.
        if store.borrow().any_download_coming().unwrap_or(false) {
            let _ = store.borrow_mut().interrupt_downloads(options.clock.now());
        }
        // Where searches go (020): what the store has of a choice, if any.
        let search = crate::search::kept(&store.borrow(), options.search_settled);
        App {
            space: space.id,
            written: Rc::new(RefCell::new(kept.clone().unwrap_or_default())),
            state: Rc::new(RefCell::new(State::new(space, options.engine, kept))),
            host,
            search: Rc::new(Cell::new(search)),
            fake_start: options.fake_start.map(Into::into),
            outside: Rc::new(RefCell::new(BTreeMap::new())),
            store,
            clock: options.clock,
            downloads: Rc::new(RefCell::new(BTreeMap::new())),
            downloaded: Rc::new(RefCell::new(BTreeMap::new())),
            waiting: Rc::new(RefCell::new(BTreeMap::new())),
            prompts_shown: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    /// Handles one of the app's requests: a window's (`OpenWindow`,
    /// `CloseWindow`), a tab's (`OpenTab`, `CloseTab`, `ActivateTab`,
    /// `SubmitAddress`, `TabCommand`, `FindInPage`) or history's
    /// (`QueryHistory`, `ForgetHistory`, `SuggestAddresses`), a page's zoom
    /// (`ZoomPage`) or the downloads' (`QueryDownloads`, `CancelDownload`,
    /// `ForgetDownloads`). The dispatcher answers the rest.
    pub fn request(&self, kind: request::Kind) -> AppFuture {
        match kind {
            request::Kind::OpenWindow(_) => self.open_window(),
            request::Kind::CloseWindow(close) => {
                let closing = self.close_window(&close);
                // What waited in its tabs for the person has nobody to ask.
                self.drop_orphaned_downloads();
                closing
            }
            request::Kind::OpenTab(open) => self.open_tab(&open),
            request::Kind::CloseTab(close) => {
                let closing = self.close_tab(&close);
                self.drop_orphaned_downloads();
                closing
            }
            request::Kind::ActivateTab(activate) => self.activate_tab(&activate),
            request::Kind::SubmitAddress(submit) => self.submit_address(&submit),
            request::Kind::TabCommand(command) => self.tab_command(&command),
            request::Kind::FindInPage(find) => self.find_in_page(&find),
            request::Kind::QueryHistory(query) => answer(self.query_history(&query)),
            request::Kind::SuggestAddresses(suggest) => answer(self.suggest_addresses(&suggest)),
            request::Kind::ZoomPage(zoom) => self.zoom_page(&zoom),
            request::Kind::QueryDownloads(query) => answer(self.query_downloads(&query)),
            request::Kind::CancelDownload(cancel) => answer(self.cancel_download(&cancel)),
            request::Kind::ForgetDownloads(forget) => answer(self.forget_downloads(&forget)),
            request::Kind::AnswerPrompt(prompt) => answer(self.answer_prompt(&prompt)),
            request::Kind::QueryPermissions(query) => answer(self.query_permissions(&query)),
            request::Kind::ForgetPermissions(forget) => answer(self.forget_permissions(&forget)),
            request::Kind::ForgetHistory(forget) => answer(self.forget_history(&forget)),
            request::Kind::QuerySearch(_) => answer(Ok(self.search_settings())),
            request::Kind::ChooseSearchEngine(choice) => answer(self.choose_search_engine(&choice)),
            request::Kind::Ping(_)
            | request::Kind::Subscribe(_)
            | request::Kind::Unsubscribe(_)
            | request::Kind::EngineEvent(_) => refuse(AppError::new("request.unsupported")),
        }
    }

    /// Takes a shell's report about a web view. Must be called from inside
    /// a `LocalSet`: a report can leave a web view that no tab has (a window
    /// a page asked for and was refused, a tab a page closed), and the shell
    /// is told to destroy it.
    pub fn notify(&self, event: EngineEvent) {
        let Some(kind) = event.kind else { return };
        let web_view = WebViewId(event.web_view_id);
        if let engine_event::Kind::OutsideLinkRequested(asked) = kind {
            self.outside_link(web_view, &asked);
            return;
        }
        if let engine_event::Kind::LinkOpenRequested(asked) = kind {
            self.link_open(web_view, &asked);
            return;
        }
        if let engine_event::Kind::SearchRequested(asked) = kind {
            self.search_for(web_view, &asked);
            return;
        }
        // A download's reports are about the download (013), whatever has
        // become of the web view that asked for it.
        if self.download_event(web_view, &kind) {
            return;
        }
        // A page that asks to use something of the device's (014) is
        // answered by the app: nothing of the tab's page changes by asking.
        if self.permission_event(web_view, &kind) {
            return;
        }
        // And so is a page that says something, or asks (016).
        if self.dialog_event(web_view, &kind) {
            return;
        }
        // And a server that asks who is there (017).
        if self.sign_in_event(web_view, &kind) {
            return;
        }
        // A server's question is its load's: when another load begins, or
        // one fails or is stopped, no server waits for an answer any more.
        // (A page that commits ends it with everything else, below.)
        if matches!(
            kind,
            engine_event::Kind::NavigationStarted(_) | engine_event::Kind::NavigationFailed(_)
        ) {
            self.sign_ins_ended(web_view);
        }
        // History (008): a page that commits, or changes its own address, is
        // a visit; the page before it is left, and takes the title it ended
        // with. A title that arrives is the visit's when it is its first.
        let arrives = matches!(
            kind,
            engine_event::Kind::NavigationCommitted(_) | engine_event::Kind::AddressChanged(_)
        );
        let titled = matches!(kind, engine_event::Kind::TitleChanged(_));
        // A page that only changes its own address stays on its site.
        let committed = matches!(kind, engine_event::Kind::NavigationCommitted(_));
        // The page is gone, by another or by its renderer: what it was
        // asking the person is over (014).
        if committed || matches!(kind, engine_event::Kind::RendererGone(_)) {
            self.prompts_ended(web_view);
        }
        if arrives {
            self.write_title(web_view, true);
        }
        let unwanted = self.state.borrow_mut().apply(web_view, kind);
        if arrives {
            self.record_visit(web_view);
        } else if titled {
            self.write_title(web_view, false);
        }
        if committed {
            self.zoom_arrived(web_view);
        }
        // A page that closed its own tab may have left one that is asleep
        // in front, and a download that waited there for the person (015).
        self.drop_orphaned_downloads();
        self.wake_in_the_background();
        if let Some(web_view) = unwanted {
            let destroying =
                self.host
                    .request(host_request::Kind::WebViewDestroy(WebViewDestroy {
                        web_view_id: web_view.get(),
                    }));
            // Asked already; this only waits for the answer, so that a shell
            // that never gives one is given up on like any other.
            tokio::task::spawn_local(async move {
                let _ = destroying.await;
            });
        }
    }

    /// Writes the page a web view now shows into history, when it is a visit
    /// (008 §3). A store that refuses loses the visit and nothing else:
    /// there is nobody to tell, and no log an address may go in.
    fn record_visit(&self, web_view: WebViewId) {
        let Some((login_set, url)) = self.state.borrow().visit_to_record(web_view) else {
            return;
        };
        let recorded = self
            .store
            .borrow_mut()
            .record_visit(login_set, &url, "", self.clock.now());
        if let Ok(id) = recorded {
            self.state.borrow_mut().visit_recorded(web_view, id, url);
        }
    }

    /// Writes the title of a web view's visit when it is time to (008 §3):
    /// its first, or, with `leaving`, the one it ended with.
    fn write_title(&self, web_view: WebViewId, leaving: bool) {
        let Some((visit, title)) = self.state.borrow_mut().title_to_write(web_view, leaving) else {
            return;
        };
        let _ = self.store.borrow_mut().retitle_visit(visit, &title);
    }

    /// A page of a window's history (008): its Space's login set's visits,
    /// newest first.
    fn query_history(&self, query: &QueryHistory) -> Result<response::Kind, AppError> {
        let login_set = self
            .state
            .borrow()
            .history_of(WindowId(query.window_id))
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))?;
        let limit = if query.limit == 0 {
            HISTORY_PAGE
        } else {
            query.limit
        };
        // An id beyond any the store could have given is "from the newest".
        let before = i64::try_from(query.before_visit_id)
            .ok()
            .filter(|before| *before > 0);
        let page = self
            .store
            .borrow()
            .visits(login_set, &query.text, before, limit)
            .map_err(|error| AppError::new(error.code()))?;
        let entries = page
            .visits
            .into_iter()
            .map(|visit| HistoryEntry {
                visit_id: u64::try_from(visit.id).unwrap_or_default(),
                // The address as the field shows it; one the parser no
                // longer takes is shown whole.
                display: Address::parse(&visit.url)
                    .map_or_else(|_| visit.url.clone(), |address| address.display()),
                url: visit.url,
                title: visit.title,
                visited_at_ms: visit.visited_at.unix_millis(),
            })
            .collect();
        Ok(response::Kind::HistoryPage(HistoryPage {
            entries,
            more: page.more,
        }))
    }

    /// What to offer for what is typed in a window's address field (011):
    /// places from its Space's own history, best first.
    fn suggest_addresses(&self, suggest: &SuggestAddresses) -> Result<response::Kind, AppError> {
        let login_set = self
            .state
            .borrow()
            .history_of(WindowId(suggest.window_id))
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))?;
        let limit = if suggest.limit == 0 {
            SUGGESTIONS
        } else {
            suggest.limit
        };
        let places = self
            .store
            .borrow()
            .suggestions(login_set, &suggest.text, limit)
            .map_err(|error| AppError::new(error.code()))?;
        let suggestions = places
            .into_iter()
            .map(|place| AddressSuggestion {
                display: Address::parse(&place.url)
                    .map_or_else(|_| place.url.clone(), |address| address.display()),
                url: place.url,
                title: place.title,
            })
            .collect();
        Ok(response::Kind::AddressSuggestions(AddressSuggestions {
            suggestions,
        }))
    }

    /// Forgets visits of a window's history (008): one, those since a time,
    /// or all of them.
    fn forget_history(&self, forget: &ForgetHistory) -> Result<response::Kind, AppError> {
        let login_set = self
            .state
            .borrow()
            .history_of(WindowId(forget.window_id))
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))?;
        let mut store = self.store.borrow_mut();
        let forgotten = match forget.what {
            Some(forget_history::What::VisitId(visit)) => match i64::try_from(visit) {
                Ok(visit) => store.forget_visit(login_set, visit).map(|_| ()),
                // No visit has such an id: there is nothing to forget.
                Err(_) => Ok(()),
            },
            Some(forget_history::What::SinceMs(since)) => {
                store.forget_visits_since(login_set, Timestamp(since))
            }
            Some(forget_history::What::Everything(true)) => store.forget_all_visits(login_set),
            Some(forget_history::What::Everything(false)) | None => {
                return Err(AppError::new("history.nothing_named").field("what"));
            }
        };
        forgotten.map_err(|error| AppError::new(error.code()))?;
        drop(store);
        // No tab's page is a visit any more, as far as writing its title
        // goes: one that was forgotten must stay forgotten.
        self.state.borrow_mut().visits_forgotten();
        Ok(response::Kind::Ack(Ack {}))
    }

    /// A page tried to go to an address that is not a browser's, and the
    /// shell refused it in the web view (007). It is handed to the system
    /// when it is one that may be (a letter, a number, a message), a person
    /// asked, and the web view has not just had one. The page that asked is
    /// left as it is.
    /// The person asked for a link of a page in a new tab (018): by its
    /// menu, or by how they pressed it. A tab is opened for it, or nothing
    /// happens.
    fn link_open(&self, web_view: WebViewId, asked: &LinkOpenRequested) {
        // A page that only makes it look as if its link was pressed so is
        // not obeyed, as one that asks for a window by itself is not (006).
        if !asked.user_gesture {
            return;
        }
        // Only the web's own addresses are a tab's to start from: not a
        // script, not a page of no address, not a file, and not what is
        // for the system, which pressing the link itself opens (007).
        let Ok(address) = Address::parse(&asked.url) else {
            return;
        };
        if !matches!(address.scheme(), "http" | "https") {
            return;
        }
        let opened = self
            .state
            .borrow_mut()
            .open_link_tab(web_view, asked.background);
        let Some(tab) = opened else {
            return;
        };
        // The tab is there whatever becomes of its page: a load that fails
        // shows in the tab.
        if let Some(loading) = self.begin_load_as(tab, address, false) {
            tokio::task::spawn_local(async move {
                let _ = loading.await;
            });
        }
    }

    /// The person asked to search for words of a page (023): words they
    /// had selected. The search opens in a new tab in front, with the
    /// engine they chose (020), or nothing happens.
    fn search_for(&self, web_view: WebViewId, asked: &SearchRequested) {
        // A page that only makes it look as if the person asked is not
        // obeyed: a search sends words to somebody.
        if !asked.user_gesture {
            return;
        }
        let Some(words) = searched_words(&asked.text) else {
            return;
        };
        // Always a search: the words are never taken for an address,
        // whatever they look like.
        let Some(address) = self.search.get().0.search(&words) else {
            return;
        };
        let opened = self.state.borrow_mut().open_link_tab(web_view, false);
        let Some(tab) = opened else {
            return;
        };
        // The person's own search, as one typed in the address field is:
        // the tab says where it goes from the start.
        if let Some(loading) = self.begin_load_as(tab, address, true) {
            tokio::task::spawn_local(async move {
                let _ = loading.await;
            });
        }
    }

    fn outside_link(&self, web_view: WebViewId, asked: &OutsideLinkRequested) {
        // Checked against the core's own policy: what the shell calls an
        // outside link is not taken on its word.
        if !opens_outside(&asked.url) {
            return;
        }
        let now = Instant::now();
        {
            let mut opened = self.outside.borrow_mut();
            opened.retain(|_, at| now.duration_since(*at) < OUTSIDE_LINK_INTERVAL);
            if opened.contains_key(&web_view) {
                return;
            }
            if !self
                .state
                .borrow_mut()
                .may_open_outside(web_view, asked.user_gesture)
            {
                return;
            }
            opened.insert(web_view, now);
        }
        let opening = self
            .host
            .request(host_request::Kind::OpenOutside(OpenOutside {
                url: asked.url.clone(),
            }));
        // Asked already. Nothing is said when nothing opens it: there is
        // nowhere to say it until W1's prompts.
        tokio::task::spawn_local(async move {
            let _ = opening.await;
        });
    }

    /// Loads every tab that is in front and asleep (009): a tab that came
    /// back from the session is loaded when it comes in front. The loads are
    /// asked for before this returns; what it returns is the wait for each.
    fn wake(&self) -> Vec<impl Future<Output = Result<(), AppError>> + use<>> {
        let waking = self.state.borrow_mut().tabs_to_wake();
        waking
            .into_iter()
            .filter_map(|(tab, address)| self.begin_load(tab, address))
            .collect()
    }

    /// [`App::wake`], for where nobody waits for the loads: a failure shows
    /// in the tab.
    fn wake_in_the_background(&self) {
        for loading in self.wake() {
            tokio::task::spawn_local(async move {
                let _ = loading.await;
            });
        }
    }

    /// Writes the session to the store when it is not what the store has
    /// (009). Called when the windows' snapshots are published, so that
    /// what is kept is what was last seen. A store that refuses keeps what
    /// it had: the next change tries again.
    pub fn persist(&self) {
        let Some(session) = self.state.borrow_mut().session() else {
            return;
        };
        if *self.written.borrow() == session {
            return;
        }
        if self
            .store
            .borrow_mut()
            .save_tabs(self.space, &session)
            .is_ok()
        {
            *self.written.borrow_mut() = session;
        }
    }

    /// A window as its subscribers see it, or `None` when there is none.
    #[must_use]
    pub fn window_snapshot(&self, window: WindowId) -> Option<WindowSnapshot> {
        let snapshot = self.state.borrow().window_snapshot(window);
        // What the tab in front asks the person is before them from the
        // snapshot that first has it (014).
        let prompt = snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.tab.as_ref())
            .and_then(|tab| tab.prompt.as_ref())
            .map(|prompt| prompt.prompt_id);
        self.prompt_shown(window, prompt);
        snapshot
    }

    /// The windows that may have changed since this was last called.
    #[must_use]
    pub fn take_dirty(&self) -> BTreeSet<WindowId> {
        self.state.borrow_mut().take_dirty()
    }

    /// The windows closed since this was last called.
    #[must_use]
    pub fn take_closed(&self) -> Vec<WindowId> {
        self.state.borrow_mut().take_closed()
    }

    fn open_window(&self) -> AppFuture {
        let (window, tab, restored) = self.state.borrow_mut().open_window();
        let opened = response::Kind::WindowOpened(WindowOpened {
            window_id: window.get(),
            tab_id: tab.get(),
        });
        if restored {
            // The window has the session's tabs (009): the one in front is
            // loaded now, and the window is open whatever becomes of it.
            let waking = self.wake();
            return Box::pin(async move {
                for loading in waking {
                    let _ = loading.await;
                }
                Ok(opened)
            });
        }
        let start = self
            .fake_start
            .as_deref()
            .and_then(|text| self.address_for(text));
        let Some(loading) = start.and_then(|address| self.begin_load(tab, address)) else {
            return Box::pin(std::future::ready(Ok(opened)));
        };
        Box::pin(async move {
            // The window opened whatever became of its first page: a failed
            // load shows in the tab.
            let _ = loading.await;
            Ok(opened)
        })
    }

    fn close_window(&self, close: &CloseWindow) -> AppFuture {
        let closed = self
            .state
            .borrow_mut()
            .close_window(WindowId(close.window_id));
        let Some(web_views) = closed else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        let destroying: Vec<HostFuture> = web_views
            .into_iter()
            .map(|web_view| {
                self.host
                    .request(host_request::Kind::WebViewDestroy(WebViewDestroy {
                        web_view_id: web_view.get(),
                    }))
            })
            .collect();
        Box::pin(async move {
            // The window is closed in the models already. A shell that fails
            // to destroy a web view keeps a leak, not a window.
            for destroyed in destroying {
                let _ = destroyed.await;
            }
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    fn open_tab(&self, open: &OpenTab) -> AppFuture {
        let after = (open.after_tab_id != 0).then_some(TabId(open.after_tab_id));
        let opened =
            self.state
                .borrow_mut()
                .open_tab(WindowId(open.window_id), after, open.background);
        let Some(tab) = opened else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        let opened = response::Kind::TabOpened(TabOpened { tab_id: tab.get() });
        let page = self
            .address_for(&open.text)
            .and_then(|address| self.begin_load(tab, address));
        let Some(loading) = page else {
            return Box::pin(std::future::ready(Ok(opened)));
        };
        Box::pin(async move {
            // The tab opened whatever became of its page: a failed load
            // shows in the tab.
            let _ = loading.await;
            Ok(opened)
        })
    }

    fn close_tab(&self, close: &CloseTab) -> AppFuture {
        let closed = self.state.borrow_mut().close_tab(TabId(close.tab_id));
        let Some(web_view) = closed else {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        };
        let destroying = web_view.map(|web_view| {
            self.host
                .request(host_request::Kind::WebViewDestroy(WebViewDestroy {
                    web_view_id: web_view.get(),
                }))
        });
        // The tab that came forward may be one that is asleep (009).
        let waking = self.wake();
        Box::pin(async move {
            // The tab is closed in the models already. A shell that fails to
            // destroy its web view keeps a leak, not a tab.
            if let Some(destroyed) = destroying {
                let _ = destroyed.await;
            }
            for loading in waking {
                let _ = loading.await;
            }
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    fn activate_tab(&self, activate: &ActivateTab) -> AppFuture {
        if !self.state.borrow_mut().activate_tab(TabId(activate.tab_id)) {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        }
        // A tab that came back from the session is loaded now that it is in
        // front (009). The tab is in front whatever becomes of its page.
        let waking = self.wake();
        Box::pin(async move {
            for loading in waking {
                let _ = loading.await;
            }
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    fn submit_address(&self, submit: &SubmitAddress) -> AppFuture {
        let tab = TabId(submit.tab_id);
        if self.state.borrow().web_view_of(tab) == Err(false) {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        }
        let Some(address) = self.address_for(&submit.text) else {
            return refuse(AppError::new("address.empty").field("text"));
        };
        let Some(loading) = self.begin_load(tab, address) else {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        };
        Box::pin(async move {
            loading.await?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    fn tab_command(&self, command: &TabCommand) -> AppFuture {
        let web_view = match self.state.borrow().web_view_of(TabId(command.tab_id)) {
            Ok(web_view) => web_view,
            Err(false) => return refuse(AppError::new("tab.unknown").field("tab_id")),
            Err(true) => return refuse(AppError::new("tab.no_page").field("tab_id")),
        };
        if !matches!(
            command.command(),
            NavigationCommand::Back
                | NavigationCommand::Forward
                | NavigationCommand::Reload
                | NavigationCommand::Stop
        ) {
            return refuse(AppError::new("tab.bad_command").field("command"));
        }
        let commanded = self
            .host
            .request(host_request::Kind::WebViewCommand(WebViewCommand {
                web_view_id: web_view.get(),
                command: command.command,
            }));
        Box::pin(async move {
            commanded.await?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    /// Finds text in a tab's page (010), or with empty text stops finding.
    /// The tab's find is what the snapshot shows before this returns; what
    /// the engine found arrives as its report.
    fn find_in_page(&self, find: &FindInPage) -> AppFuture {
        let web_view = match self.state.borrow_mut().find(TabId(find.tab_id), &find.text) {
            Ok(web_view) => web_view,
            Err(false) => return refuse(AppError::new("tab.unknown").field("tab_id")),
            Err(true) => return refuse(AppError::new("tab.no_page").field("tab_id")),
        };
        let finding = self
            .host
            .request(host_request::Kind::WebViewFind(WebViewFind {
                web_view_id: web_view.get(),
                text: find.text.clone(),
                backwards: find.backwards,
            }));
        Box::pin(async move {
            finding.await?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    /// Makes a tab's page bigger or smaller, or puts it back (012). The
    /// level is its site's, in the tab's login set: it is written, and every
    /// tab that shows the site has it, before this returns; what it returns
    /// is the wait for their web views.
    fn zoom_page(&self, zoom: &ZoomPage) -> AppFuture {
        let (login_set, site, now) = match self.state.borrow().zoom_of(TabId(zoom.tab_id)) {
            Ok(found) => found,
            Err(false) => return refuse(AppError::new("tab.unknown").field("tab_id")),
            Err(true) => return refuse(AppError::new("tab.no_page").field("tab_id")),
        };
        let level = match zoom.change() {
            ZoomChange::In => zoom_in(now),
            ZoomChange::Out => zoom_out(now),
            ZoomChange::Reset => Some(ACTUAL_SIZE),
            ZoomChange::Unspecified => {
                return refuse(AppError::new("tab.bad_command").field("change"));
            }
        };
        // No further level, or the level it has: nothing changes.
        let Some(level) = level.filter(|level| *level != now) else {
            return Box::pin(async { Ok(response::Kind::Ack(Ack {})) });
        };
        if let Err(error) = self.store.borrow_mut().set_zoom(login_set, &site, level) {
            return refuse(AppError::new(error.code()));
        }
        let to_tell = self.state.borrow_mut().set_zoom(login_set, &site, level);
        // Asked at once, in order; what follows only waits.
        let asked: Vec<_> = to_tell
            .into_iter()
            .map(|web_view| self.set_web_view_zoom(web_view, level))
            .collect();
        Box::pin(async move {
            for answer in asked {
                answer.await?;
            }
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    /// A page has arrived in a web view (012): it is given its site's zoom,
    /// when the web view was last told another. A store that refuses leaves
    /// the page at actual size.
    fn zoom_arrived(&self, web_view: WebViewId) {
        let Some((login_set, site, now)) = self.state.borrow().zoom_at(web_view) else {
            return;
        };
        let level = site
            .and_then(|site| self.store.borrow().zoom(login_set, &site).ok().flatten())
            .unwrap_or(ACTUAL_SIZE);
        if level == now {
            return;
        }
        self.state.borrow_mut().zoomed(web_view, level);
        let asked = self.set_web_view_zoom(web_view, level);
        // Asked already; this only waits for the answer, so that a shell
        // that never gives one is given up on like any other.
        tokio::task::spawn_local(async move {
            let _ = asked.await;
        });
    }

    /// Asks the shell to zoom a web view. The request is on its way when
    /// this returns.
    fn set_web_view_zoom(&self, web_view: WebViewId, percent: u32) -> HostFuture {
        self.host
            .request(host_request::Kind::WebViewSetZoom(WebViewSetZoom {
                web_view_id: web_view.get(),
                percent,
            }))
    }

    /// Where typed text goes: the address it is, or a search for it. `None`
    /// when it is empty.
    fn address_for(&self, text: &str) -> Option<Address> {
        match classify(text)? {
            Input::Address(address) => Some(address),
            Input::Search(words) => self.search.get().0.search(&words),
        }
    }

    /// Sends a tab to an address: the models change and the shell is asked
    /// before this returns, the web view's creation and its load one after
    /// the other, so two loads asked in a row reach the shell in that order.
    /// The future is the wait for the shell's answers. `None` when there is
    /// no such tab.
    fn begin_load(
        &self,
        tab: TabId,
        address: Address,
    ) -> Option<impl Future<Output = Result<(), AppError>> + use<>> {
        self.begin_load_as(tab, address, true)
    }

    /// The same, for an address that is the user's own (typed, chosen from
    /// history) or, when it is not, where a page's link leads (018).
    fn begin_load_as(
        &self,
        tab: TabId,
        address: Address,
        users: bool,
    ) -> Option<impl Future<Output = Result<(), AppError>> + use<>> {
        let url = address.as_str().to_owned();
        let target = self
            .state
            .borrow_mut()
            .begin_load(tab, users.then_some(address))?;
        let LoadTarget {
            web_view,
            create,
            engine,
            login_set,
        } = target;
        let creating = create.then(|| {
            self.host
                .request(host_request::Kind::WebViewCreate(WebViewCreate {
                    web_view_id: web_view.get(),
                    engine: engine as i32,
                    login_set: Some(LoginSet {
                        id: login_set.to_string(),
                        ephemeral: false,
                    }),
                }))
        });
        let loading = self
            .host
            .request(host_request::Kind::WebViewLoad(WebViewLoad {
                web_view_id: web_view.get(),
                url,
            }));
        let state = Rc::clone(&self.state);
        Some(async move {
            if let Some(creating) = creating
                && let Err(error) = creating.await
            {
                state.borrow_mut().web_view_failed(web_view, error.code());
                // The load was asked of a web view that never came to be;
                // its answer, whatever it is, has nothing to add.
                let _ = loading.await;
                return Err(error.into());
            }
            if let Err(error) = loading.await {
                state.borrow_mut().load_failed(web_view, error.code());
                return Err(error.into());
            }
            Ok(())
        })
    }
}

fn refuse(error: AppError) -> AppFuture {
    Box::pin(std::future::ready(Err(error)))
}

/// The answer to a request that waits for nothing.
fn answer(result: Result<response::Kind, AppError>) -> AppFuture {
    Box::pin(std::future::ready(result))
}
