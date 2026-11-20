//! Where searches go (020): the engines the core knows, the person's
//! choice among them, and where it is kept.

use wt_core::{DEFAULT_SEARCH_ENGINE, SEARCH_ENGINES, SearchEngine, search_engine};
use wt_proto::v1::{Ack, ChooseSearchEngine, SearchSettings, response};
use wt_store::Store;

use crate::app::App;
use crate::error::AppError;

/// The setting the choice is written by: the engine's id.
const SETTING: &str = "search.engine";

/// The default engine, or the first of the list should the list ever lose
/// it.
fn default_engine() -> &'static SearchEngine {
    search_engine(DEFAULT_SEARCH_ENGINE).unwrap_or(&SEARCH_ENGINES[0])
}

/// Where searches go as the app starts, and whether a person chose it. What
/// the store has, when it is an engine the core still knows; otherwise the
/// default, which is taken as chosen only where nobody is
/// there to be asked (`settled`).
pub(crate) fn kept(store: &Store, settled: bool) -> (&'static SearchEngine, bool) {
    let chosen = store
        .setting(SETTING)
        .ok()
        .flatten()
        .and_then(|id| search_engine(&id));
    match chosen {
        Some(engine) => (engine, true),
        None => (default_engine(), settled),
    }
}

impl App {
    /// The engines a person chooses among, the one in use, and whether
    /// anybody chose it.
    pub(crate) fn search_settings(&self) -> response::Kind {
        let (engine, chosen) = self.search.get();
        response::Kind::SearchSettings(SearchSettings {
            engines: SEARCH_ENGINES
                .iter()
                .map(|engine| wt_proto::v1::SearchEngine {
                    id: engine.id.to_owned(),
                    name: engine.name.to_owned(),
                    host: engine.host(),
                })
                .collect(),
            chosen_id: engine.id.to_owned(),
            chosen,
        })
    }

    /// The person's choice of where searches go. It is the whole app's from
    /// now, and is written down for the next time the app starts.
    pub(crate) fn choose_search_engine(
        &self,
        choice: &ChooseSearchEngine,
    ) -> Result<response::Kind, AppError> {
        let engine = search_engine(&choice.id)
            .ok_or_else(|| AppError::new("search.unknown_engine").field("id"))?;
        // A store that refuses keeps nothing: the choice holds for this run
        // all the same, and the person is asked again at the next.
        let _ = self.store.borrow_mut().set_setting(SETTING, engine.id);
        self.search.set((engine, true));
        Ok(response::Kind::Ack(Ack {}))
    }
}
