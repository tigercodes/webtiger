//! The models: windows and tabs, and what each one shows. Plain state with
//! no I/O: [`crate::App`] changes it and asks the shell for the rest.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use wt_core::words::{all_found, typed_words};
use wt_core::{
    ACTUAL_SIZE, Address, IdMint, LoginSetId, Security, SpaceId, TabId, WebViewId, WindowId,
    zoom_in, zoom_out,
};
use wt_proto::v1::{
    AddressView, DialogKind, DialogPrompt, DownloadPrompt, EngineKind, FindView, LoadState,
    NavigationFailed, PermissionKind, PermissionPrompt, PromptView, SignInPrompt, TabSummary,
    TabView, WindowSnapshot, ZoomView, address_view, engine_event, prompt_view,
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

/// What a window is told an undo would undo when a closed tab can be brought
/// back (033): `WindowSnapshot.undo`.
pub(crate) const UNDO_CLOSED_TAB: &str = "tab.closed";

/// The most a tab asks at once (014 §3): the prompt it shows, and three
/// waiting behind it. A page that asks for more is told no.
pub(crate) const MOST_PROMPTS: usize = 4;

/// A page's request that waits for the person (014): the tab's prompt, or
/// one waiting its turn.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Asking {
    /// The core's name for the prompt.
    prompt: u64,
    /// The shell's name for the request.
    pub(crate) request: u64,
    /// Who asks: the page's origin.
    pub(crate) origin: String,
    /// Who asks, as the prompt names it.
    site: String,
    pub(crate) kinds: Vec<PermissionKind>,
    /// What it is that waits: a page's request for permission, a download,
    /// or what a page says.
    what: AskedFor,
}

/// What a prompt is about. The shell names each kind's requests by numbers
/// of its own, from one: the kind is part of the name.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AskedFor {
    /// A page asks to use something of the device's (014).
    Permission,
    /// A download waits to be allowed (015), under this name.
    Download { file_name: String },
    /// A page says something, or asks (016).
    Dialog {
        kind: DialogKind,
        message: String,
        default_text: String,
        /// The page said something before: it can be stopped.
        may_silence: bool,
    },
    /// A server asks who is there (017).
    SignIn {
        /// The server's own name for what it protects: its questions are
        /// told apart by it, and it is never shown.
        realm: String,
        /// What is typed reaches the server unread.
        confidential: bool,
        /// The server refused an answer already.
        again: bool,
    },
}

/// The kind of thing that waited for the person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Waited {
    Permission,
    Download,
    Dialog,
    SignIn,
}

impl Asking {
    fn view(&self) -> PromptView {
        let kind = match &self.what {
            AskedFor::Download { file_name } => prompt_view::Kind::Download(DownloadPrompt {
                site: self.site.clone(),
                file_name: file_name.clone(),
            }),
            AskedFor::Permission => prompt_view::Kind::Permission(PermissionPrompt {
                site: self.site.clone(),
                kinds: self.kinds.iter().map(|kind| *kind as i32).collect(),
            }),
            AskedFor::Dialog {
                kind,
                message,
                default_text,
                may_silence,
            } => prompt_view::Kind::Dialog(DialogPrompt {
                site: self.site.clone(),
                kind: *kind as i32,
                message: message.clone(),
                default_text: default_text.clone(),
                may_silence: *may_silence,
            }),
            AskedFor::SignIn {
                confidential,
                again,
                ..
            } => prompt_view::Kind::SignIn(SignInPrompt {
                site: self.site.clone(),
                confidential: *confidential,
                again: *again,
            }),
        };
        PromptView {
            prompt_id: self.prompt,
            kind: Some(kind),
        }
    }

    /// The kind of thing this is.
    pub(crate) fn waited(&self) -> Waited {
        match self.what {
            AskedFor::Permission => Waited::Permission,
            AskedFor::Download { .. } => Waited::Download,
            AskedFor::Dialog { .. } => Waited::Dialog,
            AskedFor::SignIn { .. } => Waited::SignIn,
        }
    }

    /// When this is a server asking who is there: its realm.
    pub(crate) fn sign_in(&self) -> Option<&str> {
        match &self.what {
            AskedFor::SignIn { realm, .. } => Some(realm),
            _ => None,
        }
    }

    /// When this is what a page said or asked: whether it may be silenced.
    pub(crate) fn dialog(&self) -> Option<bool> {
        match self.what {
            AskedFor::Dialog { may_silence, .. } => Some(may_silence),
            _ => None,
        }
    }

    /// What is to be answered, once this is settled.
    pub(crate) fn settled(&self) -> Settled {
        Settled {
            request: self.request,
            waited: self.waited(),
        }
    }
}

/// Something that waited for the person and has its answer, by the shell's
/// name for it among its kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Settled {
    pub(crate) request: u64,
    pub(crate) waited: Waited,
}

/// Who says something in a web view's page (016): the page's origin, the
/// name it is shown by, and how the page stands with the person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Speaker {
    pub(crate) login_set: LoginSetId,
    pub(crate) origin: String,
    pub(crate) site: String,
    /// The person stopped this page's messages.
    pub(crate) silenced: bool,
    /// The page has said something before.
    pub(crate) spoke_before: bool,
}

/// Who asks for a download, and whether the person asked for it themselves
/// (015 §3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Downloader {
    pub(crate) login_set: LoginSetId,
    /// The site: the page's origin, or with no page, where the file comes
    /// from. `None` when neither is the web's: there is nothing to remember
    /// an answer by.
    pub(crate) origin: Option<String>,
    /// The site as the prompt names it.
    pub(crate) site: String,
    /// The load in flight was the person's own, and no page has come of it:
    /// what it turns out to be, they asked for.
    pub(crate) own: bool,
}

/// Who a web view's page is to a request for permission (014): its login
/// set, its origin, and the name the origin is shown by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Asker {
    pub(crate) login_set: LoginSetId,
    pub(crate) origin: String,
    pub(crate) site: String,
}

/// A tab: a browsing context, with or without a web view behind it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tab {
    pub(crate) window: WindowId,
    /// The Space it was opened in (024): it is in that Space's row of its
    /// window, and in no other.
    space: SpaceId,
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
    /// What the tab's page asks the person (014): the first is shown, the
    /// rest wait their turn.
    prompts: VecDeque<Asking>,
    /// What the person would not let this page use: it is not asked for the
    /// same again until another page has committed.
    refused: Vec<PermissionKind>,
    /// The person sent the tab somewhere, and no page has come of it yet: a
    /// download that comes of it is what they asked for (015). Good for
    /// one.
    own_load: bool,
    /// Where the tab's failed load was going, when it failed for its site's
    /// certificate and the adapter holds what it refused (026): the load may
    /// be gone on with. `None` for any other failure, and once another load
    /// begins.
    passable: Option<Address>,
    /// The page has said something or asked (016): its next can be stopped.
    spoke: bool,
    /// The person stopped this page's messages: it says no more until
    /// another page has committed.
    silenced: bool,
    /// The empty tab that took the place of a row's last tab when that was
    /// closed (005). It makes way when the closed tab is brought back (033);
    /// an empty tab the person opened does not.
    stands_in: bool,
    /// The tab is kept (039): a Site.
    site: Option<Site>,
}

/// What makes a tab a Site (039): where its home is, and what it is called
/// while it is asleep. Both are what the page was when the tab was kept.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Site {
    home: Address,
    name: String,
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
    fn new(window: WindowId, space: SpaceId, engine: EngineKind, login_set: LoginSetId) -> Tab {
        Tab {
            window,
            space,
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
            prompts: VecDeque::new(),
            refused: Vec::new(),
            own_load: false,
            passable: None,
            spoke: false,
            silenced: false,
            stands_in: false,
            site: None,
        }
    }

    /// A tab as the session kept it (009): where it was and what it was
    /// called, asleep. `None` when what was kept is no address of the
    /// web's: that is an empty tab.
    fn restored(
        window: WindowId,
        space: SpaceId,
        engine: EngineKind,
        login_set: LoginSetId,
        saved: &SavedTab,
    ) -> Tab {
        let mut tab = Tab::new(window, space, engine, login_set);
        let web = |text: &str| {
            Address::parse(text)
                .ok()
                .filter(|address| matches!(address.scheme(), "http" | "https"))
        };
        // A Site comes back as one (039). A home that is no address of the
        // web's (a store that was meddled with) is no home.
        tab.site = web(&saved.home).map(|home| Site {
            home,
            name: saved.name.clone(),
        });
        let Some(address) = web(&saved.url) else {
            // A Site is never an empty tab: with nowhere kept, it is at home.
            return match tab.site.clone() {
                Some(site) => tab.asleep_at(&site),
                None => tab,
            };
        };
        tab.session_title =
            (!saved.title.is_empty()).then(|| (address.as_str().to_owned(), saved.title.clone()));
        tab.title.clone_from(&saved.title);
        tab.pending = Some(address);
        tab.asleep = true;
        tab
    }

    /// The tab as a Site that is asleep at its home (039): no page, no web
    /// view, nothing of what its page was doing; its home is where it will
    /// wake, and its name is what it is called until then.
    fn asleep_at(&self, site: &Site) -> Tab {
        let mut tab = Tab::new(self.window, self.space, self.engine, self.login_set);
        tab.session_title =
            (!site.name.is_empty()).then(|| (site.home.as_str().to_owned(), site.name.clone()));
        tab.title.clone_from(&site.name);
        tab.pending = Some(site.home.clone());
        tab.asleep = true;
        tab.site = Some(site.clone());
        tab
    }

    /// The page the tab is at: the one that owns its web view, or, for a tab
    /// that is asleep, the one it will wake at.
    fn page(&self) -> Option<&Address> {
        if self.asleep {
            self.pending.as_ref()
        } else {
            self.committed.as_ref()
        }
    }

    /// The tab as the session keeps it (009): its address when that is the
    /// web's, and the first title its page said.
    fn saved(&mut self, front: bool) -> SavedTab {
        let (home, name) = self.site.as_ref().map_or_else(Default::default, |site| {
            (site.home.as_str().to_owned(), site.name.clone())
        });
        // A Site that is not in front comes back asleep at its home (039):
        // a quit closes it, as closing it does.
        if !front && !home.is_empty() {
            return SavedTab {
                url: home.clone(),
                title: name.clone(),
                front,
                home,
                name,
            };
        }
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
                home,
                name,
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
        SavedTab {
            url,
            title,
            front,
            home,
            name,
        }
    }

    /// The user sent the tab to `address`.
    fn begin_user_load(&mut self, address: Address) {
        self.asleep = false;
        self.own_load = true;
        self.pending = Some(address);
        self.load = Load::Loading;
        self.progress = 0.0;
        self.error.clear();
    }

    /// The tab is sent where a page's link leads (018). Nobody typed that
    /// address: it is not shown as the tab's until its page commits, and
    /// what comes of it is not the person's own load.
    fn begin_link_load(&mut self) {
        self.load = Load::Loading;
        self.progress = 0.0;
        self.error.clear();
    }

    /// The tab's load failed with a code.
    fn fail(&mut self, code: &str) {
        self.load = Load::Failed;
        self.progress = 0.0;
        self.passable = None;
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
                self.passable = None;
            }
            Kind::NavigationCommitted(committed) => {
                // An address the engine reports but the parser refuses is
                // shown as no address at all rather than as the old one.
                self.committed = Address::parse(&committed.url).ok();
                // What was found went with the page it was found in, and so
                // did what the person would not let it use. (What it was
                // still asking is ended by the app, which tells the shell.)
                self.find = None;
                self.refused.clear();
                // A page came of what the person asked for: what the page
                // then starts is the page's.
                self.own_load = false;
                // And a new page has said nothing yet.
                self.spoke = false;
                self.silenced = false;
                self.had_page = true;
                self.pending = None;
                self.passable = None;
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
                    self.passable = None;
                } else {
                    // Nothing came of what the person asked for, and the
                    // page that is still there is not it.
                    self.own_load = false;
                    self.fail(&failed.code);
                    // It may be gone on with (026 §3) when the trouble was
                    // the site's certificate and the adapter holds what it
                    // refused.
                    self.passable = passable(&failed);
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
            | Kind::OutsideLinkRequested(_)
            | Kind::DownloadRequested(_)
            | Kind::DownloadProgress(_)
            | Kind::DownloadFinished(_)
            | Kind::DownloadFailed(_)
            | Kind::PermissionRequested(_)
            | Kind::PermissionWithdrawn(_)
            | Kind::DialogRequested(_)
            | Kind::SignInRequested(_)
            | Kind::LinkOpenRequested(_)
            | Kind::SearchRequested(_) => {}
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
            kept: self.site.is_some(),
            asleep: self.asleep,
        }
    }

    fn view(&self, id: TabId) -> TabView {
        let shown = self.shown();
        let mut address = shown.map_or_else(AddressView::default, address_view);
        // The mark beside the address speaks of a page that has arrived
        // (029). An address the tab was sent to that has not committed, one
        // a sleeping tab remembers, and a load that failed have no page to
        // speak of: beside an address whose certificate was just refused,
        // "secure" would be a lie (026 §3).
        if self.load == Load::Failed || self.pending.is_some() {
            address.security = address_view::Security::Unspecified as i32;
        }
        TabView {
            tab_id: id.get(),
            web_view_id: self.web_view.map_or(0, WebViewId::get),
            address: Some(address),
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
            prompt: self.prompts.front().map(Asking::view),
            certificate_passable: self.passable.is_some(),
            certificate_host: self
                .passable
                .as_ref()
                .and_then(Address::host)
                .unwrap_or_default()
                .to_owned(),
            kept: self.site.is_some(),
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

/// Where a failed load may be gone on to (026 §3): the address that failed,
/// when the trouble was its site's certificate, the adapter holds the
/// certificate it refused, and the address is a page's over `https`.
fn passable(failed: &NavigationFailed) -> Option<Address> {
    let certificate = matches!(
        failed.code.as_str(),
        "net.tls.untrusted" | "net.tls.date" | "net.tls.name"
    );
    if !certificate || !failed.certificate_held {
        return None;
    }
    Address::parse(&failed.url)
        .ok()
        .filter(|address| address.scheme() == "https" && address.origin().is_some())
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

/// A row of tabs: a Space's, in one window. In the order they are shown,
/// and the one in front. It always has a tab (005, T1).
#[derive(Debug)]
struct Row {
    tabs: Vec<TabId>,
    front: TabId,
}

/// How many of a row's tabs are Sites (039): they are the first of it, and
/// no other tab is put among them.
fn sites_in(tabs: &BTreeMap<TabId, Tab>, row: &Row) -> usize {
    row.tabs
        .iter()
        .filter(|id| tabs.get(*id).is_some_and(|tab| tab.site.is_some()))
        .count()
}

/// A way to go along a row of tabs (035).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// The next tab; from the last, the first.
    Next,
    /// The tab before; from the first, the last.
    Previous,
    /// The last tab.
    Last,
    /// The tab at a place, counted from one.
    Place(u32),
}

/// A window: the Space it shows, and the row of tabs of each Space it has
/// shown (024). The Space it shows has a row.
#[derive(Debug)]
struct Window {
    space: SpaceId,
    rows: BTreeMap<SpaceId, Row>,
}

impl Window {
    /// The row the window shows.
    fn shown(&self) -> Option<&Row> {
        self.rows.get(&self.space)
    }
}

/// The session as it now is (009, 024).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Session {
    /// The Space its window shows.
    pub(crate) shown: SpaceId,
    /// The tabs of each Space the window has shown in this run, in order.
    /// A Space it has not shown has what was kept of it.
    pub(crate) rows: BTreeMap<SpaceId, Vec<SavedTab>>,
}

/// Everything the app knows.
#[derive(Debug)]
pub(crate) struct State {
    /// The Spaces, in their order (024).
    spaces: Vec<Space>,
    /// The Space a window opens with, and its login set: the one the window
    /// with the session shows, or showed.
    start: (SpaceId, LoginSetId),
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
    /// The session (009): the tabs of each Space in its window as last
    /// worked out, which is what a window opened when there is none is
    /// given. `None` when nothing is kept between runs: fake mode.
    saved: Option<BTreeMap<SpaceId, Vec<SavedTab>>>,
    /// The window whose tabs are the session: the first opened, and the
    /// next one opened once that is closed.
    session_window: Option<WindowId>,
    /// How many downloads of each login set are coming (013), for the
    /// snapshots of its windows.
    downloads_active: BTreeMap<LoginSetId, u32>,
    /// The certificates a person went on past (026): each a login set and
    /// the origin whose certificate it was. Until the app quits, and kept
    /// nowhere else. A page of one is never shown as secure.
    exceptions: BTreeSet<(LoginSetId, String)>,
    /// How many closed tabs of each Space the journal has (033), for the
    /// snapshots of the windows that show it: whether there is something
    /// to undo. A Space that is not here has none.
    closed_tabs: BTreeMap<SpaceId, usize>,
    /// The rows of private windows (037): each a Space only in name, with a
    /// login set of its own, held here and in no store. It is in no list of
    /// Spaces, has no name and no colour, and goes with its window.
    private: Vec<Space>,
    /// How many private windows have been opened: each one's row is numbered
    /// from [`PRIVATE_SPACES_FROM`], far above any Space a store has.
    private_made: u64,
}

/// The number of the first private window's row, where a Space's number is
/// wanted: far above any a store gives a Space.
const PRIVATE_SPACES_FROM: u64 = 1 << 62;

impl State {
    /// A state with no windows. `spaces` are the Spaces in their order, and
    /// `start` the one of them a window opens with. `saved` is the session
    /// the store kept, each Space's tabs, to be given back to the first
    /// window; `None` keeps no session at all.
    pub(crate) fn new(
        spaces: Vec<Space>,
        start: &Space,
        engine: EngineKind,
        saved: Option<BTreeMap<SpaceId, Vec<SavedTab>>>,
    ) -> State {
        State {
            saved,
            session_window: None,
            downloads_active: BTreeMap::new(),
            exceptions: BTreeSet::new(),
            closed_tabs: BTreeMap::new(),
            private: Vec::new(),
            private_made: 0,
            spaces,
            start: (start.id, start.login_set),
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
    ///
    /// It shows the Space the window with the session shows, or showed
    /// (024).
    ///
    /// With `private`, the login set made for it, the window is a private
    /// one (037): it has that login set and a row of its own with one empty
    /// tab, it never has the session, and no window opened after it takes
    /// its place.
    pub(crate) fn open_window(&mut self, private: Option<LoginSetId>) -> (WindowId, TabId, bool) {
        let window = WindowId(self.ids.mint());
        let (space, login_set) = match private {
            Some(login_set) => {
                let space = SpaceId(PRIVATE_SPACES_FROM.saturating_add(self.private_made));
                self.private_made = self.private_made.saturating_add(1);
                self.private.push(Space {
                    id: space,
                    name: String::new(),
                    login_set,
                    colour: String::new(),
                });
                (space, login_set)
            }
            None => {
                let has_session = self
                    .session_window
                    .is_some_and(|session| self.windows.contains_key(&session));
                if self.saved.is_some() && !has_session {
                    self.session_window = Some(window);
                }
                self.start
            }
        };
        let (row, restored) = self.make_row(window, space, login_set);
        let front = row.front;
        let rows = BTreeMap::from([(space, row)]);
        self.windows.insert(window, Window { space, rows });
        self.dirty.insert(window);
        (window, front, restored)
    }

    /// The row of a Space in a window that has not shown it: the tabs the
    /// session kept of it, asleep, when the window is the one with the
    /// session (009); otherwise, and with none kept, one empty tab. The
    /// row, and whether its tabs came back.
    fn make_row(&mut self, window: WindowId, space: SpaceId, login_set: LoginSetId) -> (Row, bool) {
        let kept = match &self.saved {
            Some(saved) if self.session_window == Some(window) => {
                saved.get(&space).cloned().unwrap_or_default()
            }
            _ => Vec::new(),
        };
        let mut tabs = Vec::new();
        let mut front = None;
        for saved in &kept {
            let id = TabId(self.ids.mint());
            self.tabs.insert(
                id,
                Tab::restored(window, space, self.engine, login_set, saved),
            );
            if saved.front {
                front = Some(id);
            }
            tabs.push(id);
        }
        let restored = !tabs.is_empty();
        let first = match tabs.first() {
            Some(first) => *first,
            None => {
                let id = TabId(self.ids.mint());
                self.tabs
                    .insert(id, Tab::new(window, space, self.engine, login_set));
                tabs.push(id);
                id
            }
        };
        // A session with nobody in front (a store that was meddled with)
        // puts its first tab there: a row always has one.
        let front = front.unwrap_or(first);
        (Row { tabs, front }, restored)
    }

    /// A Space, or a private window's row, which is one in name only.
    fn space(&self, id: SpaceId) -> Option<&Space> {
        self.spaces
            .iter()
            .chain(&self.private)
            .find(|space| space.id == id)
    }

    /// The login set is a private window's (037): nothing of it is written,
    /// and the engine is to keep nothing of it.
    pub(crate) fn is_private(&self, login_set: LoginSetId) -> bool {
        self.private
            .iter()
            .any(|space| space.login_set == login_set)
    }

    /// The window is a private one (037).
    pub(crate) fn is_private_window(&self, window: WindowId) -> bool {
        self.windows
            .get(&window)
            .is_some_and(|window| self.is_private_space(window.space))
    }

    /// The login set of a private window; `None` for an ordinary window and
    /// for one there is not.
    pub(crate) fn private_login_set(&self, window: WindowId) -> Option<LoginSetId> {
        let window = self.windows.get(&window)?;
        self.private
            .iter()
            .find(|space| space.id == window.space)
            .map(|space| space.login_set)
    }

    /// The row is a private window's (037), where a Space's number is wanted.
    pub(crate) fn is_private_space(&self, space: SpaceId) -> bool {
        self.private.iter().any(|private| private.id == space)
    }

    /// The Spaces, in their order (024).
    pub(crate) fn spaces(&self) -> &[Space] {
        &self.spaces
    }

    /// A Space was made: it is after the others.
    pub(crate) fn space_made(&mut self, space: Space) {
        self.spaces.push(space);
    }

    /// A Space has a name now: the windows that show it say so. Whether
    /// there is such a Space.
    pub(crate) fn space_named(&mut self, id: SpaceId, name: &str) -> bool {
        let Some(space) = self.spaces.iter_mut().find(|space| space.id == id) else {
            return false;
        };
        if space.name != name {
            name.clone_into(&mut space.name);
            let showing = self
                .windows
                .iter()
                .filter(|(_, window)| window.space == id)
                .map(|(window, _)| *window);
            self.dirty.extend(showing);
        }
        true
    }

    /// A Space has a colour now, or none (025): the windows that show it say
    /// so. Whether there is such a Space.
    pub(crate) fn space_coloured(&mut self, id: SpaceId, colour: &str) -> bool {
        let Some(space) = self.spaces.iter_mut().find(|space| space.id == id) else {
            return false;
        };
        if space.colour != colour {
            colour.clone_into(&mut space.colour);
            let showing = self
                .windows
                .iter()
                .filter(|(_, window)| window.space == id)
                .map(|(window, _)| *window);
            self.dirty.extend(showing);
        }
        true
    }

    /// A window shows another Space (024 §3): that Space's row, as it was
    /// left, or as the session kept it, or one empty tab. The row of the
    /// Space it showed stays as it is. The Space the window shows already
    /// changes nothing. Refused with the code for a window, or a Space,
    /// that is not there.
    pub(crate) fn switch_space(
        &mut self,
        window_id: WindowId,
        space: SpaceId,
    ) -> Result<(), &'static str> {
        let window = self.windows.get(&window_id).ok_or("window.unknown")?;
        // A private window has no Space to switch (037); and its row is no
        // Space that a window could be switched to.
        if self.is_private_space(window.space) {
            return Err("window.private");
        }
        let login_set = self
            .spaces
            .iter()
            .find(|stored| stored.id == space)
            .ok_or("space.unknown")?
            .login_set;
        if window.space == space {
            return Ok(());
        }
        let made = if window.rows.contains_key(&space) {
            None
        } else {
            Some(self.make_row(window_id, space, login_set).0)
        };
        let window = self.windows.get_mut(&window_id).ok_or("window.unknown")?;
        if let Some(row) = made {
            window.rows.insert(space, row);
        }
        window.space = space;
        if self.session_window == Some(window_id) {
            self.start = (space, login_set);
        }
        self.dirty.insert(window_id);
        Ok(())
    }

    /// The tabs in front that are asleep, and where each was (009): each is
    /// to be loaded now, and is asleep no longer. The tab in front of what
    /// a window shows is never left asleep; one in front of a Space that is
    /// not shown sleeps on until it is.
    pub(crate) fn tabs_to_wake(&mut self) -> Vec<(TabId, Address)> {
        let fronts: Vec<TabId> = self
            .windows
            .values()
            .filter_map(|window| window.shown().map(|row| row.front))
            .collect();
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

    /// The session as it now is (009, 024): the Space its window shows,
    /// and the tabs, in order, of each Space the window has shown. `None`
    /// when there is nothing to write: no session is kept, or its window is
    /// closed, which closes none of its tabs.
    pub(crate) fn session(&mut self) -> Option<Session> {
        self.saved.as_ref()?;
        let window = self.windows.get(&self.session_window?)?;
        let shown = window.space;
        let listed: Vec<(SpaceId, TabId, Vec<TabId>)> = window
            .rows
            .iter()
            .map(|(space, row)| (*space, row.front, row.tabs.clone()))
            .collect();
        let mut rows = BTreeMap::new();
        for (space, front, ids) in listed {
            let tabs: Vec<SavedTab> = ids
                .into_iter()
                .filter_map(|id| Some(self.tabs.get_mut(&id)?.saved(id == front)))
                .collect();
            rows.insert(space, tabs);
        }
        if let Some(saved) = &mut self.saved {
            saved.extend(rows.clone());
        }
        Some(Session { shown, rows })
    }

    /// Closes a window. The web views to destroy, or `None` when there is no
    /// such window.
    pub(crate) fn close_window(&mut self, window: WindowId) -> Option<Vec<WebViewId>> {
        let closing = self.windows.remove(&window)?;
        let closing_space = closing.space;
        let mut web_views = Vec::new();
        // The tabs of every Space it has shown, not only the one it shows.
        for tab in closing.rows.into_values().flat_map(|row| row.tabs) {
            if let Some(web_view) = self.tabs.remove(&tab).and_then(|tab| tab.web_view) {
                self.web_views.remove(&web_view);
                web_views.push(web_view);
            }
        }
        self.dirty.remove(&window);
        self.closed.push(window);
        // A private window's row goes with it (037), and what was kept of
        // its login set here: the certificates gone on past, the count of
        // its downloads.
        if let Some(at) = self
            .private
            .iter()
            .position(|space| space.id == closing_space)
        {
            let login_set = self.private.remove(at).login_set;
            self.exceptions.retain(|(of, _)| *of != login_set);
            self.downloads_active.remove(&login_set);
        }
        Some(web_views)
    }

    /// The tab in front in a window, in the Space it shows. `None` when
    /// there is no such window.
    pub(crate) fn front_tab(&self, window_id: WindowId) -> Option<TabId> {
        self.windows
            .get(&window_id)
            .and_then(Window::shown)
            .map(|row| row.front)
    }

    /// Opens an empty tab in a window, in the Space it shows: after `after`
    /// when that is one of the tabs shown, at the end otherwise; in front
    /// unless `background`. `None` when there is no such window.
    pub(crate) fn open_tab(
        &mut self,
        window_id: WindowId,
        after: Option<TabId>,
        background: bool,
    ) -> Option<TabId> {
        let window = self.windows.get_mut(&window_id)?;
        let space = window.space;
        // The Space the window shows; or, for a private window, its own row.
        let login_set = self
            .spaces
            .iter()
            .chain(&self.private)
            .find(|s| s.id == space)?
            .login_set;
        let row = window.rows.get_mut(&space)?;
        let tab = TabId(self.ids.mint());
        let place = after
            .and_then(|after| row.tabs.iter().position(|id| *id == after))
            .map_or(row.tabs.len(), |index| index + 1)
            .max(sites_in(&self.tabs, row));
        row.tabs.insert(place, tab);
        if !background {
            row.front = tab;
        }
        self.tabs
            .insert(tab, Tab::new(window_id, space, self.engine, login_set));
        self.dirty.insert(window_id);
        Some(tab)
    }

    /// Closes a tab. Its web view to destroy, if it had one; or `None` when
    /// there is no such tab.
    ///
    /// When the tab was in front, the one after it comes forward, or the one
    /// before it when it was the last. A row always has a tab: an empty one
    /// takes the place of its only tab, in the Space that one was in.
    pub(crate) fn close_tab(&mut self, tab_id: TabId) -> Option<Option<WebViewId>> {
        if self.tabs.get(&tab_id)?.site.is_some() {
            return self.sleep_site(tab_id);
        }
        let tab = self.tabs.remove(&tab_id)?;
        if let Some(web_view) = tab.web_view {
            self.web_views.remove(&web_view);
        }
        let row = self
            .windows
            .get_mut(&tab.window)
            .and_then(|window| window.rows.get_mut(&tab.space));
        if let Some(row) = row {
            let index = row.tabs.iter().position(|id| *id == tab_id);
            row.tabs.retain(|id| *id != tab_id);
            if row.tabs.is_empty() {
                let fresh = TabId(self.ids.mint());
                let mut stand_in = Tab::new(tab.window, tab.space, self.engine, tab.login_set);
                stand_in.stands_in = true;
                self.tabs.insert(fresh, stand_in);
                row.tabs.push(fresh);
                row.front = fresh;
            } else if row.front == tab_id {
                // Back to the tab whose page opened it, when that is still
                // here. Otherwise what was after it, which is now where it
                // was; past the end, the last.
                let opener = tab.opener.filter(|opener| row.tabs.contains(opener));
                let next = index.unwrap_or(0).min(row.tabs.len() - 1);
                if let Some(next) = opener.or_else(|| row.tabs.get(next).copied()) {
                    row.front = next;
                }
            }
            self.dirty.insert(tab.window);
        }
        Some(tab.web_view)
    }

    /// Closing a Site puts it to sleep (039): its page and its web view go,
    /// and it stays in its row, at its home and by its name. Its web view to
    /// destroy, if it had one.
    ///
    /// When it was in front, the tab that comes forward is the nearest that
    /// is awake or is no Site, the one after before the one before: closing
    /// a Site does not wake another. With none, an empty tab is opened at
    /// the end, as when a row's last tab is closed.
    fn sleep_site(&mut self, tab_id: TabId) -> Option<Option<WebViewId>> {
        let tab = self.tabs.get(&tab_id)?;
        let site = tab.site.clone()?;
        let (window, space, login_set, web_view) =
            (tab.window, tab.space, tab.login_set, tab.web_view);
        let asleep = tab.asleep_at(&site);
        if let Some(web_view) = web_view {
            self.web_views.remove(&web_view);
        }
        self.tabs.insert(tab_id, asleep);
        let row = self.windows.get(&window)?.rows.get(&space)?;
        if row.front == tab_id {
            let index = row.tabs.iter().position(|id| *id == tab_id).unwrap_or(0);
            let comes_forward = |id: &&TabId| {
                self.tabs
                    .get(*id)
                    .is_some_and(|tab| tab.site.is_none() || !tab.asleep)
            };
            let next = row.tabs[index + 1..]
                .iter()
                .find(comes_forward)
                .or_else(|| row.tabs[..index].iter().rev().find(comes_forward))
                .copied();
            let next = next.unwrap_or_else(|| {
                let fresh = TabId(self.ids.mint());
                self.tabs
                    .insert(fresh, Tab::new(window, space, self.engine, login_set));
                fresh
            });
            let row = self.windows.get_mut(&window)?.rows.get_mut(&space)?;
            if !row.tabs.contains(&next) {
                row.tabs.push(next);
            }
            row.front = next;
        }
        self.dirty.insert(window);
        Some(web_view)
    }

    /// Keeps a tab (039): it becomes a Site, whose home is the page it is
    /// at and whose name is that page's title, and moves to the end of its
    /// row's Sites. A tab that is a Site already stays as it is. The code
    /// to refuse with, for a tab there is not, one with no page of the
    /// web's, and one of a private window, of which nothing is kept.
    pub(crate) fn keep_tab(&mut self, tab_id: TabId) -> Result<(), &'static str> {
        let tab = self.tabs.get(&tab_id).ok_or("tab.unknown")?;
        if self.is_private_space(tab.space) {
            return Err("window.private");
        }
        if tab.site.is_some() {
            return Ok(());
        }
        let home = tab
            .page()
            .filter(|address| matches!(address.scheme(), "http" | "https"))
            .cloned()
            .ok_or("tab.no_page")?;
        let (window, space, name) = (tab.window, tab.space, tab.title.clone());
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.site = Some(Site { home, name });
        }
        self.place_among(window, space, tab_id);
        Ok(())
    }

    /// Lets a Site go (039): it is a tab like any other again, the first of
    /// its row's other tabs, as it is. Whether there is such a tab; one that
    /// is no Site stays where it is.
    pub(crate) fn let_go_site(&mut self, tab_id: TabId) -> bool {
        let Some(tab) = self.tabs.get_mut(&tab_id) else {
            return false;
        };
        if tab.site.take().is_some() {
            let (window, space) = (tab.window, tab.space);
            self.place_among(window, space, tab_id);
        }
        true
    }

    /// Puts a tab that has just become a Site, or has just stopped being
    /// one, where the row's order has it: after the row's Sites. For a new
    /// Site that is the end of the Sites; for a tab let go, the start of the
    /// other tabs.
    fn place_among(&mut self, window: WindowId, space: SpaceId, tab_id: TabId) {
        let Some(row) = self
            .windows
            .get_mut(&window)
            .and_then(|window| window.rows.get_mut(&space))
        else {
            return;
        };
        row.tabs.retain(|id| *id != tab_id);
        let sites = row
            .tabs
            .iter()
            .filter(|id| self.tabs.get(*id).is_some_and(|tab| tab.site.is_some()))
            .count();
        row.tabs.insert(sites.min(row.tabs.len()), tab_id);
        self.dirty.insert(window);
    }

    /// Brings a tab in front in its row: among its Space's tabs in its
    /// window. Whether there is such a tab.
    pub(crate) fn activate_tab(&mut self, tab_id: TabId) -> bool {
        let Some(tab) = self.tabs.get(&tab_id) else {
            return false;
        };
        let row = self
            .windows
            .get_mut(&tab.window)
            .and_then(|window| window.rows.get_mut(&tab.space));
        if let Some(row) = row
            && row.front != tab_id
        {
            row.front = tab_id;
            self.dirty.insert(tab.window);
        }
        true
    }

    /// The tab a choice made while typing leaves behind, when it is to go
    /// (038): the tab in front of the row of `chosen`, when that is another
    /// tab and an empty one. Empty as a row's stand-in is (033): it shows no
    /// address and has no web view. It was opened to go somewhere, and the
    /// person has gone to a tab that was open. `None` otherwise.
    pub(crate) fn empty_tab_left_for(&self, chosen: TabId) -> Option<TabId> {
        let tab = self.tabs.get(&chosen)?;
        let row = self.windows.get(&tab.window)?.rows.get(&tab.space)?;
        let front = self.tabs.get(&row.front)?;
        (row.front != chosen && front.shown().is_none() && front.web_view.is_none())
            .then_some(row.front)
    }

    /// The tab a step along the row leads to (035), in the row the window
    /// shows: the tab in front itself when the step leads nowhere (one tab,
    /// a place the row does not reach). `None` when there is no such
    /// window.
    pub(crate) fn tab_at_step(&self, window_id: WindowId, step: Step) -> Option<TabId> {
        let window = self.windows.get(&window_id)?;
        let row = window.rows.get(&window.space)?;
        let last = row.tabs.len().checked_sub(1)?;
        let front = row.tabs.iter().position(|tab| *tab == row.front)?;
        let to = match step {
            Step::Next if front == last => 0,
            Step::Next => front + 1,
            Step::Previous if front == 0 => last,
            Step::Previous => front - 1,
            Step::Last => last,
            // Places are counted from one; there is no place nought.
            Step::Place(place) => match usize::try_from(place).ok()?.checked_sub(1) {
                Some(at) if at <= last => at,
                _ => front,
            },
        };
        row.tabs.get(to).copied()
    }

    /// A tab is sent to an address: by the user, or (with no address of
    /// the user's) where a page's link leads. Says which web view loads it
    /// and whether that web view is new, or `None` when there is no such
    /// tab.
    pub(crate) fn begin_load(
        &mut self,
        tab_id: TabId,
        users: Option<Address>,
    ) -> Option<LoadTarget> {
        let tab = self.tabs.get_mut(&tab_id)?;
        match users {
            Some(address) => tab.begin_user_load(address),
            None => tab.begin_link_load(),
        }
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

    /// The login set of the tab a web view is in (013): whose download one
    /// that its page asks for is. `None` for a web view no tab has.
    pub(crate) fn login_set_of(&self, web_view: WebViewId) -> Option<LoginSetId> {
        self.tab_of(web_view).map(|tab| tab.login_set)
    }

    /// Who a web view's page is to a request for permission (014 §3).
    /// `None` when it may not ask at all: the web view is no tab's, the tab
    /// has no page, the page has no origin of the web's, or what arrives
    /// from it cannot be taken to be its own.
    pub(crate) fn asker(&self, web_view: WebViewId) -> Option<Asker> {
        let tab = self.tab_of(web_view)?;
        let page = tab
            .committed
            .as_ref()
            .filter(|page| page.is_trustworthy())?;
        let origin = page.origin()?;
        // The origin as an address field shows it: the host, and the scheme
        // when it is not `https`.
        let site = Address::parse(&origin).map_or_else(|_| origin.clone(), |named| named.display());
        Some(Asker {
            login_set: tab.login_set,
            origin,
            site,
        })
    }

    /// Who asks for a download in a web view, of the file at `url` (015
    /// §3). `None` for a web view no tab has. Said once: if the person asked
    /// for it themselves, that is used up by saying so.
    pub(crate) fn downloader(&mut self, web_view: WebViewId, url: &str) -> Option<Downloader> {
        let tab = self.tab_of_mut(web_view)?;
        let own = std::mem::take(&mut tab.own_load);
        // The page that asks, wherever the file comes from; with no page,
        // where the file comes from.
        let from = Address::parse(url).ok();
        let named = tab
            .committed
            .as_ref()
            .filter(|page| page.origin().is_some())
            .or(from.as_ref());
        let origin = named.and_then(Address::origin);
        let site = match &origin {
            Some(origin) => {
                Address::parse(origin).map_or_else(|_| origin.clone(), |site| site.display())
            }
            // A page that is not the web's, with a file that is not either:
            // named by what kind of address it is, and no more.
            None => named.map_or_else(String::new, |address| address.scheme().to_owned()),
        };
        Some(Downloader {
            login_set: tab.login_set,
            origin,
            site,
            own,
        })
    }

    /// Who says something in a web view's page (016 §3). `None` when the
    /// web view is no tab's, or its tab has no page of the web's.
    pub(crate) fn speaker(&self, web_view: WebViewId) -> Option<Speaker> {
        let tab = self.tab_of(web_view)?;
        let origin = tab.committed.as_ref()?.origin()?;
        let site = Address::parse(&origin).map_or_else(|_| origin.clone(), |named| named.display());
        Some(Speaker {
            login_set: tab.login_set,
            origin,
            site,
            silenced: tab.silenced,
            spoke_before: tab.spoke,
        })
    }

    /// The server that asks who is there in a web view (017 §3), when it is
    /// the tab's own: the one its load is heading for, or its page's. By
    /// its origin where the engine names one, and by its host where it
    /// names no more: the server is then taken to be at the scheme and port
    /// of the address that matched.
    pub(crate) fn server(
        &self,
        web_view: WebViewId,
        host: &str,
        origin: &str,
        heading: &str,
    ) -> Option<Asker> {
        let tab = self.tab_of(web_view)?;
        let heading = Address::parse(heading).ok();
        let named = Address::parse(origin)
            .ok()
            .and_then(|server| server.origin());
        // No origin named: the host alone. (An origin that cannot be read is
        // no server at all, whatever host is beside it.)
        let by_host = origin.is_empty();
        let origin = [heading.as_ref(), tab.committed.as_ref()]
            .into_iter()
            .flatten()
            .find_map(|own| {
                let theirs = own.origin()?;
                let same = match &named {
                    Some(named) => *named == theirs,
                    None => by_host && own.host().is_some_and(|own| own.eq_ignore_ascii_case(host)),
                };
                same.then_some(theirs)
            })?;
        let site = Address::parse(&origin).map_or_else(|_| origin.clone(), |named| named.display());
        Some(Asker {
            login_set: tab.login_set,
            origin,
            site,
        })
    }

    /// A server's question has its answer: every other of the same server
    /// and realm that waits in the login set is taken away, in whatever
    /// tab (017 §3). The shell's names for them.
    pub(crate) fn settle_signed_in(
        &mut self,
        login_set: LoginSetId,
        origin: &str,
        realm: &str,
    ) -> Vec<Settled> {
        let mut settled = Vec::new();
        for tab in self.tabs.values_mut() {
            if tab.login_set != login_set {
                continue;
            }
            let before = tab.prompts.len();
            tab.prompts.retain(|asking| {
                let same = asking.origin == origin && asking.sign_in() == Some(realm);
                if same {
                    settled.push(asking.settled());
                }
                !same
            });
            if tab.prompts.len() != before {
                self.dirty.insert(tab.window);
            }
        }
        settled
    }

    /// A web view's load is over, or another began: the servers that asked
    /// who is there in it wait no more. The shell's names for their
    /// questions.
    pub(crate) fn end_sign_ins(&mut self, web_view: WebViewId) -> Vec<Settled> {
        let Some(tab) = self.tab_of_mut(web_view) else {
            return Vec::new();
        };
        let mut ended = Vec::new();
        tab.prompts.retain(|asking| {
            let asked = asking.waited() == Waited::SignIn;
            if asked {
                ended.push(asking.settled());
            }
            !asked
        });
        if !ended.is_empty() {
            let window = tab.window;
            self.dirty.insert(window);
        }
        ended
    }

    /// A web view's page has now said something: its next can be stopped.
    pub(crate) fn spoke(&mut self, web_view: WebViewId) {
        if let Some(tab) = self.tab_of_mut(web_view) {
            tab.spoke = true;
        }
    }

    /// The person stopped a tab's page's messages: it says no more until
    /// another page, and what it was saying that waited is taken away. The
    /// shell's names for those.
    pub(crate) fn silence(&mut self, tab_id: TabId) -> Vec<Settled> {
        let Some(tab) = self.tabs.get_mut(&tab_id) else {
            return Vec::new();
        };
        tab.silenced = true;
        let mut settled = Vec::new();
        tab.prompts.retain(|asking| {
            let said = asking.waited() == Waited::Dialog;
            if said {
                settled.push(asking.settled());
            }
            !said
        });
        settled
    }

    /// Whether a request by that name of the shell's, among its kind, waits
    /// in some tab.
    pub(crate) fn is_asking(&self, request: u64, waited: Waited) -> bool {
        self.tabs.values().any(|tab| {
            tab.prompts
                .iter()
                .any(|asking| asking.request == request && asking.waited() == waited)
        })
    }

    /// Whether the person would not let the web view's page use one of
    /// these already.
    pub(crate) fn refused_here(&self, web_view: WebViewId, kinds: &[PermissionKind]) -> bool {
        self.tab_of(web_view)
            .is_some_and(|tab| kinds.iter().any(|kind| tab.refused.contains(kind)))
    }

    /// Puts something before the person: the tab's prompt, or behind the
    /// ones it has. False when the tab has as many as it may.
    pub(crate) fn ask(
        &mut self,
        web_view: WebViewId,
        request: u64,
        asker: Asker,
        kinds: Vec<PermissionKind>,
        what: AskedFor,
    ) -> bool {
        let prompt = self.ids.mint();
        let Some(tab) = self.tab_of_mut(web_view) else {
            return false;
        };
        if tab.prompts.len() >= MOST_PROMPTS {
            return false;
        }
        tab.prompts.push_back(Asking {
            prompt,
            request,
            origin: asker.origin,
            site: asker.site,
            kinds,
            what,
        });
        let window = tab.window;
        self.dirty.insert(window);
        true
    }

    /// The page no longer waits for an answer to a request of its web
    /// view's: it is asked about no more.
    pub(crate) fn withdraw(&mut self, web_view: WebViewId, request: u64) {
        let Some(tab) = self.tab_of_mut(web_view) else {
            return;
        };
        let before = tab.prompts.len();
        tab.prompts
            .retain(|asking| asking.request != request || asking.waited() != Waited::Permission);
        if tab.prompts.len() != before {
            let window = tab.window;
            self.dirty.insert(window);
        }
    }

    /// The window a tab is in. `None` for no such tab.
    pub(crate) fn window_of(&self, tab_id: TabId) -> Option<WindowId> {
        self.tabs.get(&tab_id).map(|tab| tab.window)
    }

    /// The person answered the prompt a tab shows: it is taken away, and
    /// given back with the tab's login set. `Err(false)` for no such tab,
    /// `Err(true)` when the tab does not show that prompt. A refusal is
    /// kept against the page.
    pub(crate) fn answer_prompt(
        &mut self,
        tab_id: TabId,
        prompt: u64,
        allow: bool,
    ) -> Result<(LoginSetId, Asking), bool> {
        let tab = self.tabs.get_mut(&tab_id).ok_or(false)?;
        if tab.prompts.front().map(|asking| asking.prompt) != Some(prompt) {
            return Err(true);
        }
        let asking = tab.prompts.pop_front().ok_or(true)?;
        if !allow {
            for kind in &asking.kinds {
                if !tab.refused.contains(kind) {
                    tab.refused.push(*kind);
                }
            }
        }
        let window = tab.window;
        self.dirty.insert(window);
        Ok((tab.login_set, asking))
    }

    /// An origin may now use `allowed` in a login set: every request of
    /// its pages that waited for no more than that is taken away, in
    /// whatever tab. The shell's names for them.
    pub(crate) fn settle_allowed(
        &mut self,
        login_set: LoginSetId,
        origin: &str,
        allowed: &[PermissionKind],
    ) -> Vec<Settled> {
        let mut settled = Vec::new();
        for tab in self.tabs.values_mut() {
            if tab.login_set != login_set {
                continue;
            }
            let before = tab.prompts.len();
            tab.prompts.retain(|asking| {
                // What a page says wants nothing that can be allowed: it
                // waits for its own answer.
                let all = asking.origin == origin
                    && !asking.kinds.is_empty()
                    && asking.kinds.iter().all(|kind| allowed.contains(kind));
                if all {
                    settled.push(asking.settled());
                }
                !all
            });
            if tab.prompts.len() != before {
                self.dirty.insert(tab.window);
            }
        }
        settled
    }

    /// What a tab's page was just refused, it is refused wherever else it
    /// waits in that tab. The shell's names for those requests.
    pub(crate) fn settle_refused(&mut self, tab_id: TabId) -> Vec<Settled> {
        let Some(tab) = self.tabs.get_mut(&tab_id) else {
            return Vec::new();
        };
        let mut settled = Vec::new();
        let refused = tab.refused.clone();
        tab.prompts.retain(|asking| {
            let any = asking.kinds.iter().any(|kind| refused.contains(kind));
            if any {
                settled.push(asking.settled());
            }
            !any
        });
        settled
    }

    /// The page a web view showed is gone: whatever it was asking is over.
    /// The shell's names for the requests that waited.
    pub(crate) fn end_prompts(&mut self, web_view: WebViewId) -> Vec<Settled> {
        let Some(tab) = self.tab_of_mut(web_view) else {
            return Vec::new();
        };
        if tab.prompts.is_empty() {
            return Vec::new();
        }
        let ended = tab
            .prompts
            .drain(..)
            .map(|asking| asking.settled())
            .collect();
        let window = tab.window;
        self.dirty.insert(window);
        ended
    }

    /// The journal has so many closed tabs of a Space now (033): the
    /// snapshots of the windows that show it say whether there is one to
    /// bring back.
    pub(crate) fn set_closed_tabs(&mut self, space: SpaceId, count: usize) {
        let before = self.closed_tabs.insert(space, count).unwrap_or(0);
        if (before == 0) != (count == 0) {
            let showing: Vec<WindowId> = self
                .windows
                .iter()
                .filter(|(_, window)| window.space == space)
                .map(|(window, _)| *window)
                .collect();
            self.dirty.extend(showing);
        }
    }

    /// The Space a window shows.
    pub(crate) fn space_shown(&self, window: WindowId) -> Option<SpaceId> {
        self.windows.get(&window).map(|window| window.space)
    }

    /// The Spaces whose logins are `login_set`.
    pub(crate) fn spaces_of(&self, login_set: LoginSetId) -> Vec<SpaceId> {
        self.spaces
            .iter()
            .filter(|space| space.login_set == login_set)
            .map(|space| space.id)
            .collect()
    }

    /// What the journal is to keep of a tab the person is closing (033):
    /// its Space, its place in the row, and its address and title as the
    /// session keeps them (009). `None` for a tab there is not, and for one
    /// with no address of the web's: an empty tab is nothing to bring back.
    pub(crate) fn closing(&mut self, tab_id: TabId) -> Option<(SpaceId, u32, SavedTab)> {
        let (window, space) = self.tabs.get(&tab_id).map(|tab| (tab.window, tab.space))?;
        // Nothing of a private window is written (037): its closed tab is
        // not journalled, and is not brought back.
        if self.is_private_space(space) {
            return None;
        }
        // A Site that is closed is asleep, and still there (039): there is
        // nothing to bring back.
        if self.tabs.get(&tab_id)?.site.is_some() {
            return None;
        }
        let saved = self.tabs.get_mut(&tab_id)?.saved(false);
        if saved.url.is_empty() {
            return None;
        }
        let row = self.windows.get(&window)?.rows.get(&space)?;
        let place = row.tabs.iter().position(|id| *id == tab_id)?;
        Some((space, u32::try_from(place).ok()?, saved))
    }

    /// Brings a closed tab back into the row of the Space a window shows
    /// (033): at the place it had, or at the end when the row is shorter
    /// now; in front; asleep at its address, with the title it had, as a
    /// tab of the session comes back (009), so that waking it loads it.
    /// When the row's only tab is the empty one that was put there when the
    /// row's last tab was closed, the tab takes its place. An empty tab the
    /// person opened stays. `None` when there is no such window.
    pub(crate) fn reopen(
        &mut self,
        window_id: WindowId,
        place: u32,
        saved: &SavedTab,
    ) -> Option<TabId> {
        let window = self.windows.get_mut(&window_id)?;
        let space = window.space;
        let login_set = self.spaces.iter().find(|s| s.id == space)?.login_set;
        let row = window.rows.get_mut(&space)?;
        let only_a_stand_in = match row.tabs.as_slice() {
            [only] => self.tabs.get(only).is_some_and(|tab| {
                tab.stands_in && tab.shown().is_none() && tab.web_view.is_none()
            }),
            _ => false,
        };
        if only_a_stand_in {
            for gone in row.tabs.drain(..) {
                self.tabs.remove(&gone);
            }
        }
        let tab = TabId(self.ids.mint());
        let place = usize::try_from(place)
            .map_or(row.tabs.len(), |place| place.min(row.tabs.len()))
            .max(sites_in(&self.tabs, row));
        row.tabs.insert(place, tab);
        row.front = tab;
        self.tabs.insert(
            tab,
            Tab::restored(window_id, space, self.engine, login_set, saved),
        );
        self.dirty.insert(window_id);
        Some(tab)
    }

    /// So many downloads of a login set are coming now: the snapshots of
    /// the windows that show a Space of it say so.
    pub(crate) fn set_downloads_active(&mut self, login_set: LoginSetId, coming: u32) {
        let before = self.downloads_active.insert(login_set, coming).unwrap_or(0);
        if before != coming {
            let showing: Vec<WindowId> = self
                .windows
                .iter()
                .filter(|(_, window)| self.login_set_shown(window) == Some(login_set))
                .map(|(window, _)| *window)
                .collect();
            self.dirty.extend(showing);
        }
    }

    /// The login set of the Space a window shows.
    fn login_set_shown(&self, window: &Window) -> Option<LoginSetId> {
        self.space(window.space).map(|space| space.login_set)
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

    /// The person asked for a link of a web view's page in a new tab (018).
    /// One is opened in the window of the tab that has the link, in its
    /// Space and login set, remembering where it was opened from: after that tab and
    /// whatever it opened before that still follows it, so that links
    /// opened one after another stand in that order; behind, or in front.
    /// The new tab, or `None` when the web view is no tab's.
    pub(crate) fn open_link_tab(&mut self, asking: WebViewId, background: bool) -> Option<TabId> {
        let opener_id = *self.web_views.get(&asking)?;
        let opener = self.tabs.get(&opener_id)?;
        let window_id = opener.window;
        let space = opener.space;
        let mut tab = Tab::new(window_id, space, opener.engine, opener.login_set);
        tab.opener = Some(opener_id);
        let after = self.windows.get(&window_id)?.rows.get(&space)?.tabs.clone();
        let mut place = after
            .iter()
            .position(|id| *id == opener_id)
            .map_or(after.len(), |index| index + 1);
        while after
            .get(place)
            .and_then(|id| self.tabs.get(id))
            .is_some_and(|next| next.opener == Some(opener_id))
        {
            place += 1;
        }
        let row = self.windows.get_mut(&window_id)?.rows.get_mut(&space)?;
        let tab_id = TabId(self.ids.mint());
        let place = place.max(sites_in(&self.tabs, row));
        row.tabs.insert(place, tab_id);
        if !background {
            row.front = tab_id;
        }
        self.tabs.insert(tab_id, tab);
        self.dirty.insert(window_id);
        Some(tab_id)
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
        let space = opener.space;
        let mut tab = Tab::new(window_id, space, opener.engine, opener.login_set);
        tab.web_view = Some(new);
        tab.opener = Some(opener_id);
        tab.opened_by_page = true;
        let row = self
            .windows
            .get_mut(&window_id)
            .and_then(|window| window.rows.get_mut(&space));
        let Some(row) = row else {
            return refused;
        };
        let tab_id = TabId(self.ids.mint());
        let place = row
            .tabs
            .iter()
            .position(|id| *id == opener_id)
            .map_or(row.tabs.len(), |index| index + 1)
            .max(sites_in(&self.tabs, row));
        row.tabs.insert(place, tab_id);
        row.front = tab_id;
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

    /// Where a tab's failed load may be gone on to (026 §3): its web view,
    /// the address that failed, and the tab's login set. `Err(false)` for no
    /// such tab; `Err(true)` for a tab whose failed load is not one that may
    /// be gone on with.
    pub(crate) fn passable(&self, tab: TabId) -> Result<(WebViewId, Address, LoginSetId), bool> {
        let tab = self.tabs.get(&tab).ok_or(false)?;
        let address = tab.passable.clone().ok_or(true)?;
        let web_view = tab.web_view.ok_or(true)?;
        Ok((web_view, address, tab.login_set))
    }

    /// A person went on past the certificate of `origin` in a login set
    /// (026): its pages are marked not secure from now, in every window.
    pub(crate) fn except(&mut self, login_set: LoginSetId, origin: String) {
        if self.exceptions.insert((login_set, origin)) {
            self.dirty.extend(self.windows.keys().copied());
        }
    }

    /// There is no such exception after all: the adapter held nothing to
    /// let through.
    pub(crate) fn unexcept(&mut self, login_set: LoginSetId, origin: &str) {
        if self.exceptions.remove(&(login_set, origin.to_owned())) {
            self.dirty.extend(self.windows.keys().copied());
        }
    }

    /// The login set a window's history is: that of the Space it shows.
    pub(crate) fn history_of(&self, window: WindowId) -> Option<LoginSetId> {
        self.login_set_shown(self.windows.get(&window)?)
    }

    /// The open tabs to offer for what is typed in a window's address field
    /// (038): those of the row the window shows whose page has every word
    /// typed in its title or its address, in the row's order, at most
    /// `limit`. Never the tab in front, which is where the person is; nor a
    /// tab that shows no page of the web's, which has nothing to find. Each
    /// with its page's address and title. `None` when there is no such
    /// window.
    pub(crate) fn tabs_offered(
        &self,
        window: WindowId,
        text: &str,
        limit: usize,
    ) -> Option<Vec<(TabId, Address, String)>> {
        let window = self.windows.get(&window)?;
        let typed = typed_words(text);
        let Some(row) = window.rows.get(&window.space) else {
            return Some(Vec::new());
        };
        let offered = row
            .tabs
            .iter()
            .filter(|id| **id != row.front)
            .filter_map(|id| {
                let tab = self.tabs.get(id)?;
                let address = tab.page()?;
                (matches!(address.scheme(), "http" | "https")
                    && all_found(&typed, &[&tab.title, address.as_str()]))
                .then(|| (*id, address.clone(), tab.title.clone()))
            })
            .take(limit)
            .collect();
        Some(offered)
    }

    /// The visit to write for the page a web view now shows (008): its login
    /// set and address. `None` when the page is not a visit: it has no
    /// address of the web's, or it is the visit the tab is already at (the
    /// same address again, straight after).
    pub(crate) fn visit_to_record(&self, web_view: WebViewId) -> Option<(LoginSetId, String)> {
        let tab = self.tab_of(web_view)?;
        // Nothing a private window visits is written (037).
        if self.is_private(tab.login_set) {
            return None;
        }
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
        let space = self.space(window.space)?;
        let private = self.is_private_space(space.id);
        let row = window.shown()?;
        let front = self.tabs.get(&row.front)?;
        let mut view = front.view(row.front);
        // A page whose certificate the person went on past is never shown as
        // secure (026 §3), whatever its address says.
        let excepted = front
            .shown()
            .and_then(Address::origin)
            .is_some_and(|origin| self.exceptions.contains(&(front.login_set, origin)));
        if excepted && let Some(address) = &mut view.address {
            address.security = address_view::Security::Insecure as i32;
        }
        Some(WindowSnapshot {
            window_id: window_id.get(),
            tab: Some(view),
            tabs: row
                .tabs
                .iter()
                .filter_map(|id| self.tabs.get(id).map(|tab| tab.summary(*id)))
                .collect(),
            downloads_active: self
                .downloads_active
                .get(&space.login_set)
                .copied()
                .unwrap_or(0),
            // A private window shows no Space (037): none is named.
            space_id: if private { 0 } else { space.id.get() },
            space_name: space.name.clone(),
            space_colour: space.colour.clone(),
            // What an undo would undo here (033): a closed tab, or nothing.
            undo: if self.closed_tabs.get(&space.id).copied().unwrap_or(0) > 0 {
                UNDO_CLOSED_TAB.to_owned()
            } else {
                String::new()
            },
            private_window: private,
        })
    }

    pub(crate) fn take_dirty(&mut self) -> BTreeSet<WindowId> {
        std::mem::take(&mut self.dirty)
    }

    pub(crate) fn take_closed(&mut self) -> Vec<WindowId> {
        std::mem::take(&mut self.closed)
    }
}
