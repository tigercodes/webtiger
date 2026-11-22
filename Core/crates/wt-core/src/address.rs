//! Addresses: what the user typed turned into something a web view can
//! load, and what a web view loaded turned into something the address field
//! can show without lying (000 §5.13).
//!
//! Two rules shape this file:
//!
//! - **Typed text is never executed.** `javascript:` and every scheme other
//!   than `http`, `https` and `about:blank` become a search for those words.
//! - **The shown host is the real host.** It is always the ASCII form, so a
//!   look-alike international name reads as `xn--…`, and anything before an
//!   `@` is left out, so `https://bank.example@evil.example/` shows
//!   `evil.example`.

use std::net::Ipv4Addr;
use std::str::FromStr as _;

use url::{Host, Position, Url};

/// An absolute address, parsed by the WHATWG URL Standard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Address {
    url: Url,
}

/// What typed text turned out to be.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// Somewhere to go.
    Address(Address),
    /// Words to search for, trimmed.
    Search(String),
}

/// How much the connection to an address can be trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Security {
    /// `https`.
    Secure,
    /// `http`.
    Insecure,
    /// The browser's own pages and local content: `about:`, `data:`, `blob:`.
    Local,
}

/// The text is not an absolute address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddressError;

impl Address {
    /// Parses an absolute address, as an engine reports one.
    ///
    /// # Errors
    /// When the text is not an absolute address.
    pub fn parse(text: &str) -> Result<Address, AddressError> {
        Url::parse(text)
            .map(|url| Address { url })
            .map_err(|_| AddressError)
    }

    /// An address of the web, as a page's link or another app hands one
    /// over (018, 031): absolute, and `http` or `https`. `None` for anything
    /// else. It is not typed text: nothing is mended, no scheme is supplied,
    /// and what is not an address of the web is not one to search for.
    #[must_use]
    pub fn web(text: &str) -> Option<Address> {
        Address::parse(text)
            .ok()
            .filter(|address| matches!(address.scheme(), "http" | "https"))
    }

    /// The whole address, normalised.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    #[must_use]
    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    /// The host in its ASCII form (an IPv6 address in brackets). `None` for
    /// an address without one, such as `about:blank`.
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.url.host_str().filter(|host| !host.is_empty())
    }

    /// The part of the host someone registered: `example.co.uk` for
    /// `www.example.co.uk`. `None` for an IP address, a single-label name
    /// such as `localhost`, and a host that is itself a public suffix.
    #[must_use]
    pub fn registrable_domain(&self) -> Option<&str> {
        match self.url.host()? {
            Host::Domain(domain) => psl::domain_str(domain.trim_end_matches('.')),
            Host::Ipv4(_) | Host::Ipv6(_) => None,
        }
    }

    /// The path, as it is written in the address: escapes and all, without
    /// what follows a `?` or a `#`.
    #[must_use]
    pub fn path(&self) -> &str {
        self.url.path()
    }

    /// The site a page of this address is of (012): what someone registered,
    /// or, for a host with nothing registered (an IP address, `localhost`),
    /// the host itself. Whatever the scheme and the port. `None` for an
    /// address with no host.
    #[must_use]
    pub fn site(&self) -> Option<&str> {
        self.registrable_domain().or_else(|| self.host())
    }

    /// Who the page is to the web's own rules (014): its scheme, host and
    /// port, as `https://example.com`, with a port only when it is not the
    /// scheme's own. `None` for an address that is not the web's.
    #[must_use]
    pub fn origin(&self) -> Option<String> {
        if !matches!(self.url.scheme(), "http" | "https") {
            return None;
        }
        self.host()?;
        Some(self.url.origin().ascii_serialization())
    }

    /// Whether what arrives from the address can be taken to be the
    /// address's own (014): it came over `https`, or it never left this
    /// device (`localhost`, a loopback address).
    #[must_use]
    pub fn is_trustworthy(&self) -> bool {
        match self.url.scheme() {
            "https" => true,
            "http" => match self.url.host() {
                Some(Host::Domain(name)) => name == "localhost" || name.ends_with(".localhost"),
                Some(Host::Ipv4(address)) => address.is_loopback(),
                Some(Host::Ipv6(address)) => address.is_loopback(),
                None => false,
            },
            _ => false,
        }
    }

    #[must_use]
    pub fn security(&self) -> Security {
        match self.url.scheme() {
            "https" => Security::Secure,
            "http" => Security::Insecure,
            _ => Security::Local,
        }
    }

    /// What the address field shows while it is not being edited: the host
    /// and what follows it, with the scheme only when it is not `https`, and
    /// never the part before an `@`.
    #[must_use]
    pub fn display(&self) -> String {
        let scheme = self.url.scheme();
        if (scheme != "https" && scheme != "http") || self.host().is_none() {
            return self.url.as_str().to_owned();
        }
        let mut rest = &self.url[Position::BeforeHost..];
        if self.url.path() == "/" && self.url.query().is_none() && self.url.fragment().is_none() {
            rest = rest.strip_suffix('/').unwrap_or(rest);
        }
        if scheme == "https" {
            rest.to_owned()
        } else {
            format!("http://{rest}")
        }
    }
}

/// Decides what typed text is. `None` when it is empty.
#[must_use]
pub fn classify(text: &str) -> Option<Input> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    Some(match fix_up(text) {
        Some(address) => Input::Address(address),
        None => Input::Search(text.to_owned()),
    })
}

/// The address a search for `query` loads, from a provider's template in
/// which `{query}` stands for the words. `None` when the template has no
/// `{query}` or does not make an `http` or `https` address.
#[must_use]
pub fn search_address(template: &str, query: &str) -> Option<Address> {
    if !template.contains("{query}") {
        return None;
    }
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let address = Address::parse(&template.replace("{query}", &encoded)).ok()?;
    matches!(address.scheme(), "https" | "http").then_some(address)
}

/// The address `text` means, or `None` when it is words.
fn fix_up(text: &str) -> Option<Address> {
    if let Some((scheme, rest)) = split_scheme(text) {
        // "localhost:8080" and "example.com:8080/x" are a host and a port.
        if !starts_with_port(rest) {
            return explicit(scheme, text);
        }
    }
    bare_host(text)
}

/// Splits `scheme:rest` when the part before the first colon has a scheme's
/// spelling (RFC 3986).
fn split_scheme(text: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = text.split_once(':')?;
    let mut bytes = scheme.bytes();
    if !bytes.next()?.is_ascii_alphabetic() {
        return None;
    }
    bytes
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
        .then_some((scheme, rest))
}

/// Whether `rest` (what follows a colon) begins with a port: one to five
/// digits, then the end or the start of a path, query or fragment.
fn starts_with_port(rest: &str) -> bool {
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    (1..=5).contains(&digits)
        && matches!(rest.as_bytes().get(digits), None | Some(b'/' | b'?' | b'#'))
}

/// Text that names its scheme. Only pages load from the address field.
fn explicit(scheme: &str, text: &str) -> Option<Address> {
    if scheme.eq_ignore_ascii_case("https") || scheme.eq_ignore_ascii_case("http") {
        let address = Address::parse(text).ok()?;
        address.host()?;
        Some(address)
    } else if text.eq_ignore_ascii_case("about:blank") {
        Address::parse("about:blank").ok()
    } else {
        None
    }
}

/// Text without a scheme: an address when it begins with something that
/// reads as a host.
fn bare_host(text: &str) -> Option<Address> {
    if text.chars().any(char::is_whitespace) {
        return None;
    }
    let authority_end = text.find(['/', '?', '#']).unwrap_or(text.len());
    let authority = text.get(..authority_end)?;
    // Only a slash says "this is a host": "what?" and "#tag" are words.
    let has_path = text.as_bytes().get(authority_end) == Some(&b'/');
    // "ada@example.com" is far more often an email address than a login.
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    let (host, has_port) = split_port(authority)?;
    let scheme = match host_kind(host) {
        HostKind::Local => "http",
        HostKind::Name => "https",
        // "intranet/" and "intranet:8080" say "this is a host"; "intranet"
        // alone is a word.
        HostKind::SingleLabel if has_port || has_path => "http",
        HostKind::SingleLabel | HostKind::Words => return None,
    };
    let address = Address::parse(&format!("{scheme}://{text}")).ok()?;
    address.host()?;
    Some(address)
}

/// Splits `host:port`, leaving an IPv6 address's own colons alone. `None`
/// when what follows the host is not a port.
fn split_port(authority: &str) -> Option<(&str, bool)> {
    if authority.starts_with('[') {
        let end = authority.find(']')?;
        let host = authority.get(..=end)?;
        return match authority.get(end + 1..)? {
            "" => Some((host, false)),
            rest => rest
                .strip_prefix(':')
                .is_some_and(starts_with_port)
                .then_some((host, true)),
        };
    }
    match authority.rsplit_once(':') {
        None => Some((authority, false)),
        Some((host, port)) => {
            (starts_with_port(port) && !host.contains(':')).then_some((host, true))
        }
    }
}

enum HostKind {
    /// `localhost` or an IP address: reached over plain `http` by default.
    Local,
    /// A name under a known public suffix.
    Name,
    /// One label: a word, or a machine on the local network.
    SingleLabel,
    /// Dots, but no known suffix: `readme.txt`, `3.14`, `e.g.`.
    Words,
}

fn host_kind(host: &str) -> HostKind {
    let host = host.strip_suffix('.').unwrap_or(host);
    let lower = host.to_lowercase();
    if lower == "localhost" || lower.ends_with(".localhost") || host.starts_with('[') {
        return HostKind::Local;
    }
    let Some((_, last)) = lower.rsplit_once('.') else {
        return HostKind::SingleLabel;
    };
    if lower.split('.').any(str::is_empty) {
        return HostKind::Words;
    }
    // A name never ends in a number, and the URL parser reads one that does
    // as an IPv4 address in any of its historic spellings. Only the plain
    // dotted quad is an address here; "3.14" and "1.2.3" are words.
    if last.bytes().all(|b| b.is_ascii_digit()) {
        return if Ipv4Addr::from_str(&lower).is_ok() {
            HostKind::Local
        } else {
            HostKind::Words
        };
    }
    if has_known_suffix(&lower) {
        HostKind::Name
    } else {
        HostKind::Words
    }
}

/// Whether the name ends in a suffix the Public Suffix List knows, in the
/// form typed or in its ASCII form.
fn has_known_suffix(lower: &str) -> bool {
    let known = |name: &str| psl::suffix(name.as_bytes()).is_some_and(|suffix| suffix.is_known());
    if known(lower) {
        return true;
    }
    match Host::parse(lower) {
        Ok(Host::Domain(ascii)) => known(&ascii),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address(text: &str) -> String {
        match classify(text) {
            Some(Input::Address(address)) => address.as_str().to_owned(),
            other => panic!("{text:?} should be an address, was {other:?}"),
        }
    }

    fn search(text: &str) -> String {
        match classify(text) {
            Some(Input::Search(words)) => words,
            other => panic!("{text:?} should be a search, was {other:?}"),
        }
    }

    #[test]
    fn an_address_handed_over_is_one_of_the_web_or_none() {
        let web = |text: &str| Address::web(text).map(|address| address.as_str().to_owned());
        assert_eq!(
            web("https://example.com/a?b=c#d").as_deref(),
            Some("https://example.com/a?b=c#d")
        );
        assert_eq!(
            web("http://example.com").as_deref(),
            Some("http://example.com/")
        );
        assert_eq!(
            web("HTTPS://Example.COM/Path").as_deref(),
            Some("https://example.com/Path")
        );
        // Not mended, as typed text would be: no scheme is supplied, and
        // words are not an address.
        for not_web in [
            "",
            " ",
            "example.com",
            "www.example.com/page",
            "//example.com/",
            "tigers in the wild",
            "https://",
            "http:",
            "https://exa mple.com/",
            "javascript:alert(1)",
            "data:text/html,<p>hello",
            "file:///etc/passwd",
            "about:blank",
            "blob:https://example.com/1234",
            "content://media/external/file/1",
            "intent://example.com/#Intent;scheme=https;end",
            "mailto:somebody@example.com",
            "tel:+15550100",
            "ftp://example.com/file",
            "ws://example.com/",
            "view-source:https://example.com/",
            "webtiger://open?url=https://example.com/",
        ] {
            assert_eq!(web(not_web), None, "{not_web:?}");
        }
    }

    #[test]
    fn nothing_typed_is_nothing() {
        assert_eq!(classify(""), None);
        assert_eq!(classify("  \t\n"), None);
    }

    #[test]
    fn a_name_under_a_known_suffix_is_https() {
        assert_eq!(address("example.com"), "https://example.com/");
        assert_eq!(address("  example.com  "), "https://example.com/");
        assert_eq!(
            address("Example.COM/Path?q=1#top"),
            "https://example.com/Path?q=1#top"
        );
        assert_eq!(address("www.example.co.uk"), "https://www.example.co.uk/");
        assert_eq!(address("example.com:8443/x"), "https://example.com:8443/x");
        assert_eq!(address("example.com."), "https://example.com./");
        // A public suffix that is itself a site.
        assert_eq!(address("github.io"), "https://github.io/");
    }

    #[test]
    fn an_explicit_page_scheme_is_kept() {
        assert_eq!(address("http://example.com"), "http://example.com/");
        assert_eq!(
            address("HTTPS://Example.com/a b"),
            "https://example.com/a%20b"
        );
        assert_eq!(address("http://intranet"), "http://intranet/");
        assert_eq!(address("about:blank"), "about:blank");
        assert_eq!(address("ABOUT:BLANK"), "about:blank");
    }

    #[test]
    fn local_hosts_are_http() {
        assert_eq!(address("localhost"), "http://localhost/");
        assert_eq!(address("localhost:8080"), "http://localhost:8080/");
        assert_eq!(
            address("app.localhost:3000/x"),
            "http://app.localhost:3000/x"
        );
        assert_eq!(address("127.0.0.1"), "http://127.0.0.1/");
        assert_eq!(
            address("192.168.1.10:8080/admin"),
            "http://192.168.1.10:8080/admin"
        );
        assert_eq!(address("[::1]"), "http://[::1]/");
        assert_eq!(address("[::1]:8080/x"), "http://[::1]:8080/x");
    }

    #[test]
    fn a_single_label_is_a_host_only_when_it_says_so() {
        assert_eq!(search("intranet"), "intranet");
        assert_eq!(address("intranet/"), "http://intranet/");
        assert_eq!(address("intranet:8080"), "http://intranet:8080/");
        assert_eq!(address("printer/status"), "http://printer/status");
    }

    #[test]
    fn words_are_a_search() {
        for text in [
            "rust borrow checker",
            "what is example.com",
            "readme.txt2",
            "3.14",
            "1.2.3",
            "e.g.",
            ".com",
            "foo..com",
            "ada@example.com",
            "node.js vs deno",
            "c++",
            "what?",
            "#hashtag",
            "why#not",
            "/usr/local/bin",
            "example.com:port",
            "example.com:123456",
        ] {
            assert_eq!(search(text), text, "{text:?}");
        }
    }

    #[test]
    fn typed_text_is_never_executed_or_handed_to_another_scheme() {
        for text in [
            "javascript:alert(document.cookie)",
            "JAVASCRIPT:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "file:///etc/passwd",
            "mailto:ada@example.com",
            "view-source:https://example.com",
            "about:config",
            "chrome://settings",
            "C:\\Users\\ada",
            "foo:bar",
        ] {
            assert_eq!(search(text), text, "{text:?}");
        }
    }

    #[test]
    fn an_explicit_scheme_with_a_bad_host_is_a_search() {
        assert_eq!(search("https://"), "https://");
        assert_eq!(search("http://exa mple.com"), "http://exa mple.com");
    }

    #[test]
    fn international_names_load_and_show_as_ascii() {
        assert_eq!(address("münchen.de"), "https://xn--mnchen-3ya.de/");
        assert_eq!(address("xn--mnchen-3ya.de"), "https://xn--mnchen-3ya.de/");
        let Some(Input::Address(shown)) = classify("аpple.com") else {
            panic!("a Cyrillic look-alike is still an address");
        };
        // The first letter is Cyrillic: the field must not show "apple.com".
        assert_eq!(shown.host(), Some("xn--pple-43d.com"));
        assert_eq!(shown.display(), "xn--pple-43d.com");
    }

    #[test]
    fn an_international_suffix_is_known_in_either_form() {
        assert_eq!(address("例え.中国"), "https://xn--r8jz45g.xn--fiqs8s/");
        assert_eq!(
            address("xn--r8jz45g.xn--fiqs8s"),
            "https://xn--r8jz45g.xn--fiqs8s/"
        );
    }

    #[test]
    fn an_origin_is_the_scheme_the_host_and_a_port_that_is_not_the_schemes_own() {
        let origin = |text: &str| Address::parse(text).unwrap().origin();
        for (address, expected) in [
            (
                "https://meet.example.com/room/7?x=1#top",
                "https://meet.example.com",
            ),
            ("https://meet.example.com", "https://meet.example.com"),
            ("https://MEET.Example.com:443/", "https://meet.example.com"),
            (
                "https://meet.example.com:8443/",
                "https://meet.example.com:8443",
            ),
            ("http://example.com:80/", "http://example.com"),
            ("http://127.0.0.1:8080/page", "http://127.0.0.1:8080"),
            // Never the part before an `@`.
            ("https://someone:secret@example.com/", "https://example.com"),
        ] {
            assert_eq!(origin(address).as_deref(), Some(expected), "{address}");
        }
        // One of a site's names is not another, and nor is another scheme.
        assert_ne!(
            origin("https://a.example.com/"),
            origin("https://b.example.com/")
        );
        assert_ne!(
            origin("https://example.com/"),
            origin("http://example.com/")
        );
        // An address that is not the web's has none.
        for address in [
            "about:blank",
            "data:text/plain,hello",
            "blob:https://example.com/1",
            "file:///tmp/a",
        ] {
            assert_eq!(origin(address), None, "{address}");
        }
    }

    #[test]
    fn an_address_is_its_own_over_https_or_when_it_never_left_the_device() {
        let trusted = |text: &str| Address::parse(text).unwrap().is_trustworthy();
        for address in [
            "https://example.com/",
            "http://localhost/",
            "http://localhost:3000/app",
            "http://dev.localhost/",
            "http://127.0.0.1:8080/",
            "http://127.9.9.9/",
            "http://[::1]:8080/",
        ] {
            assert!(trusted(address), "{address}");
        }
        for address in [
            "http://example.com/",
            "http://192.168.1.10/",
            "http://10.0.0.1/",
            "http://localhost.example.com/",
            "http://notlocalhost/",
            "http://[2001:db8::1]/",
            "about:blank",
            "data:text/html,hi",
            "file:///tmp/a.html",
        ] {
            assert!(!trusted(address), "{address}");
        }
    }

    #[test]
    fn a_site_is_what_was_registered_or_else_the_host() {
        let site = |url: &str| Address::parse(url).unwrap().site().map(str::to_owned);
        assert_eq!(
            site("https://www.example.co.uk/a"),
            Some("example.co.uk".into())
        );
        assert_eq!(
            site("http://shop.example.co.uk:8080/"),
            Some("example.co.uk".into())
        );
        assert_eq!(site("https://example.com/"), Some("example.com".into()));
        // Nothing registered: the host is its own site, without its port.
        assert_eq!(site("http://127.0.0.1:8080/"), Some("127.0.0.1".into()));
        assert_eq!(site("http://localhost:3000/"), Some("localhost".into()));
        assert_eq!(site("about:blank"), None);
    }

    #[test]
    fn the_registrable_domain_follows_the_public_suffix_list() {
        let domain = |text: &str| {
            Address::parse(text)
                .unwrap()
                .registrable_domain()
                .map(str::to_owned)
        };
        assert_eq!(
            domain("https://www.example.com/"),
            Some("example.com".to_owned())
        );
        assert_eq!(
            domain("https://a.b.example.co.uk/"),
            Some("example.co.uk".to_owned())
        );
        assert_eq!(
            domain("https://ada.github.io/"),
            Some("ada.github.io".to_owned())
        );
        assert_eq!(
            domain("https://example.com./"),
            Some("example.com".to_owned())
        );
        assert_eq!(domain("https://github.io/"), None);
        assert_eq!(domain("http://localhost/"), None);
        assert_eq!(domain("http://127.0.0.1/"), None);
        assert_eq!(domain("http://[::1]/"), None);
        assert_eq!(domain("about:blank"), None);
    }

    #[test]
    fn the_field_shows_the_host_and_never_the_part_before_an_at_sign() {
        let display = |text: &str| Address::parse(text).unwrap().display();
        assert_eq!(display("https://example.com/"), "example.com");
        assert_eq!(
            display("https://example.com/a?b=1#c"),
            "example.com/a?b=1#c"
        );
        assert_eq!(display("https://example.com/?q=1"), "example.com/?q=1");
        assert_eq!(display("https://example.com:8443/"), "example.com:8443");
        assert_eq!(display("http://example.com/"), "http://example.com");
        assert_eq!(
            display("http://localhost:8080/x"),
            "http://localhost:8080/x"
        );
        assert_eq!(display("about:blank"), "about:blank");
        assert_eq!(
            display("https://accounts.bank.example@evil.example/login"),
            "evil.example/login"
        );
        assert_eq!(display("https://user:secret@evil.example/"), "evil.example");
    }

    #[test]
    fn security_follows_the_scheme() {
        let security = |text: &str| Address::parse(text).unwrap().security();
        assert_eq!(security("https://example.com/"), Security::Secure);
        assert_eq!(security("http://example.com/"), Security::Insecure);
        assert_eq!(security("about:blank"), Security::Local);
        assert_eq!(security("blob:https://example.com/1"), Security::Local);
    }

    #[test]
    fn a_search_address_encodes_the_words() {
        let made = search_address("https://search.example/?q={query}", "c++ & rust/go é").unwrap();
        assert_eq!(
            made.as_str(),
            "https://search.example/?q=c%2B%2B+%26+rust%2Fgo+%C3%A9"
        );
        assert_eq!(search_address("https://search.example/", "x"), None);
        assert_eq!(search_address("javascript:{query}", "alert(1)"), None);
        assert_eq!(search_address("{query}", "file:///etc/passwd"), None);
    }

    /// RUSTSEC-2026-0592. A page can name a `blob:` of a `blob:` of a `blob:`, as deep
    /// as it likes. Before `url` 2.5.9 the library asked for the origin of such an
    /// address once for every `blob:`, each inside the one before, and the process
    /// ended when the stack did. The core asks it only of `http` and `https`; the last
    /// lines ask the library itself, as nothing in the core does.
    #[test]
    fn a_blob_nested_ten_thousand_deep_is_an_address_like_any_other() {
        let text = format!("{}https://example.com/", "blob:".repeat(10_000));
        let address = Address::parse(&text).unwrap();
        assert_eq!(address.origin(), None, "a blob is no origin of the web's");
        assert_eq!(address.site(), None);
        assert_eq!(address.security(), Security::Local);
        let asked_directly = url::Url::parse(&text).unwrap().origin();
        assert!(
            !asked_directly.is_tuple(),
            "an origin of its own, and no site's"
        );
    }
}
