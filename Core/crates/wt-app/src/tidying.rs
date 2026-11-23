//! Tidying (049): each Space's time, the look that tidies the tabs that are
//! past it, and the Archive they are listed in.
//!
//! The rule is the model's (`State::tidy`); what is kept is the store's.
//! Here they meet: a tab leaves its row once the Archive has it, and not
//! before.

use std::time::Duration;

use wt_core::{SpaceId, WindowId};
use wt_proto::v1::{
    Ack, ArchiveEntry, ArchivePage, ForgetArchived, OpenTab, QueryArchive, RestoreArchived,
    SetSpaceTidying, WebViewDestroy, forget_archived, host_request, response,
};

use crate::app::{App, AppFuture, refuse};
use crate::error::AppError;

/// The times a Space's tidying can have (049 D3), in seconds: never, twelve
/// hours, a day, a week, thirty days.
pub(crate) const TIDY_TIMES: [u32; 5] = [0, 43_200, 86_400, 604_800, 2_592_000];

/// How many of the Archive are in a page when the shell does not say; and
/// the most a page has.
const ARCHIVE_PAGE: u32 = 100;

/// How often the core looks for tabs to tidy, while some Space is tidied.
pub const TIDY_LOOK_EVERY: Duration = Duration::from_secs(60);

impl App {
    /// The Spaces' times as the store has them: read once, as the app
    /// starts. A store that refuses leaves every Space untidied.
    pub(crate) fn learn_tidy_times(&self) {
        let times = self.store.borrow().tidy_times().unwrap_or_default();
        let mut state = self.state.borrow_mut();
        for (space, seconds) in times {
            state.set_tidy_time(space, seconds);
        }
    }

    /// Some Space is tidied: there is something to look for.
    #[must_use]
    pub fn tidies(&self) -> bool {
        self.state.borrow().tidies()
    }

    /// Looks for tabs to tidy, now (049): each that is past its Space's time
    /// is listed in the Space's Archive and leaves its row, and its web view
    /// is destroyed. A tab the Archive could not be written for stays where
    /// it is, and is looked at again. While no Space is tidied nothing is
    /// looked at, and the clock is not read.
    pub fn tidy(&self) {
        if !self.tidies() {
            return;
        }
        let now = self.clock.now();
        let due = self.state.borrow_mut().tidy(now);
        let mut destroying = Vec::new();
        for tab in due {
            let listed = self
                .store
                .borrow_mut()
                .archive(tab.space, &tab.url, &tab.title, now.0);
            if listed.is_err() {
                continue;
            }
            // It is not the front of its row, and no Site: it leaves its
            // row, and nothing else of the row moves.
            let closed = self.state.borrow_mut().close_tab(tab.tab);
            if let Some(Some(web_view)) = closed {
                destroying.push(self.host.request(host_request::Kind::WebViewDestroy(
                    WebViewDestroy {
                        web_view_id: web_view.get(),
                    },
                )));
            }
        }
        if !destroying.is_empty() {
            // Asked already; this only waits for the answers. A shell that
            // fails to destroy a web view keeps a leak, not a tab.
            tokio::task::spawn_local(async move {
                for destroyed in destroying {
                    let _ = destroyed.await;
                }
            });
        }
    }

    /// Gives a Space its time: one of [`TIDY_TIMES`]. The tabs of its rows
    /// that had no time start theirs now.
    pub(crate) fn set_space_tidying(
        &self,
        set: &SetSpaceTidying,
    ) -> Result<response::Kind, AppError> {
        if !TIDY_TIMES.contains(&set.after_seconds) {
            return Err(AppError::new("tidy.time_unknown").field("after_seconds"));
        }
        let space = SpaceId(set.space_id);
        let unknown = || AppError::new("space.unknown").field("space_id");
        if !self
            .state
            .borrow()
            .spaces()
            .iter()
            .any(|one| one.id == space)
        {
            return Err(unknown());
        }
        match self
            .store
            .borrow_mut()
            .set_tidy_time(space, set.after_seconds)
        {
            Ok(true) => {}
            Ok(false) => return Err(unknown()),
            Err(error) => return Err(AppError::new(error.code())),
        }
        self.state
            .borrow_mut()
            .set_tidy_time(space, set.after_seconds);
        self.tidy();
        Ok(response::Kind::Ack(Ack {}))
    }

    /// A page of the Archive of the Space a window shows, newest first. A
    /// private window has none.
    pub(crate) fn query_archive(&self, query: &QueryArchive) -> Result<response::Kind, AppError> {
        let space = self
            .state
            .borrow()
            .archive_of(WindowId(query.window_id))
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))?;
        let Some(space) = space else {
            return Ok(response::Kind::ArchivePage(ArchivePage::default()));
        };
        let limit = if query.limit == 0 {
            ARCHIVE_PAGE
        } else {
            query.limit.min(ARCHIVE_PAGE)
        } as usize;
        let (entries, more) = self
            .store
            .borrow()
            .archived(space, query.before, limit)
            .map_err(|error| AppError::new(error.code()))?;
        Ok(response::Kind::ArchivePage(ArchivePage {
            entries: entries
                .into_iter()
                .map(|entry| ArchiveEntry {
                    id: entry.id,
                    url: entry.url,
                    title: entry.title,
                    tidied_at: entry.tidied_at,
                })
                .collect(),
            more,
        }))
    }

    /// Brings a tidied tab back (049 D7): it is taken out of the Archive of
    /// the Space the window shows, and is a tab again, opened as any is: the
    /// last of the row's tabs, in front, loading its address.
    pub(crate) fn restore_archived(&self, restore: &RestoreArchived) -> AppFuture {
        let window = WindowId(restore.window_id);
        let Some(space) = self.state.borrow().archive_of(window) else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        let unknown = || AppError::new("archive.unknown").field("entry_id");
        let Some(space) = space else {
            return refuse(unknown());
        };
        let taken = self
            .store
            .borrow_mut()
            .take_archived(space, restore.entry_id);
        match taken {
            Ok(Some(entry)) => self.open_tab(&OpenTab {
                window_id: restore.window_id,
                text: entry.url,
                ..OpenTab::default()
            }),
            Ok(None) => refuse(unknown()),
            Err(error) => refuse(AppError::new(error.code())),
        }
    }

    /// Forgets a tidied tab of the Archive of the Space a window shows, or
    /// all of them. Nothing named, and a private window, forget nothing.
    pub(crate) fn forget_archived(
        &self,
        forget: &ForgetArchived,
    ) -> Result<response::Kind, AppError> {
        let space = self
            .state
            .borrow()
            .archive_of(WindowId(forget.window_id))
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))?;
        let which = match forget.which {
            Some(forget_archived::Which::EntryId(id)) => Some(Some(id)),
            Some(forget_archived::Which::All(true)) => Some(None),
            Some(forget_archived::Which::All(false)) | None => None,
        };
        if let (Some(space), Some(which)) = (space, which) {
            self.store
                .borrow_mut()
                .forget_archived(space, which)
                .map_err(|error| AppError::new(error.code()))?;
        }
        Ok(response::Kind::Ack(Ack {}))
    }
}
