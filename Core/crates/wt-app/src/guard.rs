//! A person's own rules for Guard (066): one text, as they wrote it, kept in
//! the store and in force from the moment it is set. The blocker itself is
//! not here: whoever holds it (the fast path's side of the core) is told the
//! text each time it changes, and makes a new blocker of it.

use std::rc::Rc;

use wt_core::{GuardSwitches, LoginSetId, SpaceId, TabId};
use wt_proto::v1::{
    Ack, GuardRules, GuardSwitch, NavigationCommand, SetGuard, SetGuardRules, WebViewCommand,
    host_request, response, set_guard,
};
use wt_store::Store;

use crate::app::{App, AppFuture, refuse};
use crate::error::AppError;

/// The setting a person's rules are kept in.
const SETTING: &str = "guard.rules";

/// The most a person's rules may be, in bytes of text (066 D5): they are a
/// person's rules, and not a list by another way.
pub(crate) const MOST_RULES: usize = 256 * 1024;

/// Who is told the rules that are in force: at once when it begins to
/// listen, and each time they change.
pub(crate) type RulesSink = Rc<dyn Fn(&str)>;

/// Who is told Guard's switches, and which web view is whose (067): at once
/// when it begins to listen, and each time either changes.
pub(crate) type SwitchesSink = Rc<dyn Fn(&GuardSwitches)>;

/// The setting everywhere's switch is kept in: "off", or nothing for on.
const EVERYWHERE: &str = "guard.everywhere";

/// The setting a Space's switch is kept in: "on", "off", or nothing said.
fn space_setting(space: SpaceId) -> String {
    format!("guard.space.{}", space.get())
}

/// Guard's switches as the store has kept them (067), for the Spaces there
/// are: everywhere's, each Space's, and each site's in each login set. A
/// store that cannot say has none, and Guard is then on everywhere.
pub(crate) fn kept_switches(store: &Store, spaces: impl Iterator<Item = SpaceId>) -> GuardSwitches {
    let said = |key: &str| store.setting(key).ok().flatten();
    let mut switches = GuardSwitches {
        off_everywhere: said(EVERYWHERE).as_deref() == Some("off"),
        ..GuardSwitches::default()
    };
    for space in spaces {
        match said(&space_setting(space)).as_deref() {
            Some("on") => switches.spaces.insert(space, true),
            Some("off") => switches.spaces.insert(space, false),
            _ => None,
        };
    }
    for (login_set, site, on) in store.site_guards().unwrap_or_default() {
        switches
            .sites
            .entry(login_set)
            .or_default()
            .insert(site, on);
    }
    switches
}

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

    /// Sets one of Guard's three switches (067). Kept; in force when this
    /// returns, whoever answers the fast path having been told; and, for a
    /// site's switch that changed what Guard is for the tab's page, the tab
    /// reloaded.
    pub(crate) fn set_guard(&self, set: &SetGuard) -> AppFuture {
        let says = match GuardSwitch::try_from(set.switch) {
            Ok(GuardSwitch::On) => Some(true),
            Ok(GuardSwitch::Off) => Some(false),
            Ok(GuardSwitch::Unspecified) => None,
            Err(_) => return refuse(AppError::new("guard.switch").field("switch")),
        };
        let mut reload = None;
        match set.of {
            None => return refuse(AppError::new("guard.where").field("of")),
            Some(set_guard::Of::Everywhere(_)) => {
                // Everywhere is on or off: there is no level above it.
                let Some(on) = says else {
                    return refuse(AppError::new("guard.switch").field("switch"));
                };
                // A store that refuses keeps nothing: the switch holds for
                // this run all the same.
                let _ = self
                    .store
                    .borrow_mut()
                    .set_setting(EVERYWHERE, if on { "on" } else { "off" });
                self.state.borrow_mut().set_guard_everywhere(on);
            }
            Some(set_guard::Of::SpaceId(space)) => {
                let space = SpaceId(space);
                if !self.state.borrow_mut().set_guard_space(space, says) {
                    return refuse(AppError::new("space.unknown").field("space_id"));
                }
                let kept = match says {
                    Some(true) => "on",
                    Some(false) => "off",
                    None => "",
                };
                let _ = self
                    .store
                    .borrow_mut()
                    .set_setting(&space_setting(space), kept);
            }
            Some(set_guard::Of::TabId(tab)) => {
                let tab = TabId(tab);
                let (login_set, site, web_view) = match self.state.borrow().guard_site_of(tab) {
                    Ok(found) => found,
                    Err(false) => return refuse(AppError::new("tab.unknown").field("tab_id")),
                    Err(true) => return refuse(AppError::new("guard.no_site").field("tab_id")),
                };
                let before = self.state.borrow().guard_for(tab);
                self.keep_site_guard(login_set, &site, says);
                self.state
                    .borrow_mut()
                    .set_guard_site(login_set, &site, says);
                // The page the person is looking at, under their choice: when
                // the choice changed what Guard is for it.
                if self.state.borrow().guard_for(tab).map(|(on, _)| on) != before.map(|(on, _)| on)
                {
                    reload = web_view;
                }
            }
        }
        // Whoever answers the fast path knows before anything is reloaded.
        self.tell_guard_switches();
        let Some(web_view) = reload else {
            return Box::pin(std::future::ready(Ok(response::Kind::Ack(Ack {}))));
        };
        let reloading = self
            .host
            .request(host_request::Kind::WebViewCommand(WebViewCommand {
                web_view_id: web_view.get(),
                command: NavigationCommand::Reload as i32,
            }));
        Box::pin(async move {
            // The switch is set whatever becomes of the reload.
            let _ = reloading.await;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    /// Keeps a site's switch with its logins: in the store of an ordinary
    /// login set, and in memory for a private window's (037).
    fn keep_site_guard(&self, login_set: LoginSetId, site: &str, says: Option<bool>) {
        let _ = self
            .store_of(login_set)
            .borrow_mut()
            .set_site_guard(login_set, site, says);
    }

    /// Has `sink` told Guard's switches and which web view is whose: now,
    /// and each time either changes.
    pub(crate) fn on_guard_switches(&self, sink: SwitchesSink) {
        *self.guard_switches_sink.borrow_mut() = Some(sink);
        *self.guard_switches_told.borrow_mut() = None;
        self.tell_guard_switches();
    }

    /// Tells whoever answers the fast path Guard's switches and which web
    /// view is whose, when either is not what it was last told. Called when
    /// a switch is set; before a new web view is asked of the shell, so that
    /// its first request is under the right switches; and whenever the
    /// windows are told of a change.
    pub(crate) fn tell_guard_switches(&self) {
        let Some(sink) = self.guard_switches_sink.borrow().clone() else {
            return;
        };
        let now = self.state.borrow().guard_switches();
        if self.guard_switches_told.borrow().as_ref() == Some(&now) {
            return;
        }
        sink(&now);
        *self.guard_switches_told.borrow_mut() = Some(now);
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
