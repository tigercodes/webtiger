//! What the core holds in memory (060): the core driven in process by the
//! test kit's shell, under an allocator that counts what each thread has
//! asked for and not given back. Counted: everything Rust allocates on the
//! thread the core runs on. Not counted: what SQLite allocates for itself
//! (its page cache), which is C's, and is bounded by SQLite's own setting.
//!
//! This test is here, and not with the core's others, because an allocator
//! is unsafe code, and that is this crate's alone.
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "an allocator is unsafe by its nature; and a test's helpers fail the test the way its assertions do"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use wt_proto::v1::{RowStep, SleepOthers, SwitchSpace, move_tab, request};
use wt_testkit::{FakeShell, SPACES, run, standard_pages, standard_session_of};

/// The plan's budgets (000 §7) that are bytes the core holds.
const A_SLEEPING_TAB: isize = 4 * 1024;
const A_HUNDRED_SLEEPING_TABS: isize = 5 * 1024 * 1024;
const A_SPACE_WITH_NOTHING_LIVE: isize = 100 * 1024;
const THE_CORES_SHARE: isize = 30 * 1024 * 1024;

/// This crate's own ceiling for Guard's blocker (064): 100,000 rules made up
/// for the measure, of the shapes the public lists have. The plan's number is
/// for the public lists (000 §5.8: some 15 MB within the core's share), which
/// are not here: this holds the blocker to what it was measured at with the
/// rules there are, so that one that takes more than its rules for requests
/// fails.
const A_BLOCKER_OF_100_000_RULES: isize = 6 * 1024 * 1024;

/// This crate's own ceilings for a row's line of things to undo (062): the
/// plan has no budget for the line by itself. For a row of forty Sites and
/// for one of two thousand, each with eight Groups: fifty moves of a Site,
/// which keep the order and no tab; fifty names given to a Group, which keep
/// the Groups, and neither a tab nor the order, whatever the row's length;
/// fifty times a Site is made a Favourite or not, which keep the order and
/// the one tab. Each about a quarter over what was measured (25,153, 34,306
/// and 38,733 bytes; 812,033, 34,306 and 809,933), so that a line that keeps
/// more than the thing done changed fails.
const A_SHORT_ROWS_LINE: [isize; 3] = [32 * 1024, 44 * 1024, 48 * 1024];
const A_LONG_ROWS_LINE: [isize; 3] = [1024 * 1024, 44 * 1024, 1024 * 1024];

thread_local! {
    /// The bytes this thread has asked for and not given back.
    static HELD: Cell<isize> = const { Cell::new(0) };
}

/// The system's allocator, with a count beside it.
struct Counting;

/// Adds to this thread's count. A thread that is ending has nowhere left to
/// count: what it gives back then is not counted, and nobody reads it.
fn count(bytes: isize) {
    let _ = HELD.try_with(|held| held.set(held.get() + bytes));
}

fn size(layout: Layout) -> isize {
    isize::try_from(layout.size()).unwrap_or(isize::MAX)
}

// SAFETY: every call is handed to the system's allocator unchanged, so its
// contract is kept by keeping the caller's. The count beside it is a
// thread's own number in a `const` thread-local `Cell`: reading and writing
// it allocates nothing, so the allocator never calls itself.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller keeps `GlobalAlloc::alloc`'s contract, which is
        // the system allocator's own.
        let block = unsafe { System.alloc(layout) };
        if !block.is_null() {
            count(size(layout));
        }
        block
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        // SAFETY: `block` was given by `alloc` or `realloc` above, which are
        // the system allocator's, with this layout.
        unsafe { System.dealloc(block, layout) };
        count(-size(layout));
    }

    unsafe fn realloc(&self, block: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: as `dealloc`; and `new_size` is the caller's to have made
        // valid for this layout's alignment.
        let moved = unsafe { System.realloc(block, layout, new_size) };
        if !moved.is_null() {
            count(isize::try_from(new_size).unwrap_or(isize::MAX) - size(layout));
        }
        moved
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The bytes this thread holds now.
fn held() -> isize {
    HELD.with(Cell::get)
}

/// A page's address and its title, of ordinary lengths: the `number`th.
fn page(number: usize) -> (String, String) {
    (
        format!(
            "https://www.example.com/articles/2026/11/a-page-with-a-path-of-an-ordinary-length-{number}"
        ),
        format!("A page with a title of an ordinary length, number {number}"),
    )
}

/// Opens tabs in a window until it has `tabs` beyond its first, puts every
/// one that can sleep to sleep, has the shell forget its notes, and says
/// what the thread then holds.
async fn with_sleeping_tabs(
    shell: &mut FakeShell,
    window: u64,
    opened: &mut usize,
    tabs: usize,
) -> isize {
    while *opened < tabs {
        *opened += 1;
        shell.open_tab(window, &page(*opened).0).await;
    }
    let answer = shell
        .request(request::Kind::SleepOthers(SleepOthers {
            window_id: window,
        }))
        .await;
    assert!(
        matches!(answer, wt_proto::v1::response::Kind::Ack(_)),
        "{answer:?}"
    );
    shell.settle().await;
    shell.forget();
    held()
}

#[test]
fn the_allocator_counts_what_is_held_and_what_is_given_back() {
    let before = held();
    let kept = vec![0_u8; 100_000];
    let with = held();
    assert!(
        with - before >= 100_000,
        "a hundred thousand bytes are held: {}",
        with - before
    );
    drop(kept);
    assert_eq!(held(), before, "and given back");
    // A vector that grows is counted at what it has come to.
    let mut grown = Vec::with_capacity(10);
    grown.extend(std::iter::repeat_n(7_u64, 5_000));
    assert!(held() - before >= 40_000);
    drop(grown);
    assert_eq!(held(), before);
}

#[test]
fn a_sleeping_tab_is_no_more_than_4_kb_in_the_core_and_a_hundred_add_no_more_than_5_mb() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        for number in 0..=600 {
            let (url, title) = page(number);
            shell.engine.page(&url, &title);
        }
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, &page(0).0).await;
        let mut opened = 0;
        let none = with_sleeping_tabs(&mut shell, window, &mut opened, 0).await;
        let mut last = (0_usize, none);
        println!("with one tab awake and none asleep: {none} bytes held");
        for tabs in [100, 200, 400, 600] {
            let now = with_sleeping_tabs(&mut shell, window, &mut opened, tabs).await;
            let each = (now - last.1) / isize::try_from(tabs - last.0).unwrap();
            println!(
                "with {tabs} tabs opened and all but the one in front asleep: {now} bytes held; {} more than with none; {each} a tab since {}",
                now - none,
                last.0
            );
            // Each tab that sleeps, at every length of row: a row's cost does not creep
            // up with its length.
            assert!(
                each <= A_SLEEPING_TAB,
                "a sleeping tab is {each} bytes in the core, and may be {A_SLEEPING_TAB}"
            );
            // And not nothing: a tab that sleeps has an address and a name to keep, and a
            // count that said otherwise would be counting something else.
            assert!(each >= 500, "a sleeping tab cannot be {each} bytes");
            if tabs == 100 {
                assert!(
                    now - none <= A_HUNDRED_SLEEPING_TABS,
                    "a hundred sleeping tabs add {} bytes, and may add {A_HUNDRED_SLEEPING_TABS}",
                    now - none
                );
            }
            last = (tabs, now);
        }
    });
}

/// What this thread holds once a shell has started on the store in `folder`
/// and opened its window; less what it held before the shell was made.
async fn held_by_a_start_on(folder: &std::path::Path, spaces: usize) -> isize {
    let before = held();
    let mut shell = FakeShell::live(folder);
    standard_pages(&mut shell, spaces);
    shell.open_window().await;
    shell.settle().await;
    shell.forget();
    let after = held();
    drop(shell);
    after - before
}

#[test]
fn a_start_on_the_standard_session_is_within_the_cores_share_and_a_space_not_shown_is_under_100_kb()
{
    run(async {
        // A start on nothing; on a session of one Space; and on the standard session's five.
        let mut started = Vec::new();
        for spaces in [0, 1, SPACES] {
            let folder = tempfile::tempdir().unwrap();
            if spaces > 0 {
                let mut shell = FakeShell::live(folder.path());
                standard_session_of(&mut shell, spaces).await;
            }
            let start = held_by_a_start_on(folder.path(), spaces).await;
            println!("a start on {spaces} Spaces of the standard session: {start} bytes held");
            started.push(start);
        }
        // The core's share (without the default lists, which are Guard's and not built).
        assert!(
            started[2] <= THE_CORES_SHARE,
            "the core holds {} bytes for the standard session, and may hold {THE_CORES_SHARE}",
            started[2]
        );
        // A Space that is not shown, with its forty Sites and eight tabs, none of them
        // alive: what four more of them add, each.
        let space = (started[2] - started[1]) / 4;
        println!("a Space that is not shown, with nothing live: {space} bytes");
        assert!(
            space <= A_SPACE_WITH_NOTHING_LIVE,
            "a Space with nothing live is {space} bytes in the core, and may be {A_SPACE_WITH_NOTHING_LIVE}"
        );
        assert!(space >= 5_000, "forty-eight tabs cannot be {space} bytes");
        // The session is counted: a start on it holds more than a start on nothing.
        assert!(started[1] > started[0] + 20_000 && started[2] > started[1]);
    });
}

/// What the thread holds for a session of so many of the standard session's
/// Spaces, each of which this run has shown and left: the first is shown,
/// one page is alive, and every other tab of every Space is asleep. With
/// `arranged`, the Spaces were made in this run: every Site kept, grouped
/// or made a Favourite, each of which is a thing a row can undo. Without,
/// they were made in a run before, and each was shown once in this one.
async fn held_with_spaces_shown_and_left(spaces: usize, arranged: bool) -> isize {
    let folder = tempfile::tempdir().unwrap();
    if !arranged {
        let mut shell = FakeShell::live(folder.path());
        standard_session_of(&mut shell, spaces).await;
    }
    let before = held();
    let mut shell = FakeShell::live(folder.path());
    if arranged {
        standard_session_of(&mut shell, spaces).await;
    } else {
        standard_pages(&mut shell, spaces);
        let (window, _, _) = shell.open_window().await;
        let ids: Vec<u64> = wt_store::Store::open(&folder.path().join("webtiger.sqlite"))
            .unwrap()
            .spaces()
            .unwrap()
            .iter()
            .map(|space| space.id.0)
            .collect();
        for space in ids.iter().skip(1).chain(ids.first()) {
            shell
                .request(request::Kind::SwitchSpace(SwitchSpace {
                    window_id: window,
                    space_id: *space,
                }))
                .await;
        }
        shell
            .request(request::Kind::SleepOthers(SleepOthers {
                window_id: window,
            }))
            .await;
        shell.settle().await;
    }
    shell.forget();
    let now = held();
    drop(shell);
    now - before
}

/// What a Space that this run has shown and left comes to, with nothing of
/// it alive: what three more of them add to a session of two, each. (Of
/// two, and not of one: with a second Space, the first has itself been left
/// and shown again, as it has with five. A session of one Space has only
/// the Space that is shown, with all it has gathered.)
async fn a_space_shown_and_left(arranged: bool) -> isize {
    let two = held_with_spaces_shown_and_left(2, arranged).await;
    let five = held_with_spaces_shown_and_left(SPACES, arranged).await;
    (five - two) / 3
}

#[test]
fn a_space_that_was_shown_and_left_with_nothing_live_is_no_more_than_100_kb() {
    run(async {
        let space = a_space_shown_and_left(false).await;
        println!("a Space that was shown and left, with nothing live: {space} bytes");
        assert!(
            space <= A_SPACE_WITH_NOTHING_LIVE,
            "a Space that was shown and left is {space} bytes in the core, and may be {A_SPACE_WITH_NOTHING_LIVE}"
        );
        assert!(space >= 5_000, "forty-eight tabs cannot be {space} bytes");
    });
}

#[test]
fn a_space_that_was_arranged_shown_and_left_with_nothing_live_is_no_more_than_100_kb() {
    run(async {
        let space = a_space_shown_and_left(true).await;
        println!(
            "a Space that was shown and left, with nothing live, and arranged in this run: {space} bytes"
        );
        assert!(
            space <= A_SPACE_WITH_NOTHING_LIVE,
            "a Space that was arranged, shown and left is {space} bytes in the core, and may be {A_SPACE_WITH_NOTHING_LIVE}"
        );
    });
}

/// A row of so many Sites, eight of them each in a Group of its own, made in a run that is
/// over: the store
/// in `folder` has it, and a start there finds it asleep, with nothing of its arrangement
/// to undo.
async fn a_long_row_made_before(folder: &std::path::Path, sites: usize) {
    let mut shell = FakeShell::live(folder);
    for number in 0..=sites {
        let (url, title) = page(number);
        shell.engine.page(&url, &title);
    }
    let (window, first, _) = shell.open_window().await;
    shell.submit(first, &page(0).0).await;
    let mut kept = Vec::new();
    for number in 1..=sites {
        let tab = shell.open_tab(window, &page(number).0).await;
        acked(shell.keep_tab(tab).await);
        kept.push(tab);
        if number % 50 == 0 {
            shell.forget();
        }
    }
    for (number, tab) in kept.iter().rev().take(8).enumerate() {
        let made = shell.make_group(window, &format!("Group {number}")).await;
        let wt_proto::v1::response::Kind::GroupMade(group) = made else {
            panic!("a Group, not {made:?}");
        };
        acked(shell.move_to_group(*tab, group.group_id).await);
    }
    shell.settle().await;
}

fn acked(answer: wt_proto::v1::response::Kind) {
    assert!(
        matches!(answer, wt_proto::v1::response::Kind::Ack(_)),
        "{answer:?}"
    );
}

/// A thing done to arrange a row, fifty times over, that leaves the row as it was.
#[derive(Clone, Copy, Debug)]
enum Fifty {
    /// A Site moved a place on and a place back: the order changes, and no tab.
    Moves,
    /// A Group named one thing and the other: a Group changes, and no tab.
    Names,
    /// A Site made a Favourite and not: a tab changes, and the order.
    Favourites,
}

/// What a row's line of things to undo holds (062): what the thread holds once fifty
/// things have been done to arrange the row a start found in `folder`, less what it held
/// before them. The fifty leave the row as it was, so the difference is the line.
async fn held_by_a_line_of(folder: &std::path::Path, fifty: Fifty) -> isize {
    let mut shell = FakeShell::live(folder);
    let (window, _, subscription) = shell.open_window().await;
    shell.settle().await;
    let told = shell.window(subscription);
    assert_eq!(told.undo_arrangement, "", "a start has nothing to undo");
    let site = told
        .tabs
        .iter()
        .find(|tab| tab.kept && !tab.favourite && tab.group_id == 0)
        .expect("a Site in no Group")
        .tab_id;
    let group = told.groups.first().expect("a Group").group_id;
    let name = told.groups[0].name.clone();
    shell.forget();
    let before = held();
    for step in 0..50 {
        let even = step % 2 == 0;
        let answer = match fifty {
            Fifty::Moves => {
                let by = if even {
                    RowStep::Forward
                } else {
                    RowStep::Back
                };
                shell.move_tab(site, move_tab::To::By(by.into())).await
            }
            Fifty::Names => {
                shell
                    .rename_group(group, if even { "Play" } else { &name })
                    .await
            }
            Fifty::Favourites => shell.set_favourite(site, even).await,
        };
        acked(answer);
    }
    shell.settle().await;
    assert_ne!(
        shell.window(subscription).undo_arrangement,
        "",
        "there is a line to undo"
    );
    let _ = window;
    shell.forget();
    let after = held();
    drop(shell);
    after - before
}

#[test]
fn a_rows_line_of_things_to_undo_holds_what_changed_and_not_the_row() {
    run(async {
        for (sites, ceilings) in [(40, A_SHORT_ROWS_LINE), (2_000, A_LONG_ROWS_LINE)] {
            let folder = tempfile::tempdir().unwrap();
            a_long_row_made_before(folder.path(), sites).await;
            for (fifty, may) in [Fifty::Moves, Fifty::Names, Fifty::Favourites]
                .into_iter()
                .zip(ceilings)
            {
                let line = held_by_a_line_of(folder.path(), fifty).await;
                println!(
                    "a row of {sites} Sites, {fifty:?} fifty times: its line holds {line} bytes"
                );
                assert!(
                    line <= may,
                    "a row of {sites} Sites, {fifty:?} fifty times: its line holds {line} bytes, and may hold {may}"
                );
                // And not nothing: there are fifty things to undo, and a count that said
                // otherwise would be counting something else.
                assert!(line >= 2_000, "a line of fifty cannot be {line} bytes");
            }
        }
    });
}

/// So many made-up rules for requests, of five ordinary shapes (064).
fn made_up_rules(count: usize) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    for n in 0..count {
        match n % 5 {
            0 => writeln!(text, "||ads{n}.tracker{}.example.com^", n % 977),
            1 => writeln!(text, "/banner/{n}/*$image,third-party"),
            2 => writeln!(text, "||metrics{n}.example.net^$script,third-party"),
            3 => writeln!(text, "@@||good{n}.example.com^$script"),
            _ => writeln!(text, "-ad-unit-{n}."),
        }
        .unwrap();
    }
    text
}

#[test]
fn guards_blocker_holds_what_its_rules_for_requests_take_and_gives_it_back() {
    let rules = made_up_rules(100_000);
    let before = held();
    let guard = wt_guard::Guard::with_rules([rules.as_str()]);
    let with = held() - before;
    println!("a blocker of 100,000 made-up rules holds {with} bytes");
    let first_line = held() - before - with;
    assert!(
        with <= A_BLOCKER_OF_100_000_RULES,
        "a blocker of 100,000 made-up rules holds {with} bytes, and may hold {A_BLOCKER_OF_100_000_RULES}"
    );
    assert!(with >= 1_000_000, "100,000 rules cannot be {with} bytes");
    // Decisions leave nothing held but what the blocker keeps of having made them once:
    // the same asked again add nothing.
    let decide_all = || {
        for n in 0..200 {
            let url = format!(
                "https://ads{}.tracker{}.example.com/pixel.gif",
                n * 5,
                (n * 5) % 977
            );
            assert_eq!(
                guard.decide(
                    &url,
                    "https://news.example.com/story",
                    wt_guard::Kind::Image
                ),
                wt_guard::Verdict::Block
            );
        }
    };
    decide_all();
    let asked_once = held();
    decide_all();
    decide_all();
    let asked_again = held();
    // (Said after both were read: what a test prints is kept, on its own thread, until
    // the test ends, and would be counted between them.)
    println!(
        "two hundred decisions add {} bytes the first time they are made, and {} when asked again",
        asked_once - before - with - first_line,
        asked_again - asked_once
    );
    assert_eq!(
        asked_again, asked_once,
        "asked again, they leave nothing more held"
    );
    let printed = held() - asked_again;
    drop(guard);
    // Less what this test has printed so far, which is held with it.
    let left = held() - before - printed - first_line;
    println!("and {left} bytes are left when it has gone");
    assert!(
        left <= 64 * 1024,
        "a blocker that has gone leaves {left} bytes held"
    );

    // A blocker of no rules is nothing at all, however the having of none is said: with
    // none, nothing is looked up, which is how every app is until there are rules.
    for none in [vec![], vec![""], vec!["", "  \n\n"]] {
        let before = held();
        let guard = wt_guard::Guard::with_rules(none.iter().copied());
        assert_eq!(held(), before, "a blocker of no rules holds something");
        drop(guard);
    }

    // As many rules again that hide what is in a page: they are not Guard's yet (064 §2),
    // and a blocker holds nothing for them.
    use std::fmt::Write as _;
    let mut both = rules;
    for n in 0..100_000 {
        writeln!(both, "site{n}.example.com##.ad-banner-{n}").unwrap();
    }
    let before = held();
    let guard = wt_guard::Guard::with_rules([both.as_str()]);
    let with_hiding = held() - before;
    println!("with 100,000 rules more that hide a page's content: {with_hiding} bytes");
    assert!(
        with_hiding <= with + with / 20,
        "rules that hide a page's content are held: {with_hiding} bytes, where {with} are the rules for requests"
    );
    drop(guard);
}
