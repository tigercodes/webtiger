//! Several Spaces (024): listing them, making and naming one, the Space a
//! window shows, and the session, which is each Space's tabs (009).

use std::collections::BTreeMap;

use wt_core::{MOST_SPACES, SpaceId, WindowId, space_name};
use wt_proto::v1::{
    Ack, CreateSpace, RenameSpace, SpaceCreated, SpaceSummary, SpacesPage, SwitchSpace, response,
};
use wt_store::{Logins, SavedTab, Space, Store};

use crate::app::{App, AppFuture, refuse};
use crate::error::AppError;

/// The session as the store has it: written again only where it differs.
#[derive(Debug, Default)]
pub(crate) struct Written {
    shown: Option<SpaceId>,
    rows: BTreeMap<SpaceId, Vec<SavedTab>>,
}

/// What the store has of the Spaces as the app starts.
pub(crate) struct Kept {
    /// The Spaces, in their order.
    pub(crate) spaces: Vec<Space>,
    /// The one a window opens with: the one it showed when the app last
    /// ran, or the first.
    pub(crate) start: Space,
    /// Each Space's tabs, where a session is kept at all.
    pub(crate) saved: Option<BTreeMap<SpaceId, Vec<SavedTab>>>,
    pub(crate) written: Written,
}

/// Reads what the store has of the Spaces. `own` is the app's own Space,
/// which is all there is when the list cannot be read; `session` is whether
/// tabs are kept between runs.
pub(crate) fn kept(store: &Store, own: Space, session: bool) -> Kept {
    let spaces = store
        .spaces()
        .ok()
        .filter(|spaces| !spaces.is_empty())
        .unwrap_or_else(|| vec![own.clone()]);
    let shown = store.shown_space().ok().flatten();
    let start = shown
        .and_then(|id| spaces.iter().find(|space| space.id == id))
        .or_else(|| spaces.first())
        .cloned()
        .unwrap_or(own);
    // A session that cannot be read is no session: the window opens empty,
    // and what it then holds is what is kept.
    let saved = session.then(|| {
        spaces
            .iter()
            .map(|space| (space.id, store.saved_tabs(space.id).unwrap_or_default()))
            .collect::<BTreeMap<_, _>>()
    });
    let written = Written {
        shown,
        rows: saved.clone().unwrap_or_default(),
    };
    Kept {
        spaces,
        start,
        saved,
        written,
    }
}

impl App {
    /// Writes the session to the store where it is not what the store has
    /// (009): each Space's tabs, and the Space the window shows (024).
    /// Called when the windows' snapshots are published, so that what is
    /// kept is what was last seen. A store that refuses keeps what it had:
    /// the next change tries again.
    pub fn persist(&self) {
        let Some(session) = self.state.borrow_mut().session() else {
            return;
        };
        let mut written = self.written.borrow_mut();
        let mut store = self.store.borrow_mut();
        for (space, tabs) in session.rows {
            if written.rows.get(&space) == Some(&tabs) {
                continue;
            }
            if store.save_tabs(space, &tabs).is_ok() {
                written.rows.insert(space, tabs);
            }
        }
        if written.shown != Some(session.shown) && store.show_space(session.shown).is_ok() {
            written.shown = Some(session.shown);
        }
    }

    /// The Spaces, in their order, and which of them share their logins.
    pub(crate) fn query_spaces(&self) -> response::Kind {
        let state = self.state.borrow();
        let spaces = state.spaces();
        let summaries = spaces
            .iter()
            .map(|space| {
                // The place of the first Space with these logins names them:
                // a number of this answer's, and nothing of the login set.
                let first = spaces
                    .iter()
                    .position(|other| other.login_set == space.login_set)
                    .unwrap_or(0);
                SpaceSummary {
                    space_id: space.id.get(),
                    name: space.name.clone(),
                    logins: u32::try_from(first + 1).unwrap_or(u32::MAX),
                }
            })
            .collect();
        response::Kind::SpacesPage(SpacesPage { spaces: summaries })
    }

    /// Makes a Space, after the others (024 §3). Refused, nothing is made.
    pub(crate) fn create_space(&self, create: &CreateSpace) -> Result<response::Kind, AppError> {
        let name = space_name(&create.name)
            .ok_or_else(|| AppError::new("space.name_empty").field("name"))?;
        if self.state.borrow().spaces().len() >= MOST_SPACES {
            return Err(AppError::new("space.too_many"));
        }
        let logins = if create.share_logins_with == 0 {
            // Logins of its own: a login set nobody has had.
            Logins::Own((self.new_login_set)().ok_or_else(|| AppError::new("space.random"))?)
        } else {
            Logins::Of(SpaceId(create.share_logins_with))
        };
        // The store says whether the Space to share with is there.
        let made = self
            .store
            .borrow_mut()
            .create_space(&name, logins, "", self.clock.now())?
            .ok_or_else(|| AppError::new("space.unknown").field("share_logins_with"))?;
        let space_id = made.id.get();
        self.state.borrow_mut().space_made(made);
        Ok(response::Kind::SpaceCreated(SpaceCreated { space_id }))
    }

    /// Names a Space, the app's own among them (024 §3).
    pub(crate) fn rename_space(&self, rename: &RenameSpace) -> Result<response::Kind, AppError> {
        let id = SpaceId(rename.space_id);
        let unknown = || AppError::new("space.unknown").field("space_id");
        if !self
            .state
            .borrow()
            .spaces()
            .iter()
            .any(|space| space.id == id)
        {
            return Err(unknown());
        }
        let name = space_name(&rename.name)
            .ok_or_else(|| AppError::new("space.name_empty").field("name"))?;
        if !self.store.borrow_mut().rename_space(id, &name)? {
            return Err(unknown());
        }
        self.state.borrow_mut().space_named(id, &name);
        Ok(response::Kind::Ack(Ack {}))
    }

    /// A window shows another Space (024 §3).
    pub(crate) fn switch_space(&self, switch: &SwitchSpace) -> AppFuture {
        let switched = self
            .state
            .borrow_mut()
            .switch_space(WindowId(switch.window_id), SpaceId(switch.space_id));
        if let Err(code) = switched {
            let field = if code == "window.unknown" {
                "window_id"
            } else {
                "space_id"
            };
            return refuse(AppError::new(code).field(field));
        }
        // A tab that came back from the session is loaded when it comes in
        // front (009): the one in front of the Space now shown is. The
        // window shows the Space whatever becomes of that load.
        let waking = self.wake();
        Box::pin(async move {
            for loading in waking {
                let _ = loading.await;
            }
            Ok(response::Kind::Ack(Ack {}))
        })
    }
}
