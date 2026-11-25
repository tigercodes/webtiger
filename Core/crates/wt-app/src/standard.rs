//! The standard session (000 §7), for an app that is to be measured on it
//! (060): "5 Spaces, 200 Sites, 40 temporary tabs, one live page". Fake mode
//! can start on it: it is written into fake mode's store, which is in
//! memory, before the app is made, and the app then finds a session as it
//! finds one a person left.
//!
//! Each Space has forty Sites (four of them Favourites, sixteen in two
//! Groups, twenty in none) and eight tabs, the last of them in front. (The
//! test kit makes the same session through the contract, as a person would:
//! `wt_testkit::standard_session`. The two are held to the same counts by
//! the tests of each.)

use wt_core::{SPACE_COLOURS, Timestamp, Uuid};
use wt_store::{Logins, SavedGroup, SavedTab, Space, Store, StoreError};

const SPACES: usize = 5;
const FAVOURITES_A_SPACE: usize = 4;
const LOOSE_SITES_A_SPACE: usize = 20;
const GROUPS_A_SPACE: usize = 2;
const SITES_A_GROUP: usize = 8;
const TABS_A_SPACE: usize = 8;

/// A Site of the standard session, as the session keeps one: asleep at its
/// home, by the name it was kept with.
fn site(space: usize, number: usize, group: u32, favourite: bool) -> SavedTab {
    let url = format!("https://site-{number}.space-{space}.example.com/");
    SavedTab {
        title: format!("Site {number} of Space {space}"),
        front: false,
        home: url.clone(),
        name: format!("Site {number} of Space {space}"),
        group,
        favourite,
        shown_at: 0,
        state: 0,
        url,
    }
}

/// A tab of the standard session: at an address and by a title of ordinary
/// lengths, as a page somebody was reading has.
fn tab(space: usize, number: usize) -> SavedTab {
    SavedTab {
        url: format!(
            "https://www.example.com/space-{space}/articles/a-page-with-a-path-of-an-ordinary-length-{number}"
        ),
        title: format!(
            "A page with a title of an ordinary length, number {number} of Space {space}"
        ),
        front: false,
        home: String::new(),
        name: String::new(),
        group: 0,
        favourite: false,
        shown_at: 0,
        state: 0,
    }
}

/// A Space's row, in a row's order (041, 042): its Favourites; its Sites
/// that are in no Group; each Group's Sites; its tabs, the last in front.
fn row(space: usize) -> Vec<SavedTab> {
    let mut number = 0..;
    let mut next = |group: u32, favourite: bool| {
        let n = number.next().unwrap_or(0);
        site(space, n, group, favourite)
    };
    let mut tabs: Vec<SavedTab> = (0..FAVOURITES_A_SPACE).map(|_| next(0, true)).collect();
    tabs.extend((0..LOOSE_SITES_A_SPACE).map(|_| next(0, false)));
    for group in 1..=GROUPS_A_SPACE {
        let place = u32::try_from(group).unwrap_or(0);
        tabs.extend((0..SITES_A_GROUP).map(|_| next(place, false)));
    }
    tabs.extend((0..TABS_A_SPACE).map(|n| tab(space, n)));
    if let Some(last) = tabs.last_mut() {
        last.front = true;
    }
    tabs
}

/// Writes the standard session into a store that has the app's own Space
/// and nothing else: four Spaces more, each with logins of its own; every
/// Space's Groups and row; the first Space shown. `front` is where the tab
/// in front of the first Space is: the one page that will be alive.
///
/// # Errors
/// The store's, when it refuses.
pub(crate) fn make(
    store: &mut Store,
    own: &Space,
    front: Option<&str>,
    now: Timestamp,
) -> Result<(), StoreError> {
    let mut spaces = vec![own.id];
    for number in 1..SPACES {
        // Logins of its own, by an id that is the same every run and is none
        // of those fake mode gives a Space that is made later.
        let byte = 0xA0_u8.wrapping_add(u8::try_from(number).unwrap_or(0));
        let made = store.create_space(
            &format!("Space {number}"),
            Logins::Own(Uuid::from_random_bytes([byte; 16])),
            SPACE_COLOURS[number % SPACE_COLOURS.len()],
            "",
            now,
        )?;
        if let Some(space) = made {
            spaces.push(space.id);
        }
    }
    let groups: Vec<SavedGroup> = (1..=GROUPS_A_SPACE)
        .map(|number| SavedGroup {
            name: format!("Group {number}"),
            colour: String::new(),
            shut: false,
        })
        .collect();
    for (number, space) in spaces.iter().enumerate() {
        let mut tabs = row(number);
        if number == 0
            && let (Some(front), Some(last)) = (front, tabs.last_mut())
        {
            front.clone_into(&mut last.url);
            last.title.clear();
        }
        store.save_groups(*space, &groups)?;
        store.save_tabs(*space, &tabs)?;
    }
    store.show_space(own.id)
}
