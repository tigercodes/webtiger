//! A person's own rules for Guard (066): one text, as they wrote it, kept in
//! the store and in force from the moment it is set. The blocker itself is
//! not here: whoever holds it (the fast path's side of the core) is told the
//! text each time it changes, and makes a new blocker of it.

use std::rc::Rc;

use wt_proto::v1::{Ack, GuardRules, SetGuardRules, response};
use wt_store::Store;

use crate::app::App;
use crate::error::AppError;

/// The setting a person's rules are kept in.
const SETTING: &str = "guard.rules";

/// The most a person's rules may be, in bytes of text (066 D5): they are a
/// person's rules, and not a list by another way.
pub(crate) const MOST_RULES: usize = 256 * 1024;

/// Who is told the rules that are in force: at once when it begins to
/// listen, and each time they change.
pub(crate) type RulesSink = Rc<dyn Fn(&str)>;

/// The rules the store has kept; none when it has none, or cannot say.
pub(crate) fn kept(store: &Store) -> Rc<str> {
    store
        .setting(SETTING)
        .ok()
        .flatten()
        .unwrap_or_default()
        .into()
}

impl App {
    /// A person's own rules, as they were set.
    pub(crate) fn query_guard_rules(&self) -> response::Kind {
        response::Kind::GuardRules(GuardRules {
            text: self.guard_rules.borrow().to_string(),
        })
    }

    /// Sets a person's own rules: the whole text, in place of what there
    /// was. Kept for the next start, and in force when this returns: whoever
    /// holds the blocker has been told. Refused over the limit, with what
    /// there was unchanged.
    pub(crate) fn set_guard_rules(&self, set: &SetGuardRules) -> Result<response::Kind, AppError> {
        if set.text.len() > MOST_RULES {
            return Err(AppError::new("guard.rules_too_long").field("text"));
        }
        // A store that refuses keeps nothing: the rules hold for this run
        // all the same, and the ones before come back at the next.
        let _ = self.store.borrow_mut().set_setting(SETTING, &set.text);
        *self.guard_rules.borrow_mut() = set.text.as_str().into();
        self.tell_guard_rules();
        Ok(response::Kind::Ack(Ack {}))
    }

    /// Has `sink` told the rules that are in force: now, and each time they
    /// change, before the request that changed them is answered.
    pub(crate) fn on_guard_rules(&self, sink: RulesSink) {
        *self.guard_sink.borrow_mut() = Some(sink);
        self.tell_guard_rules();
    }

    fn tell_guard_rules(&self) {
        // (Neither is borrowed while the other's code runs.)
        let sink = self.guard_sink.borrow().clone();
        let rules = Rc::clone(&self.guard_rules.borrow());
        if let Some(sink) = sink {
            sink(&rules);
        }
    }
}
