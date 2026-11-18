//! Which navigations a web view may make. This is the logic behind the fast
//! path's `wt_nav_check` (000 §5.5): it runs on the engine's own thread for
//! every navigation, so it borrows its input and never allocates.

/// What to do with a navigation a web view is about to make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationAction {
    /// Let the web view load it.
    Allow,
    /// Refuse it, silently.
    Cancel,
    /// Not a page: refuse it in the web view and hand the address to the
    /// system (`mailto:`, `tel:`, an app's own scheme).
    External,
}

/// Decides a navigation by its address alone.
///
/// - `http` and `https` load.
/// - `about` and `blob` load: they are the page's own documents.
/// - `data` loads in a frame, and is refused at the top level, where a page
///   could use it to show its own content under no address at all.
/// - `javascript` and `file` are refused: the first is not a navigation, and
///   WebTiger opens no local files yet.
/// - Anything else is for the system to open.
#[must_use]
pub fn check_navigation(url: &str, main_frame: bool) -> NavigationAction {
    let Some(scheme) = scheme_of(url) else {
        return NavigationAction::Cancel;
    };
    if is(scheme, "https") || is(scheme, "http") || is(scheme, "about") || is(scheme, "blob") {
        NavigationAction::Allow
    } else if is(scheme, "data") {
        if main_frame {
            NavigationAction::Cancel
        } else {
            NavigationAction::Allow
        }
    } else if is(scheme, "javascript") || is(scheme, "file") {
        NavigationAction::Cancel
    } else {
        NavigationAction::External
    }
}

/// The scheme of an absolute address: the letters, digits, `+`, `-` and `.`
/// before its first colon, starting with a letter (RFC 3986). Leading
/// whitespace and control characters are skipped, as the URL Standard strips
/// them before parsing: `"\tjavascript:"` must not read as scheme-less.
fn scheme_of(url: &str) -> Option<&str> {
    let url = url.trim_start_matches(|c: char| c <= ' ');
    let end = url.find(':')?;
    let scheme = url.get(..end)?;
    let mut bytes = scheme.bytes();
    if !bytes.next()?.is_ascii_alphabetic() {
        return None;
    }
    if bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.')) {
        Some(scheme)
    } else {
        None
    }
}

fn is(scheme: &str, name: &str) -> bool {
    scheme.eq_ignore_ascii_case(name)
}

#[cfg(test)]
mod tests {
    use super::NavigationAction::{Allow, Cancel, External};
    use super::*;

    #[test]
    fn pages_load() {
        for url in [
            "https://example.com/",
            "http://example.com/a?b#c",
            "HTTPS://EXAMPLE.COM/",
            "about:blank",
            "blob:https://example.com/3f2b8c1e",
        ] {
            assert_eq!(check_navigation(url, true), Allow, "{url}");
            assert_eq!(check_navigation(url, false), Allow, "{url}");
        }
    }

    #[test]
    fn data_loads_only_in_a_frame() {
        let url = "data:text/html,<h1>Sign in</h1>";
        assert_eq!(check_navigation(url, true), Cancel);
        assert_eq!(check_navigation(url, false), Allow);
    }

    #[test]
    fn scripts_and_local_files_are_refused() {
        for url in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "\tjavascript:alert(1)",
            " \n javascript:alert(1)",
            "file:///etc/passwd",
        ] {
            assert_eq!(check_navigation(url, true), Cancel, "{url:?}");
            assert_eq!(check_navigation(url, false), Cancel, "{url:?}");
        }
    }

    #[test]
    fn other_schemes_go_to_the_system() {
        for url in [
            "mailto:ada@example.com",
            "tel:+15551234567",
            "itms-apps://apps.apple.com/app/id1",
            "intent://scan/#Intent;scheme=zxing;end",
            "x-my.app+v2://open",
        ] {
            assert_eq!(check_navigation(url, true), External, "{url}");
        }
    }

    #[test]
    fn what_has_no_scheme_is_refused() {
        for url in [
            "",
            "example.com",
            "/path",
            "://x",
            "1http://x",
            "ht tp://x",
            "é:x",
        ] {
            assert_eq!(check_navigation(url, true), Cancel, "{url:?}");
        }
    }
}
