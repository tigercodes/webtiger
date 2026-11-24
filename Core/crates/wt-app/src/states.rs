//! A tab's kept state (051): the engine's own state of a web view, asked
//! for when its tab leaves the front, kept in the store, and handed back
//! when the tab wakes. To the core it is bytes: it never reads them.

use std::time::Duration;

use wt_core::{TabId, WebViewId};
use wt_proto::v1::{
    CoreConfig, DeviceKind, MemoryLevel, WebViewDestroy, WebViewSaveState, host_request,
    host_response,
};

use crate::app::App;
use crate::model::Sleeper;

/// The most bytes of a state that is kept (051 D3). A larger one is not
/// kept: its tab wakes at its address.
pub(crate) const MOST_STATE: usize = 512 * 1024;

/// How long a tab is out of sight before it is put to sleep (053 D1): thirty
/// of the core's looks, one a minute.
const SLEEP_AFTER_LOOKS: u32 = 30;
const SLEEP_LOOK_EVERY: Duration = Duration::from_secs(60);

/// The most tabs a phone has alive at once, and a tablet (054 D1).
const LIVE_TABS_PHONE: usize = 3;
const LIVE_TABS_TABLET: usize = 6;
/// On a desktop (054 D2): one for each so many megabytes of the machine's
/// memory, and no fewer than so many.
const MEGABYTES_A_LIVE_TAB: u32 = 512;
const LIVE_TABS_AT_LEAST: usize = 8;

/// The most tabs that are alive at once (054), from what the shell says of
/// the device. A shell that does not say what it runs on is taken for a
/// desktop; one that does not say its memory, for one with none to speak
/// of. Fake mode has no limit of its own: only the one a suite gives.
pub(crate) fn live_tabs(config: &CoreConfig, fake: bool) -> usize {
    if fake {
        let given = config.fake.as_ref().map_or(0, |options| options.live_tabs);
        return match given {
            0 => usize::MAX,
            given => usize::try_from(given).unwrap_or(usize::MAX),
        };
    }
    match DeviceKind::try_from(config.device_kind) {
        Ok(DeviceKind::Phone) => LIVE_TABS_PHONE,
        Ok(DeviceKind::Tablet) => LIVE_TABS_TABLET,
        Ok(DeviceKind::Desktop | DeviceKind::Unspecified) | Err(_) => {
            let by_memory = config.memory_megabytes / MEGABYTES_A_LIVE_TAB;
            usize::try_from(by_memory)
                .unwrap_or(usize::MAX)
                .max(LIVE_TABS_AT_LEAST)
        }
    }
}

/// The sleep time as the app counts it: so many looks, so far apart. With a
/// time in seconds (fake mode, for a suite that must see a tab sleep), a
/// look a second, and as many looks as seconds.
pub(crate) fn sleep_time(fake_seconds: u32) -> (u32, Duration) {
    if fake_seconds == 0 {
        (SLEEP_AFTER_LOOKS, SLEEP_LOOK_EVERY)
    } else {
        (fake_seconds, Duration::from_secs(1))
    }
}

impl App {
    /// Whether some tab is out of sight with a web view (053): there is
    /// something for the look to count.
    #[must_use]
    pub fn could_sleep(&self) -> bool {
        self.state.borrow().could_sleep()
    }

    /// How often the app looks for tabs to put to sleep.
    #[must_use]
    pub fn sleep_look_every(&self) -> Duration {
        self.sleep.1
    }

    /// A look for tabs to put to sleep (053): each tab that has been out of
    /// sight for the time, with no reason to be awake, is asleep when this
    /// returns. Its web view's state is asked for, where it has one worth
    /// asking for, and then the web view is let go: the two are asked
    /// together, in that order. The state is the tab's when it arrives.
    pub fn sleep_idle_tabs(&self) {
        let due = self.state.borrow_mut().sleepers(self.sleep.0);
        self.let_go(due);
    }

    /// A look at the limit of live tabs (054): with more tabs alive than
    /// the device can hold, the ones longest out of sight are asleep when
    /// this returns, each as the time would have put it to sleep.
    pub(crate) fn sleep_over_the_limit(&self) {
        let due = self.state.borrow_mut().over_the_limit(self.live_tabs);
        self.let_go(due);
    }

    /// The system is short of memory (055): the tabs that are out of sight
    /// with no reason to be awake are asleep when this returns, each as the
    /// time would have put it to sleep; and when it is very short, the tabs
    /// that are only on their way too. A level this core does not know is
    /// taken for short. Done once, and nothing lasts.
    pub fn memory_short(&self, level: i32) {
        let very = matches!(MemoryLevel::try_from(level), Ok(MemoryLevel::VeryShort));
        let due = self.state.borrow_mut().short_of_memory(very);
        self.let_go(due);
    }

    /// Asks the shell for what is left to do for tabs that were put to
    /// sleep: each one's state, where it has one worth asking for, and then
    /// its web view let go, the two together and in that order.
    fn let_go(&self, due: Vec<Sleeper>) {
        for sleeper in due {
            if sleeper.ask_state {
                let asking =
                    self.host
                        .request(host_request::Kind::WebViewSaveState(WebViewSaveState {
                            web_view_id: sleeper.web_view.get(),
                        }));
                let app = self.clone();
                tokio::task::spawn_local(async move {
                    // A shell that has none to give, or will not say, gives none.
                    let state = match asking.await {
                        Ok(host_response::Kind::WebViewState(answer)) => answer.state,
                        _ => Vec::new(),
                    };
                    app.sleeper_answered(sleeper.tab, sleeper.web_view, sleeper.engine, &state);
                });
            }
            let destroying =
                self.host
                    .request(host_request::Kind::WebViewDestroy(WebViewDestroy {
                        web_view_id: sleeper.web_view.get(),
                    }));
            // Asked already; this only waits for the answer. A shell that
            // fails to destroy a web view keeps a leak, not a tab.
            tokio::task::spawn_local(async move {
                let _ = destroying.await;
            });
        }
    }

    /// The shell has said the state of a web view whose tab was put to
    /// sleep. It is kept when there is one and it is not too large, and is
    /// then the tab's, in the place of the one it had; with none, the one it
    /// had stays.
    fn sleeper_answered(&self, tab: TabId, web_view: WebViewId, engine: i32, state: &[u8]) {
        let keep = !state.is_empty() && state.len() <= MOST_STATE;
        let id = if keep {
            self.store.borrow_mut().keep_state(engine, state).ok()
        } else {
            None
        };
        let unwanted = self
            .state
            .borrow_mut()
            .sleeper_state_kept(tab, web_view, id);
        for id in unwanted {
            let _ = self.store.borrow_mut().drop_state(id);
        }
        // The session names the state: it is written with it.
        self.persist();
    }

    /// Asks for the state of every tab that has left the front since this
    /// was last called, and drops from the store the states whose tabs are
    /// gone. The asking is done before this returns; what comes of each is
    /// kept when it arrives.
    pub(crate) fn keep_states(&self) {
        self.drop_states();
        let left = self.state.borrow_mut().left_the_front();
        for (tab, web_view, commits, engine) in left {
            let asking =
                self.host
                    .request(host_request::Kind::WebViewSaveState(WebViewSaveState {
                        web_view_id: web_view.get(),
                    }));
            let app = self.clone();
            tokio::task::spawn_local(async move {
                // A shell that has none to give, or will not say, gives none.
                let state = match asking.await {
                    Ok(host_response::Kind::WebViewState(answer)) => answer.state,
                    _ => Vec::new(),
                };
                app.state_answered(tab, web_view, commits, engine, &state);
            });
        }
    }

    /// Drops from the store the states that are wanted no longer: of tabs
    /// that are gone, have gone on to another page, have become Sites, or
    /// whose history was forgotten.
    pub(crate) fn drop_states(&self) {
        let gone = self.state.borrow_mut().take_states_to_drop();
        for id in gone {
            // A store that refuses keeps a state nobody names: it goes at
            // the next start.
            let _ = self.store.borrow_mut().drop_state(id);
        }
    }

    /// Drops the same, for good: history was forgotten (008), and what a
    /// state held of it is gone from the store's files with the visits.
    ///
    /// # Errors
    /// The store's code when it refuses.
    pub(crate) fn forget_states(&self) -> Result<(), wt_store::StoreError> {
        let gone = self.state.borrow_mut().take_states_to_drop();
        self.store.borrow_mut().forget_states(&gone)
    }

    /// The shell has said a web view's state. It is kept when there is one
    /// and it is not too large, and is then the tab's; the one the tab had
    /// is dropped either way: an old state for a new page would wake the tab
    /// where it no longer is.
    fn state_answered(
        &self,
        tab: TabId,
        web_view: WebViewId,
        commits: u64,
        engine: i32,
        state: &[u8],
    ) {
        let keep = !state.is_empty() && state.len() <= MOST_STATE;
        let id = if keep {
            self.store.borrow_mut().keep_state(engine, state).ok()
        } else {
            None
        };
        let unwanted = self
            .state
            .borrow_mut()
            .state_kept(tab, web_view, commits, id);
        for id in unwanted {
            let _ = self.store.borrow_mut().drop_state(id);
        }
        // The session names the state: it is written with it.
        self.persist();
    }
}
