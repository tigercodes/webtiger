//! The app: what a shell's requests and reports do to the models.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use tokio::time::Instant;
use wt_core::{
    ACTUAL_SIZE, Address, Clock, GroupId, Input, LoginSetId, SearchEngine, TabId, Timestamp, Uuid,
    WebViewId, WindowId, classify, opens_outside, searched_words,
    words::{all_found, typed_words},
    zoom_in, zoom_out,
};
use wt_proto::v1::{
    Ack, ActivateTab, AddressSuggestion, AddressSuggestions, CloseTab, CloseWindow, EngineEvent,
    EngineKind, FindInPage, ForgetHistory, GroupMade, HistoryEntry, HistoryPage, KeepTab,
    LetGoSite, LinkOpenRequested, LoginSet, MakeGroup, MoveGroup, MoveTab, MoveToGroup,
    NavigationCommand, OfferedCommand, OpenAddress, OpenOutside, OpenTab, OpenWindow,
    OutsideLinkRequested, QueryHistory, RemoveGroup, RowPart, RowStep, SearchRequested,
    SearchWords, SendSiteHome, SetFavourite, StepTab, SubmitAddress, SuggestAddresses, TabCommand,
    TabOpened, Undo, UndoArrangement, WebViewCommand, WebViewCreate, WebViewDestroy, WebViewFind,
    WebViewLoad, WebViewRestore, WebViewSetZoom, WindowOpened, WindowSnapshot, ZoomChange,
    ZoomPage, engine_event, forget_history, host_request, move_tab, request, response, step_tab,
};
use wt_store::{ClosedTab, MAX_SUGGESTIONS, SavedTab, Space, Store};

use crate::downloads::{Downloading, Wanted};
use crate::error::{AppError, is_code};
use crate::host::{Host, HostFuture};
use crate::model::{LoadTarget, MoveTo, State, Step};
use crate::spaces::Written;

/// How many visits a page of history holds when the shell does not say
/// (008).
pub const HISTORY_PAGE: u32 = 100;

/// How many places are offered for what is typed when the shell does not
/// say (011).
pub const SUGGESTIONS: u32 = 6;

/// The most open tabs offered among them (038): history must still be seen.
const TABS_OFFERED: usize = 3;

/// The most Spaces, and the most commands, offered among them (048 D5).
const SPACES_OFFERED: usize = 2;
const COMMANDS_OFFERED: usize = 3;

/// The most commands of a shell's that are read for a question (048).
const COMMANDS_READ: usize = 64;

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
    /// Fake mode: how long a tab is out of sight before it is put to sleep
    /// (053), in seconds, looked for every second. 0: the usual time,
    /// looked for every minute.
    pub sleep_after_seconds: u32,
    /// The most tabs that are alive at once (054): what the device can
    /// hold.
    pub live_tabs: usize,
    /// Gives a login set nobody has had, for a Space that is made with
    /// logins of its own (024); `None` when there is none to give. Fake
    /// mode gives the same ones in the same order every run.
    pub new_login_set: Rc<dyn Fn() -> Option<Uuid>>,
    /// Fake mode: a suite's rules for Guard (064), which stand where a
    /// person's would at the start (066). `None`: what the store has kept.
    pub guard_rules: Option<String>,
}

/// The models and what changes them.
#[derive(Clone)]
pub struct App {
    pub(crate) state: Rc<RefCell<State>>,
    pub(crate) host: Rc<dyn Host>,
    /// Where searches go (020), and whether the person chose it.
    pub(crate) search: Rc<Cell<(&'static SearchEngine, bool)>>,
    /// A person's own rules for Guard (066), as they were set; and who is
    /// told when they change.
    pub(crate) guard_rules: Rc<RefCell<Rc<str>>>,
    pub(crate) guard_sink: Rc<RefCell<Option<crate::guard::RulesSink>>>,
    fake_start: Option<Rc<str>>,
    /// When each web view last had an address handed to the system, for
    /// those that had one within [`OUTSIDE_LINK_INTERVAL`].
    outside: Rc<RefCell<BTreeMap<WebViewId, Instant>>>,
    /// The store, for what browsing writes: history (008) and the session
    /// (009).
    pub(crate) store: Rc<RefCell<Store>>,
    /// What private windows keep while they are open (037): a store that is
    /// only ever in memory. `None` when it could not be made; no private
    /// window is opened then.
    pub(crate) private_store: Option<Rc<RefCell<Store>>>,
    pub(crate) clock: Rc<dyn Clock>,
    /// How many of the core's looks a tab is out of sight for before it is
    /// put to sleep (053), and how often the core looks.
    pub(crate) sleep: (u32, std::time::Duration),
    /// The most tabs that are alive at once (054).
    pub(crate) live_tabs: usize,
    pub(crate) new_login_set: Rc<dyn Fn() -> Option<Uuid>>,
    /// The session as the store has it: written again only where it
    /// differs.
    pub(crate) written: Rc<RefCell<Written>>,
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
    /// An app with no windows. `space` is the app's own Space, of
    /// `store`'s: all there is when the store cannot say what Spaces there
    /// are (024).
    #[must_use]
    pub fn new(
        options: AppOptions,
        space: Space,
        store: Rc<RefCell<Store>>,
        host: Rc<dyn Host>,
    ) -> App {
        let kept = crate::spaces::kept(&store.borrow(), space, options.session);
        // Whatever was fetching the downloads the store still calls coming
        // is gone (013 §5). A store that refuses leaves them as they are.
        // The clock is read only when there is one to end: nothing else in
        // a start takes a time.
        if store.borrow().any_download_coming().unwrap_or(false) {
            let _ = store.borrow_mut().interrupt_downloads(options.clock.now());
        }
        // Where searches go (020): what the store has of a choice, if any.
        let search = crate::search::kept(&store.borrow(), options.search_settled);
        // A person's own rules for Guard (066): what the store has kept; or,
        // in fake mode, a suite's.
        let guard_rules: Rc<str> = match options.guard_rules {
            Some(given) => given.into(),
            None => crate::guard::kept(&store.borrow()),
        };
        // What the journal has of each Space's closed tabs (033): what was
        // closed before the app was, can be brought back after.
        let closed: Vec<_> = kept
            .spaces
            .iter()
            .map(|space| (space.id, store.borrow().closed_tabs(space.id).unwrap_or(0)))
            .collect();
        let mut state = State::new(kept.spaces, &kept.start, options.engine, kept.saved);
        state.saved_groups = kept.groups;
        state.layout = crate::layout::kept(&store.borrow());
        for (space, count) in closed {
            state.set_closed_tabs(space, count);
        }
        let app = App {
            written: Rc::new(RefCell::new(kept.written)),
            state: Rc::new(RefCell::new(state)),
            new_login_set: options.new_login_set,
            host,
            search: Rc::new(Cell::new(search)),
            guard_rules: Rc::new(RefCell::new(guard_rules)),
            guard_sink: Rc::new(RefCell::new(None)),
            fake_start: options.fake_start.map(Into::into),
            outside: Rc::new(RefCell::new(BTreeMap::new())),
            store,
            private_store: Store::open_private()
                .ok()
                .map(|store| Rc::new(RefCell::new(store))),
            clock: options.clock,
            sleep: crate::states::sleep_time(options.sleep_after_seconds),
            live_tabs: options.live_tabs,
            downloads: Rc::new(RefCell::new(BTreeMap::new())),
            downloaded: Rc::new(RefCell::new(BTreeMap::new())),
            waiting: Rc::new(RefCell::new(BTreeMap::new())),
            prompts_shown: Rc::new(RefCell::new(BTreeMap::new())),
        };
        // Each Space's time (049), as the store has it.
        app.learn_tidy_times();
        // A state that no saved tab names is of no use to anybody (051): what a
        // run left behind that its session did not keep goes now.
        let _ = app.store.borrow_mut().drop_states_no_tab_names();
        app
    }

    /// Handles one of the app's requests: a window's (`OpenWindow`,
    /// `CloseWindow`), a tab's (`OpenTab`, `CloseTab`, `ActivateTab`,
    /// `SubmitAddress`, `TabCommand`, `FindInPage`) or history's
    /// (`QueryHistory`, `ForgetHistory`, `SuggestAddresses`), a page's zoom
    /// (`ZoomPage`) or the downloads' (`QueryDownloads`, `CancelDownload`,
    /// `ForgetDownloads`). The dispatcher answers the rest.
    pub fn request(&self, kind: request::Kind) -> AppFuture {
        match kind {
            request::Kind::OpenWindow(open) => self.open_window(&open),
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
            request::Kind::QuerySpaces(_) => answer(Ok(self.query_spaces())),
            request::Kind::CreateSpace(create) => answer(self.create_space(&create)),
            request::Kind::RenameSpace(rename) => answer(self.rename_space(&rename)),
            request::Kind::SwitchSpace(switch) => self.switch_space(&switch),
            request::Kind::ColourSpace(colour) => answer(self.colour_space(&colour)),
            request::Kind::IconSpace(icon) => answer(self.icon_space(&icon)),
            request::Kind::ProceedPastCertificate(proceed) => {
                self.proceed_past_certificate(&proceed)
            }
            request::Kind::SearchWords(asked) => self.search_words(&asked),
            request::Kind::OpenAddress(asked) => self.open_address(&asked),
            request::Kind::Undo(undo) => self.undo(&undo),
            request::Kind::StepTab(step) => self.step_tab(&step),
            request::Kind::KeepTab(keep) => answer(self.keep_tab(&keep)),
            request::Kind::LetGoSite(let_go) => answer(self.let_go_site(&let_go)),
            request::Kind::SendSiteHome(send) => self.send_site_home(&send),
            request::Kind::MakeGroup(make) => answer(self.make_group(&make)),
            request::Kind::RenameGroup(rename) => {
                answer(self.change_group(rename.group_id, Some(&rename.name), None, None))
            }
            request::Kind::ColourGroup(colour) => {
                answer(self.change_group(colour.group_id, None, Some(&colour.colour), None))
            }
            request::Kind::ShutGroup(shut) => {
                answer(self.change_group(shut.group_id, None, None, Some(shut.shut)))
            }
            request::Kind::RemoveGroup(remove) => answer(self.remove_group(&remove)),
            request::Kind::MoveToGroup(moved) => answer(self.move_to_group(&moved)),
            request::Kind::SetFavourite(set) => answer(self.set_favourite(&set)),
            request::Kind::SetLayout(set) => answer(self.set_layout(&set)),
            request::Kind::UndoArrangement(undo) => answer(self.undo_arrangement(&undo)),
            request::Kind::MoveTab(moved) => answer(self.move_tab(&moved)),
            request::Kind::MoveGroup(moved) => answer(self.move_group(&moved)),
            request::Kind::QueryIcons(query) => answer(Ok(self.query_icons(&query))),
            request::Kind::SetSpaceTidying(set) => answer(self.set_space_tidying(&set)),
            request::Kind::QueryArchive(query) => answer(self.query_archive(&query)),
            request::Kind::RestoreArchived(restore) => self.restore_archived(&restore),
            request::Kind::ForgetArchived(forget) => answer(self.forget_archived(&forget)),
            request::Kind::OpenDownload(open) => self.open_download(&open),
            request::Kind::MemoryShort(short) => {
                self.memory_short(short.level);
                answer(Ok(response::Kind::Ack(Ack {})))
            }
            request::Kind::QueryAwake(query) => answer(self.query_awake(&query)),
            request::Kind::SleepOthers(asked) => answer(self.sleep_others(&asked)),
            request::Kind::QueryPictures(query) => answer(Ok(self.query_pictures(&query))),
            request::Kind::SetGuardRules(set) => answer(self.set_guard_rules(&set)),
            request::Kind::QueryGuardRules(_) => answer(Ok(self.query_guard_rules())),
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
        // An engine that has the page's icon hands it over (046).
        if let engine_event::Kind::IconReceived(received) = &kind {
            self.icon_received(web_view, received);
        }
        // And one that hands none over says which its page names (046 §5a).
        if let engine_event::Kind::IconNamed(named) = &kind {
            self.icon_named(web_view, named);
        }
        let unwanted = self.state.borrow_mut().apply(web_view, kind);
        if arrives {
            self.record_visit(web_view);
        } else if titled {
            self.write_title(web_view, false);
        }
        if committed {
            self.zoom_arrived(web_view);
            self.icon_waited(web_view);
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
        let recorded = self.store_of(login_set).borrow_mut().record_visit(
            login_set,
            &url,
            "",
            self.clock.now(),
        );
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
            .store_of(login_set)
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

    /// What to offer for what is typed in a window's address field: the open
    /// tabs of the row it shows (038); to a shell that shows them, Spaces
    /// and commands (048); then places from its Space's own history (011),
    /// best first. No more rows than the shell asked for, and where there is
    /// a place to offer, one of them is a place: history is never crowded
    /// out whole.
    fn suggest_addresses(&self, suggest: &SuggestAddresses) -> Result<response::Kind, AppError> {
        let window = WindowId(suggest.window_id);
        let unknown = || AppError::new("window.unknown").field("window_id");
        let login_set = self.state.borrow().history_of(window).ok_or_else(unknown)?;
        let limit = if suggest.limit == 0 {
            SUGGESTIONS
        } else {
            suggest.limit
        }
        .clamp(1, MAX_SUGGESTIONS) as usize;
        // Open tabs first: a page that is open is brought in front, not
        // loaded a second time. A few, so that history is still seen.
        let tabs = self
            .state
            .borrow()
            .tabs_offered(window, &suggest.text, TABS_OFFERED.min(limit))
            .ok_or_else(unknown)?;
        // As many places as would fill the list had no tab's page been among
        // them: a place an offered tab shows is not offered again.
        let places = self
            .store_of(login_set)
            .borrow()
            .suggestions(login_set, &suggest.text, MAX_SUGGESTIONS)
            .map_err(|error| AppError::new(error.code()))?;
        let mut suggestions: Vec<AddressSuggestion> = tabs
            .into_iter()
            .map(|(tab, address, title)| AddressSuggestion {
                display: address.display(),
                url: address.as_str().to_owned(),
                title,
                tab_id: tab.0,
                ..AddressSuggestion::default()
            })
            .collect();
        let open: Vec<String> = suggestions.iter().map(|tab| tab.url.clone()).collect();
        // Then where a person might go that is no page (048): a Space, by
        // its name; and what the window can do, by the shell's words for it.
        let spaces: Vec<AddressSuggestion> = if suggest.spaces {
            self.state
                .borrow()
                .spaces_offered(window, &suggest.text, SPACES_OFFERED)
                .into_iter()
                .map(|(space, name)| AddressSuggestion {
                    title: name,
                    space_id: space.0,
                    ..AddressSuggestion::default()
                })
                .collect()
        } else {
            Vec::new()
        };
        let commands = commands_offered(&suggest.commands, &suggest.text);
        let places: Vec<AddressSuggestion> = places
            .into_iter()
            .filter(|place| !open.contains(&place.url))
            .map(|place| AddressSuggestion {
                display: Address::parse(&place.url)
                    .map_or_else(|_| place.url.clone(), |address| address.display()),
                url: place.url,
                title: place.title,
                ..AddressSuggestion::default()
            })
            .collect();
        // The tabs have their part of the list, as they had (038). Spaces
        // and commands have what the tabs left, less a row for a place where
        // there is a place to offer: history is never crowded out whole by
        // them. Where there is not room for all of them, each kind has a row
        // before either has a second.
        let left = limit.saturating_sub(suggestions.len());
        let room = if places.is_empty() {
            left
        } else {
            left.saturating_sub(1)
        };
        let (spaces, commands) = shared(spaces, commands, room);
        suggestions.extend(spaces);
        suggestions.extend(commands);
        suggestions.extend(places);
        suggestions.truncate(limit);
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
        let mut store = self.store_of(login_set).borrow_mut();
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
        // A closed tab is a place that was visited (033): whatever was named
        // to be forgotten, every closed tab of these logins goes with it.
        // What was asked to be forgotten does not come back by undo.
        store
            .forget_closed_tabs(login_set)
            .map_err(|error| AppError::new(error.code()))?;
        drop(store);
        let spaces = self.state.borrow().spaces_of(login_set);
        for space in spaces {
            self.count_closed_tabs(space);
        }
        // No tab's page is a visit any more, as far as writing its title
        // goes: one that was forgotten must stay forgotten.
        // Nor does a tab of these logins keep the state it had (051): the
        // list a tab goes back through is what was visited. They are gone
        // from the store's files with this answer, as the visits are.
        self.state.borrow_mut().visits_forgotten(login_set);
        self.forget_states()
            .map_err(|error| AppError::new(error.code()))?;
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
        let Some(address) = Address::web(&asked.url) else {
            return;
        };
        let opened = self
            .state
            .borrow_mut()
            .open_link_tab(web_view, asked.background);
        let Some(tab) = opened else {
            return;
        };
        // The tab is there whatever becomes of its page: a load that fails
        // shows in the tab.
        if let Some(loading) = self.begin_load_as(tab, address, false, None) {
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
        if let Some(loading) = self.begin_load_as(tab, address, true, None) {
            tokio::task::spawn_local(async move {
                let _ = loading.await;
            });
        }
    }

    /// Words the person chose outside any page, handed to a shell to search
    /// for (028): another app's selected text. Tidied as a page's selected
    /// words are, and always searched for, in a new tab in front.
    fn search_words(&self, asked: &SearchWords) -> AppFuture {
        let window = WindowId(asked.window_id);
        let Some(front) = self.state.borrow().front_tab(window) else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        // Always a search: another app's words are never taken for an
        // address, whatever they look like.
        let searched =
            searched_words(&asked.text).and_then(|words| self.search.get().0.search(&words));
        let Some(address) = searched else {
            return refuse(AppError::new("search.no_words").field("text"));
        };
        self.open_in_front(window, front, address)
    }

    /// The system handed the shell an address from outside WebTiger (031):
    /// a link pressed in another app. It opens in a new tab, after the tab
    /// in front, in front, in the Space the window shows; or it is refused
    /// and nothing opens.
    fn open_address(&self, asked: &OpenAddress) -> AppFuture {
        let window = WindowId(asked.window_id);
        let Some(front) = self.state.borrow().front_tab(window) else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        // Only an address of the web. It is another app's, and it is not
        // typed text: it is not mended, and what is not an address is not
        // searched for. Nor is it a page's navigation: nothing of it goes
        // on to the system (007).
        let Some(address) = Address::web(&asked.url) else {
            return refuse(AppError::new("address.not_web").field("url"));
        };
        self.open_in_front(window, front, address)
    }

    /// The window an address from outside is opened in when nobody names
    /// one (063): the first that was opened and is still there, and is not
    /// private. `None` when there is none.
    pub(crate) fn window_for_outside(&self) -> Option<WindowId> {
        self.state.borrow().first_ordinary_window()
    }

    /// Opens an address as one the system handed a shell from outside is
    /// opened (031), in `window`: fake mode's own list (063). The wait for
    /// its page; an address that is none of the web opens nothing.
    pub(crate) fn open_from_outside(&self, window: WindowId, url: String) -> AppFuture {
        self.open_address(&OpenAddress {
            window_id: window.get(),
            url,
        })
    }

    /// Opens a tab after `front`, in front, and sends it to an address the
    /// person asked for from outside any tab: a search for words, or a
    /// link pressed in another app. Answered with the tab that opened.
    fn open_in_front(&self, window: WindowId, front: TabId, address: Address) -> AppFuture {
        // What comes from outside the browser never opens in a private
        // window (037): the shell is told, and chooses another.
        if self.state.borrow().is_private_window(window) {
            return refuse(AppError::new("window.private").field("window_id"));
        }
        let opened = self.state.borrow_mut().open_tab(window, Some(front), false);
        let Some(tab) = opened else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        let opened = response::Kind::TabOpened(TabOpened { tab_id: tab.get() });
        // The person's own, as an address typed in the field is: the tab
        // says where it goes from the start.
        let Some(loading) = self.begin_load_as(tab, address, true, None) else {
            return Box::pin(std::future::ready(Ok(opened)));
        };
        Box::pin(async move {
            // The tab opened whatever became of its page: a failed load
            // shows in the tab.
            let _ = loading.await;
            Ok(opened)
        })
    }

    /// Undoes the last organising action of the Space a window shows (033):
    /// so far, brings back the tab the person last closed there.
    fn undo(&self, undo: &Undo) -> AppFuture {
        let window = WindowId(undo.window_id);
        let Some(space) = self.state.borrow().space_shown(window) else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        // A private window's closed tabs are kept nowhere (037).
        if self.state.borrow().is_private_space(space) {
            return refuse(AppError::new("undo.nothing"));
        }
        // Taken out of the journal: what is undone is not undone twice. A
        // journal that cannot be read has nothing to undo.
        let taken = self
            .store
            .borrow_mut()
            .take_closed_tab(space)
            .ok()
            .flatten();
        self.count_closed_tabs(space);
        let Some(closed) = taken else {
            return refuse(AppError::new("undo.nothing"));
        };
        let saved = SavedTab {
            url: closed.url,
            title: closed.title,
            front: true,
            home: String::new(),
            name: String::new(),
            group: 0,
            favourite: false,
            shown_at: 0,
            state: 0,
        };
        let reopened = self
            .state
            .borrow_mut()
            .reopen(window, closed.position, &saved);
        let Some(tab) = reopened else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        let opened = response::Kind::TabOpened(TabOpened { tab_id: tab.get() });
        // It came back asleep and in front, as a tab of the session does
        // (009): waking it loads it, as the person's own.
        let waking = self.wake();
        Box::pin(async move {
            // The tab is back whatever becomes of its page: a failed load
            // shows in the tab.
            for loading in waking {
                let _ = loading.await;
            }
            Ok(opened)
        })
    }

    /// The store that keeps what a login set's windows keep: the one on
    /// disk; or, for a private window's login set, the one in memory (037).
    /// A private login set is only ever made when that store is there.
    ///
    /// Not to be called while the state is borrowed to be changed.
    pub(crate) fn store_of(&self, login_set: LoginSetId) -> &Rc<RefCell<Store>> {
        match &self.private_store {
            Some(private) if self.state.borrow().is_private(login_set) => private,
            _ => &self.store,
        }
    }

    /// Asks the journal how many closed tabs of a Space it has (033), for
    /// what the windows that show it are told. A journal that cannot be
    /// asked is one with nothing to undo.
    fn count_closed_tabs(&self, space: wt_core::SpaceId) {
        // A private window's row is no Space of the store's, and has no
        // closed tabs (037).
        if self.state.borrow().is_private_space(space) {
            return;
        }
        let count = self.store.borrow().closed_tabs(space).unwrap_or(0);
        self.state.borrow_mut().set_closed_tabs(space, count);
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
    pub(crate) fn wake(&self) -> Vec<impl Future<Output = Result<(), AppError>> + use<>> {
        let waking = self.state.borrow_mut().tabs_to_wake();
        waking
            .into_iter()
            .filter_map(|(tab, address, kept)| {
                // From its kept state, where the store has it (051); and
                // with the picture kept with that state (059), which is the
                // store's no longer than the state is.
                let (state, picture) = kept.map_or((None, None), |id| {
                    let store = self.store.borrow();
                    (
                        store.kept_state(id).ok().flatten(),
                        store.kept_picture(id).ok().flatten(),
                    )
                });
                let loading = self.begin_load_as(tab, address, true, state)?;
                if let (Some(id), Some(picture)) = (kept, picture) {
                    self.wake_with_picture(tab, id, picture);
                }
                Some(loading)
            })
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

    /// A window as its subscribers see it, or `None` when there is none.
    #[must_use]
    pub fn window_snapshot(&self, window: WindowId) -> Option<WindowSnapshot> {
        let mut snapshot = self.state.borrow().window_snapshot(window);
        // A tab whose host the store was not yet asked about was told as
        // having no icon (046): the store is asked, and when it has one the
        // window is told with it.
        if self.learn_asked_icons() {
            snapshot = self.state.borrow().window_snapshot(window);
        }
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

    fn open_window(&self, open: &OpenWindow) -> AppFuture {
        // A private window (037) has a login set made now, for it alone: in
        // no store on disk, and in the one in memory for what the window
        // keeps while it is open.
        let private = if open.private_window {
            let made = self
                .private_store
                .as_ref()
                .zip((self.new_login_set)().map(LoginSetId));
            let Some((store, login_set)) = made else {
                return refuse(AppError::new("window.no_private"));
            };
            if store
                .borrow_mut()
                .add_login_set(login_set, self.clock.now())
                .is_err()
            {
                return refuse(AppError::new("window.no_private"));
            }
            Some(login_set)
        } else {
            None
        };
        let (window, tab, restored) = self.state.borrow_mut().open_window(private);
        // The tabs that came back past their Space's time are tidied before
        // the window is told of its row (049): none shows and then goes.
        self.tidy();
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
        let window = WindowId(close.window_id);
        let private = self.state.borrow().private_login_set(window);
        let closed = self.state.borrow_mut().close_window(window);
        let Some(web_views) = closed else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        // What a private window kept while it was open goes with it (037).
        if let (Some(login_set), Some(store)) = (private, &self.private_store) {
            let _ = store.borrow_mut().drop_login_set(login_set);
        }
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

    pub(crate) fn open_tab(&self, open: &OpenTab) -> AppFuture {
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
        // The journal first (033): a tab the person closes is written down
        // before it is closed, so that it can be brought back. A journal
        // that cannot be written does not keep the tab open: it closes, and
        // cannot be brought back.
        let closing = self.state.borrow_mut().closing(TabId(close.tab_id));
        if let Some((space, position, saved)) = closing {
            let entry = ClosedTab {
                position,
                url: saved.url,
                title: saved.title,
            };
            let written =
                self.store
                    .borrow_mut()
                    .journal_closed_tab(space, &entry, self.clock.now());
            if written.is_ok() {
                self.count_closed_tabs(space);
            }
        }
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
        let chosen = TabId(activate.tab_id);
        // Chosen from what was offered while typing (038): an empty tab that
        // was typed in was opened only to go somewhere. Which tab is left is
        // looked at before the other comes in front.
        let left = if activate.chosen_while_typing {
            self.state.borrow().empty_tab_left_for(chosen)
        } else {
            None
        };
        if !self.state.borrow_mut().activate_tab(chosen) {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        }
        if let Some(left) = left {
            // It has no web view to destroy, and nothing to bring back: the
            // journal keeps no empty tab (033).
            self.state.borrow_mut().close_tab(left);
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

    /// Keeps a tab: it becomes a Site (039). Its place in the row, and what
    /// closing it means from now on, are the model's.
    fn keep_tab(&self, keep: &KeepTab) -> Result<response::Kind, AppError> {
        let tab = TabId(keep.tab_id);
        let mut state = self.state.borrow_mut();
        let row = state.row_of_tab(tab);
        state
            .arranging(row, "site.kept", |state| state.keep_tab(tab))
            .map_err(|code| AppError::new(code).field("tab_id"))?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Lets a Site go (039): it is a tab like any other again.
    fn let_go_site(&self, let_go: &LetGoSite) -> Result<response::Kind, AppError> {
        let tab = TabId(let_go.tab_id);
        let mut state = self.state.borrow_mut();
        let row = state.row_of_tab(tab);
        if !state.arranging(row, "site.let_go", |state| state.let_go_site(tab)) {
            return Err(AppError::new("tab.unknown").field("tab_id"));
        }
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Makes a Group in the row a window shows (041).
    fn make_group(&self, make: &MakeGroup) -> Result<response::Kind, AppError> {
        let window = WindowId(make.window_id);
        let mut state = self.state.borrow_mut();
        let row = state.row_shown(window);
        let group = state
            .arranging(row, "group.made", |state| {
                state.make_group(window, &make.name)
            })
            .map_err(|code| AppError::new(code).field(group_field(code)))?;
        Ok(response::Kind::GroupMade(GroupMade {
            group_id: group.get(),
        }))
    }

    /// Names a Group, colours it, or shuts and opens it (041): whichever is
    /// given.
    fn change_group(
        &self,
        group: u64,
        name: Option<&str>,
        colour: Option<&str>,
        shut: Option<bool>,
    ) -> Result<response::Kind, AppError> {
        let group = GroupId(group);
        let mut state = self.state.borrow_mut();
        // A Group named or coloured is arranged (044): what it was is kept to
        // go back to. Shut or opened, it is only looked at another way.
        let kind = match (name, colour) {
            (Some(_), _) => Some("group.renamed"),
            (None, Some(_)) => Some("group.coloured"),
            (None, None) => None,
        };
        let changed = match kind {
            Some(kind) => {
                let row = state.row_of_group(group);
                state.arranging(row, kind, |state| {
                    state.change_group(group, name, colour, shut)
                })
            }
            None => state.change_group(group, name, colour, shut),
        };
        changed.map_err(|code| AppError::new(code).field(group_field(code)))?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Takes a Group away (041): its Sites stay.
    fn remove_group(&self, remove: &RemoveGroup) -> Result<response::Kind, AppError> {
        let group = GroupId(remove.group_id);
        let mut state = self.state.borrow_mut();
        let row = state.row_of_group(group);
        if !state.arranging(row, "group.removed", |state| state.remove_group(group)) {
            return Err(AppError::new("group.unknown").field("group_id"));
        }
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Puts a tab in a Group, or takes a Site out of its Group (041).
    fn move_to_group(&self, moved: &MoveToGroup) -> Result<response::Kind, AppError> {
        let tab = TabId(moved.tab_id);
        let group = (moved.group_id != 0).then_some(GroupId(moved.group_id));
        let mut state = self.state.borrow_mut();
        let row = state.row_of_tab(tab);
        state
            .arranging(row, "group.moved", |state| state.move_to_group(tab, group))
            .map_err(|code| AppError::new(code).field(group_field(code)))?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Marks a tab as a Favourite, or unmarks it (042).
    fn set_favourite(&self, set: &SetFavourite) -> Result<response::Kind, AppError> {
        let tab = TabId(set.tab_id);
        let kind = if set.favourite {
            "favourite.added"
        } else {
            "favourite.removed"
        };
        let mut state = self.state.borrow_mut();
        let row = state.row_of_tab(tab);
        state
            .arranging(row, kind, |state| state.set_favourite(tab, set.favourite))
            .map_err(|code| AppError::new(code).field("tab_id"))?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Moves a tab to another place in its row (045).
    fn move_tab(&self, moved: &MoveTab) -> Result<response::Kind, AppError> {
        let tab = TabId(moved.tab_id);
        let to = match moved.to {
            Some(move_tab::To::BeforeTabId(other)) => Some(MoveTo::Before(TabId(other))),
            Some(move_tab::To::EndOfGroupId(group)) => Some(MoveTo::EndOfGroup(GroupId(group))),
            Some(move_tab::To::EndOf(part)) => match RowPart::try_from(part) {
                Ok(RowPart::Favourites) => Some(MoveTo::EndOfFavourites),
                Ok(RowPart::Sites) => Some(MoveTo::EndOfSites),
                Ok(RowPart::Tabs) => Some(MoveTo::EndOfTabs),
                Ok(RowPart::Unspecified) | Err(_) => None,
            },
            Some(move_tab::To::By(step)) => match RowStep::try_from(step) {
                Ok(RowStep::Back) => Some(MoveTo::Back),
                Ok(RowStep::Forward) => Some(MoveTo::Forward),
                Ok(RowStep::Unspecified) | Err(_) => None,
            },
            None => None,
        };
        let mut state = self.state.borrow_mut();
        // A tab there is not is refused first, whatever else was said.
        let Some(row) = state.row_of_tab(tab) else {
            return Err(AppError::new("tab.unknown").field("tab_id"));
        };
        let Some(to) = to else {
            return Err(AppError::new("move.nowhere").field("to"));
        };
        state
            .arranging(Some(row), "tab.moved", |state| state.move_tab(tab, to))
            .map_err(|code| {
                AppError::new(code).field(match code {
                    "move.other_row" => "before_tab_id",
                    "group.unknown" => "end_of_group_id",
                    _ => "tab_id",
                })
            })?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Moves a Group among its row's Groups (045).
    fn move_group(&self, moved: &MoveGroup) -> Result<response::Kind, AppError> {
        let group = GroupId(moved.group_id);
        let before = (moved.before_group_id != 0).then_some(GroupId(moved.before_group_id));
        let mut state = self.state.borrow_mut();
        let Some(row) = state.row_of_group(group) else {
            return Err(AppError::new("group.unknown").field("group_id"));
        };
        state
            .arranging(Some(row), "group.reordered", |state| {
                state.move_group(group, before)
            })
            .map_err(|code| AppError::new(code).field("before_group_id"))?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Undoes the newest thing done to arrange the row a window shows (044).
    fn undo_arrangement(&self, undo: &UndoArrangement) -> Result<response::Kind, AppError> {
        self.state
            .borrow_mut()
            .undo_arrangement(WindowId(undo.window_id))
            .map_err(|code| AppError::new(code).field("window_id"))?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Sends a Site back to its home (039): the home is loaded in it, as an
    /// address the person asked for. Nothing to do for a tab that is no
    /// Site, or is not away.
    fn send_site_home(&self, send: &SendSiteHome) -> AppFuture {
        let tab = TabId(send.tab_id);
        let unknown = || AppError::new("tab.unknown").field("tab_id");
        let home = match self.state.borrow().home_to_go_to(tab) {
            Err(_) => return refuse(unknown()),
            Ok(None) => return answer(Ok(response::Kind::Ack(Ack {}))),
            Ok(Some(home)) => home,
        };
        let Some(loading) = self.begin_load(tab, home) else {
            return refuse(unknown());
        };
        Box::pin(async move {
            loading.await?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    /// Another tab of the window comes in front, by where it is in the row
    /// (035). Which tab that is, is decided here; coming in front is what
    /// it is for ActivateTab.
    fn step_tab(&self, step: &StepTab) -> AppFuture {
        let window = WindowId(step.window_id);
        let to = match step.to() {
            step_tab::To::Next => Some(Step::Next),
            step_tab::To::Previous => Some(Step::Previous),
            step_tab::To::Last => Some(Step::Last),
            step_tab::To::Place => Some(Step::Place(step.place)),
            step_tab::To::Unspecified => None,
        };
        // A window there is not is refused first, whatever else was said.
        let found = match to {
            Some(to) => self.state.borrow().tab_at_step(window, to),
            None => self.state.borrow().tab_at_step(window, Step::Last),
        };
        let Some(tab) = found else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        if to.is_none() {
            return refuse(AppError::new("tab.step").field("to"));
        }
        self.state.borrow_mut().activate_tab(tab);
        // A tab that came back from the session is loaded now that it is in
        // front (009).
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
        if let Err(error) = self
            .store_of(login_set)
            .borrow_mut()
            .set_zoom(login_set, &site, level)
        {
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
            .and_then(|site| {
                self.store_of(login_set)
                    .borrow()
                    .zoom(login_set, &site)
                    .ok()
                    .flatten()
            })
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
    pub(crate) fn begin_load(
        &self,
        tab: TabId,
        address: Address,
    ) -> Option<impl Future<Output = Result<(), AppError>> + use<>> {
        self.begin_load_as(tab, address, true, None)
    }

    /// The same, for an address that is the user's own (typed, chosen from
    /// history) or, when it is not, where a page's link leads (018).
    ///
    /// With `kept`, a state and the engine it is of, the web view is made
    /// from it where it is of the engine the tab has (051): the shell is
    /// asked to restore it, and loads the address itself should its engine
    /// not take the state.
    fn begin_load_as(
        &self,
        tab: TabId,
        address: Address,
        users: bool,
        kept: Option<(i32, Vec<u8>)>,
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
                        // A private window's (037): the engine keeps
                        // nothing of it.
                        ephemeral: self.state.borrow().is_private(login_set),
                    }),
                }))
        });
        let kept = kept
            .filter(|(of, _)| *of == engine as i32)
            .map(|(_, state)| state);
        let loading = self.host.request(match kept {
            Some(state) => host_request::Kind::WebViewRestore(WebViewRestore {
                web_view_id: web_view.get(),
                state,
                url,
            }),
            None => host_request::Kind::WebViewLoad(WebViewLoad {
                web_view_id: web_view.get(),
                url,
            }),
        });
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

/// The field of a Group's request that a code is about.
fn group_field(code: &str) -> &'static str {
    match code {
        "group.name_empty" => "name",
        "group.colour_unknown" => "colour",
        "group.unknown" => "group_id",
        "window.unknown" | "window.private" | "group.too_many" => "window_id",
        _ => "tab_id",
    }
}

pub(crate) fn refuse(error: AppError) -> AppFuture {
    Box::pin(std::future::ready(Err(error)))
}

/// The answer to a request that waits for nothing.
fn answer(result: Result<response::Kind, AppError>) -> AppFuture {
    Box::pin(std::future::ready(result))
}

/// As many of two lists as there is room for, each in its order: the first
/// of each before the second of either, so that neither kind is crowded out
/// by the other while there is room for one of each.
fn shared<T>(mut first: Vec<T>, mut second: Vec<T>, room: usize) -> (Vec<T>, Vec<T>) {
    let (mut one, mut other) = (0, 0);
    while one + other < room && (one < first.len() || other < second.len()) {
        if one < first.len() && (one <= other || other >= second.len()) {
            one += 1;
        } else {
            other += 1;
        }
    }
    first.truncate(one);
    second.truncate(other);
    (first, second)
}

/// The commands to offer for what is typed (048): of those the shell says
/// can be done now, the ones whose words have every word typed, in the
/// shell's order, at most [`COMMANDS_OFFERED`]. A command whose name is not of
/// the kind the core's codes are, or that has no words, is passed over; and
/// no more than [`COMMANDS_READ`] are read.
fn commands_offered(commands: &[OfferedCommand], text: &str) -> Vec<AddressSuggestion> {
    let typed = typed_words(text);
    commands
        .iter()
        .take(COMMANDS_READ)
        .filter(|command| is_code(&command.name) && all_found(&typed, &[&command.words]))
        .take(COMMANDS_OFFERED)
        .map(|command| AddressSuggestion {
            title: command.words.clone(),
            command: command.name.clone(),
            ..AddressSuggestion::default()
        })
        .collect()
}
