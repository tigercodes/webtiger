//! A tab's kept state (051): the engine's own state of a web view, asked
//! for when its tab leaves the front, kept in the store, and handed back
//! when the tab wakes. To the core it is bytes: it never reads them.

use wt_core::{TabId, WebViewId};
use wt_proto::v1::{WebViewSaveState, host_request, host_response};

use crate::app::App;

/// The most bytes of a state that is kept (051 D3). A larger one is not
/// kept: its tab wakes at its address.
pub(crate) const MOST_STATE: usize = 512 * 1024;

impl App {
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
