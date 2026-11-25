//! A sleeping tab's picture (059): its page as it was when the tab was put
//! to sleep, asked of the shell with the web view's state, kept with that
//! state in the store, and shown where the page will be while the tab
//! wakes. To the core it is bytes: it never looks at them.
//!
//! It is kept with the state, in the state's row: there for as long as the
//! state is, and gone when the state goes, however it goes. A tab that is
//! woken is given its state, which is then the store's no longer; the
//! picture is held by the tab for as long as the tab is shown with it, and
//! by nothing once the tab's page has arrived.

use wt_core::TabId;
use wt_proto::v1::{PagePicture, PicturesPage, QueryPictures, WebViewPicture, response};
use wt_store::KeptPicture;

use crate::app::App;

/// The most bytes of a picture that is kept (059 D4). A sleeping tab is to
/// cost no more than 250 KB on disk (the plan's budget): its state has at
/// most 128 KB of that (051 D3).
pub(crate) const MOST_BYTES: u32 = 96 * 1024;

/// The most pixels wide a picture is (059 D4).
pub(crate) const MOST_WIDE: u32 = 800;

/// The most pictures one answer carries: there is one for each tab that is
/// waking, and a window wakes one tab at a time.
const MOST_ASKED: usize = 8;

/// What a shell gave for a picture, when it is one to keep: it has bytes,
/// no more than were asked for, and says a size that a picture has and that
/// is no wider than was asked for.
pub(crate) fn worth_keeping(picture: WebViewPicture) -> Option<KeptPicture> {
    let small = u32::try_from(picture.jpeg.len()).is_ok_and(|bytes| bytes <= MOST_BYTES);
    let sized = (1..=MOST_WIDE).contains(&picture.width) && picture.height > 0;
    (!picture.jpeg.is_empty() && small && sized).then_some(KeptPicture {
        jpeg: picture.jpeg,
        width: picture.width,
        height: picture.height,
    })
}

impl App {
    /// The picture of a tab's page has arrived, for a tab that was put to
    /// sleep on `state`. It is kept with that state, where the store has
    /// the state still: one that was woken meanwhile took its state with
    /// it, and has no use for a picture.
    pub(crate) fn sleeper_pictured(&self, state: i64, picture: &KeptPicture) {
        // A store that refuses keeps none: the tab wakes without.
        let _ = self.store.borrow_mut().keep_picture(state, picture);
    }

    /// A tab that has just been woken from `state` is shown with that
    /// state's picture until its page arrives. The picture's number is its
    /// state's: never used twice.
    pub(crate) fn wake_with_picture(&self, tab: TabId, state: i64, picture: KeptPicture) {
        let Ok(number) = u64::try_from(state) else {
            return;
        };
        self.state.borrow_mut().wakes_with(
            tab,
            PagePicture {
                picture_id: number,
                jpeg: picture.jpeg,
                width: picture.width,
                height: picture.height,
            },
        );
    }

    /// The pictures asked for by their numbers (059): of those some tab is
    /// shown with now, in the order asked, and of no more than an answer
    /// carries.
    pub(crate) fn query_pictures(&self, query: &QueryPictures) -> response::Kind {
        let state = self.state.borrow();
        let pictures = query
            .picture_ids
            .iter()
            .take(MOST_ASKED)
            .filter_map(|number| state.picture(*number))
            .collect();
        response::Kind::PicturesPage(PicturesPage { pictures })
    }
}
