//! The standard session (000 §7): "5 Spaces, 200 Sites, 40 temporary tabs,
//! one live page". What the budgets are counted against (060), made once
//! and the same every time: through the contract, as a person would make
//! it, in a shell whose store keeps it.
//!
//! Each Space has forty Sites (four of them Favourites, sixteen in two
//! Groups, twenty in none) and eight tabs. When it is made, the window
//! shows the first Space with its last tab in front, and every other tab of
//! every Space is asleep.

use wt_proto::v1::{CreateSpace, SleepOthers, SwitchSpace, request, response};

use crate::FakeShell;

/// How many Sites and how many tabs a Space of the standard session has.
pub const SITES_A_SPACE: usize = 40;
pub const TABS_A_SPACE: usize = 8;
/// How many Spaces the standard session has.
pub const SPACES: usize = 5;

const FAVOURITES_A_SPACE: usize = 4;
const GROUPS_A_SPACE: usize = 2;
const SITES_A_GROUP: usize = 8;

/// The standard session as it was made.
#[derive(Debug)]
pub struct StandardSession {
    pub window: u64,
    pub subscription: u64,
    /// Its Spaces, in the order they were made: the first is the app's own.
    pub spaces: Vec<u64>,
}

/// The address and the title of a Space's `number`th Site.
fn site(space: usize, number: usize) -> (String, String) {
    (
        format!("https://site-{number}.space-{space}.example.com/"),
        format!("Site {number} of Space {space}"),
    )
}

/// The address and the title of a Space's `number`th tab: of ordinary
/// lengths, as a page somebody was reading has.
fn tab(space: usize, number: usize) -> (String, String) {
    (
        format!(
            "https://www.example.com/space-{space}/articles/a-page-with-a-path-of-an-ordinary-length-{number}"
        ),
        format!("A page with a title of an ordinary length, number {number} of Space {space}"),
    )
}

/// Has the scripted engine know the pages of a standard session of so many
/// Spaces: whatever of it is loaded, loads.
pub fn standard_pages(shell: &mut FakeShell, spaces: usize) {
    for space in 0..spaces {
        for number in 0..SITES_A_SPACE {
            let (url, title) = site(space, number);
            shell.engine.page(&url, &title);
        }
        for number in 0..TABS_A_SPACE {
            let (url, title) = tab(space, number);
            shell.engine.page(&url, &title);
        }
    }
}

async fn asked(shell: &mut FakeShell, kind: request::Kind) -> response::Kind {
    let answer = shell.request(kind).await;
    assert!(
        !matches!(answer, response::Kind::Error(_)),
        "the standard session could not be made: {answer:?}"
    );
    answer
}

/// Makes a standard session of so many Spaces in a shell that has none:
/// opens its window, and makes the Spaces one after the other. The pages
/// are the scripted engine's ([`standard_pages`] is called).
pub async fn standard_session_of(shell: &mut FakeShell, spaces: usize) -> StandardSession {
    standard_pages(shell, spaces);
    let (window, first_tab, subscription) = shell.open_window().await;
    let mut made = vec![shell.window(subscription).space_id];
    for space in 0..spaces {
        if space > 0 {
            let answer = asked(
                shell,
                request::Kind::CreateSpace(CreateSpace {
                    name: format!("Space {space}"),
                    ..CreateSpace::default()
                }),
            )
            .await;
            let response::Kind::SpaceCreated(created) = answer else {
                panic!("a Space that is made is answered with its number, not {answer:?}");
            };
            made.push(created.space_id);
            asked(
                shell,
                request::Kind::SwitchSpace(SwitchSpace {
                    window_id: window,
                    space_id: created.space_id,
                }),
            )
            .await;
        }
        // The Sites: each a tab that is opened at its home and kept.
        let mut sites = Vec::new();
        for number in 0..SITES_A_SPACE {
            let (url, _) = site(space, number);
            // A Space's first tab is the empty one it was made with.
            let id = if number == 0 {
                let empty = if space == 0 {
                    first_tab
                } else {
                    shell
                        .window(subscription)
                        .tab
                        .expect("a window has a tab")
                        .tab_id
                };
                shell.submit(empty, &url).await;
                empty
            } else {
                shell.open_tab(window, &url).await
            };
            let kept = shell.keep_tab(id).await;
            assert!(matches!(kept, response::Kind::Ack(_)), "{kept:?}");
            sites.push(id);
        }
        for id in &sites[..FAVOURITES_A_SPACE] {
            let done = shell.set_favourite(*id, true).await;
            assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
        }
        let grouped =
            &sites[FAVOURITES_A_SPACE..FAVOURITES_A_SPACE + GROUPS_A_SPACE * SITES_A_GROUP];
        for (number, members) in grouped.chunks(SITES_A_GROUP).enumerate() {
            let answer = shell.make_group(window, &format!("Group {number}")).await;
            let response::Kind::GroupMade(group) = answer else {
                panic!("a Group that is made is answered with its number, not {answer:?}");
            };
            for id in members {
                let moved = shell.move_to_group(*id, group.group_id).await;
                assert!(matches!(moved, response::Kind::Ack(_)), "{moved:?}");
            }
        }
        // And its tabs, the last of them in front.
        for number in 0..TABS_A_SPACE {
            shell.open_tab(window, &tab(space, number).0).await;
        }
    }
    // The window shows the first Space, and nothing but the page in front is alive.
    asked(
        shell,
        request::Kind::SwitchSpace(SwitchSpace {
            window_id: window,
            space_id: made[0],
        }),
    )
    .await;
    asked(
        shell,
        request::Kind::SleepOthers(SleepOthers { window_id: window }),
    )
    .await;
    shell.settle().await;
    StandardSession {
        window,
        subscription,
        spaces: made,
    }
}

/// The standard session: five Spaces.
pub async fn standard_session(shell: &mut FakeShell) -> StandardSession {
    standard_session_of(shell, SPACES).await
}
