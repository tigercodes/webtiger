//! A window's layout (043): the tabs along the top, or down the side. A view
//! setting: what the person last chose is what a new window has, and where
//! that choice is kept.

use wt_core::WindowId;
use wt_proto::v1::{Ack, Layout, SetLayout, response};
use wt_store::Store;

use crate::app::App;
use crate::error::AppError;

/// The setting the choice is written by.
const SETTING: &str = "window.layout";
const STRIP: &str = "strip";
const SIDEBAR: &str = "sidebar";

/// The layout a new window has as the app starts: what the store has, when
/// it is one the core knows; otherwise the strip.
pub(crate) fn kept(store: &Store) -> Layout {
    match store.setting(SETTING).ok().flatten().as_deref() {
        Some(SIDEBAR) => Layout::Sidebar,
        _ => Layout::Strip,
    }
}

impl App {
    /// Switches a window's layout, and nothing else. The choice is the
    /// app's from now on, and is written down for the next time it starts;
    /// a private window's is its own, and is not.
    pub(crate) fn set_layout(&self, set: &SetLayout) -> Result<response::Kind, AppError> {
        let (layout, word) = match Layout::try_from(set.layout) {
            Ok(Layout::Strip) => (Layout::Strip, STRIP),
            Ok(Layout::Sidebar) => (Layout::Sidebar, SIDEBAR),
            _ => return Err(AppError::new("layout.unknown").field("layout")),
        };
        let remember = self
            .state
            .borrow_mut()
            .set_layout(WindowId(set.window_id), layout)
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))?;
        if remember {
            // A store that refuses keeps nothing: the choice holds for this
            // run all the same.
            let _ = self.store.borrow_mut().set_setting(SETTING, word);
        }
        Ok(response::Kind::Ack(Ack {}))
    }
}
