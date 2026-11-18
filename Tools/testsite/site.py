#!/usr/bin/env python3
"""The test site (000 §8.2): the pages every shell's suite loads, from Fixtures/testsite/site.json.

  Tools/testsite/site.py              writes the pages into each suite's own loopback server:
      Swift    Packages/Tests/WTWebKitTests/TestSitePages.generated.swift
      Kotlin   Android/app/src/androidTest/kotlin/com/tigercodes/webtiger/TestSitePages.kt
  Tools/testsite/site.py --check      writes nothing, and fails when a checked-in file is stale
  Tools/testsite/site.py serve [PORT] serves the site on 127.0.0.1 (port 8765 unless given)
  Tools/testsite/site.py self-test    serves it on a free port and checks what it answers

The suites keep a server of their own, in the test's process: a suite then needs nothing
running beside it, a sandboxed test runner can reach it, and its pages are on 127.0.0.1, a
secure context. What they serve is this one definition, generated and checked in;
Tools/test.sh lint runs --check and self-test. `serve` is for a run by hand: an app started in
fake mode with one of these pages as its start address.
"""
import http.client
import json
import pathlib
import re
import socketserver
import sys
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "Fixtures" / "testsite" / "site.json"
SWIFT = ROOT / "Packages" / "Tests" / "WTWebKitTests" / "TestSitePages.generated.swift"
KOTLIN = ROOT / "Android" / "app" / "src" / "androidTest" / "kotlin" / "com" / "tigercodes" / "webtiger" / "TestSitePages.kt"

KEYS = {"path", "title", "body", "content", "type", "status", "headers", "delay_ms", "note"}
PLACEHOLDER = re.compile(r"\{(query|cookie):([A-Za-z_]+)\|([^}]*)\}")


def text_of(value) -> str:
    """A string, or a list of lines."""
    return "\n".join(value) if isinstance(value, list) else value


def pieces(text: str) -> list:
    """`text` cut where a request's value goes: ("text", words), ("query" or "cookie", name, default)."""
    out, at = [], 0
    for match in PLACEHOLDER.finditer(text):
        if match.start() > at:
            out.append(("text", text[at : match.start()]))
        out.append((match.group(1), match.group(2), match.group(3)))
        at = match.end()
    if at < len(text):
        out.append(("text", text[at:]))
    return out


def page(entry: dict) -> dict:
    unknown = set(entry) - KEYS
    if unknown:
        raise SystemExit(f"{entry.get('path', 'not_found')}: unknown keys {sorted(unknown)}")
    if ("title" in entry) == ("content" in entry):
        raise SystemExit(f"{entry.get('path', 'not_found')}: a page has a title or content, and not both")
    if "title" in entry:
        if "type" in entry:
            raise SystemExit(f"{entry['path']}: a page with a title is text/html")
        title = entry["title"]
        body = f"<!doctype html><title>{title}</title><h1>{title}</h1>{text_of(entry.get('body', ''))}"
        kind = "text/html"
    else:
        if "body" in entry:
            raise SystemExit(f"{entry.get('path', 'not_found')}: 'body' goes with a title; this page has content")
        body = text_of(entry["content"])
        kind = entry.get("type", "text/html")
    headers = list(entry.get("headers", []))
    # Nothing is kept by a cache unless the page says it may be: a test sees every request.
    if not any(header.lower().startswith("cache-control:") for header in headers):
        headers.insert(0, "Cache-Control: no-store")
    return {
        "path": entry.get("path", ""),
        "status": entry.get("status", "200 OK"),
        "type": kind,
        "headers": [pieces(header) for header in headers],
        "body": pieces(body),
        "delay_ms": int(entry.get("delay_ms", 0)),
    }


def load() -> dict:
    data = json.loads(SOURCE.read_text(encoding="utf-8"))
    pages = [page(entry) for entry in data["pages"]]
    paths = [entry["path"] for entry in pages]
    for path in paths:
        if not path.startswith("/") or "?" in path:
            raise SystemExit(f"{path!r} is not a path")
    if len(set(paths)) != len(paths):
        raise SystemExit("two pages have one path")
    if "path" in data["not_found"]:
        raise SystemExit("not_found has no path: it answers every path no page has")
    return {"pages": pages, "not_found": page(data["not_found"])}


# The generated files


def quoted(text: str, kotlin: bool) -> str:
    out = text.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
    if kotlin:
        out = out.replace("$", "\\$")
    return f'"{out}"'


def swift_pieces(parts: list) -> str:
    def one(part):
        if part[0] == "text":
            return f".text({quoted(part[1], False)})"
        return f".{part[0]}({quoted(part[1], False)}, or: {quoted(part[2], False)})"

    return "[" + ", ".join(one(part) for part in parts) + "]"


def swift_page(entry: dict, indent: str) -> list:
    headers = ", ".join(swift_pieces(header) for header in entry["headers"])
    return [
        f"{indent}TestSitePage(",
        f"{indent}    path: {quoted(entry['path'], False)},",
        f"{indent}    status: {quoted(entry['status'], False)},",
        f"{indent}    type: {quoted(entry['type'], False)},",
        f"{indent}    headers: [{headers}],",
        f"{indent}    body: {swift_pieces(entry['body'])},",
        f"{indent}    delayMilliseconds: {entry['delay_ms']}",
        f"{indent})",
    ]


def swift(site: dict) -> str:
    out = [
        "// Generated by Tools/testsite/site.py from Fixtures/testsite/site.json. Do not edit.",
        "",
        "/// The test site's pages. TestSite.swift serves them.",
        "enum TestSitePages {",
        "    static let all: [TestSitePage] = [",
    ]
    for entry in site["pages"]:
        lines = swift_page(entry, "        ")
        lines[-1] += ","
        out += lines
    out += ["    ]", "", "    /// What every other path is answered with."]
    lines = swift_page(site["not_found"], "    ")
    lines[0] = "    static let notFound = " + lines[0].lstrip()
    out += lines
    out += ["}", ""]
    return "\n".join(out)


def kotlin_pieces(parts: list) -> str:
    def one(part):
        if part[0] == "text":
            return f"TestSitePiece.Text({quoted(part[1], True)})"
        return f"TestSitePiece.{part[0].capitalize()}({quoted(part[1], True)}, or = {quoted(part[2], True)})"

    return "listOf(" + ", ".join(one(part) for part in parts) + ")"


def kotlin_page(entry: dict, indent: str) -> list:
    headers = ", ".join(kotlin_pieces(header) for header in entry["headers"])
    return [
        f"{indent}TestSitePage(",
        f"{indent}    path = {quoted(entry['path'], True)},",
        f"{indent}    status = {quoted(entry['status'], True)},",
        f"{indent}    type = {quoted(entry['type'], True)},",
        f"{indent}    headers = listOf({headers}),",
        f"{indent}    body = {kotlin_pieces(entry['body'])},",
        f"{indent}    delayMilliseconds = {entry['delay_ms']},",
        f"{indent})",
    ]


def kotlin(site: dict) -> str:
    out = [
        "// Generated by Tools/testsite/site.py from Fixtures/testsite/site.json. Do not edit.",
        "package com.tigercodes.webtiger",
        "",
        "/** The test site's pages. TestSite.kt serves them. */",
        "internal object TestSitePages {",
        "    val all: List<TestSitePage> =",
        "        listOf(",
    ]
    for entry in site["pages"]:
        lines = kotlin_page(entry, "            ")
        lines[-1] += ","
        out += lines
    out += ["        )", "", "    /** What every other path is answered with. */", "    val notFound ="]
    out += kotlin_page(site["not_found"], "        ")
    out += ["}", ""]
    return "\n".join(out)


def generate(check: bool) -> int:
    site = load()
    stale = []
    for path, text in ((SWIFT, swift(site)), (KOTLIN, kotlin(site))):
        if path.exists() and path.read_text(encoding="utf-8") == text:
            continue
        stale.append(path)
        if not check:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
            print(f"wrote {path.relative_to(ROOT)}")
    if check and stale:
        for path in stale:
            print(f"{path.relative_to(ROOT)} is stale; run Tools/testsite/site.py", file=sys.stderr)
        return 1
    if not stale:
        print("the test site's pages are up to date")
    return 0


# The site, served from here


def filled(parts: list, query: dict, cookies: dict) -> str:
    out = []
    for part in parts:
        if part[0] == "text":
            out.append(part[1])
        else:
            out.append((query if part[0] == "query" else cookies).get(part[1], part[2]))
    return "".join(out)


def pairs(text: str, separator: str) -> dict:
    """name=value pairs; the first of a name wins, as in the shells' servers."""
    out = {}
    for pair in text.split(separator):
        name, equals, value = pair.strip().partition("=")
        if equals and name not in out:
            out[name] = value
    return out


def respond(site: dict, head: str) -> tuple:
    """The whole answer to a request's head, and how long to wait before giving it."""
    lines = head.split("\r\n")
    words = lines[0].split(" ")
    target = words[1] if len(words) > 1 else "/"
    path, _, query_text = target.partition("?")
    cookie_line = next((line for line in lines[1:] if line.lower().startswith("cookie:")), "")
    query = pairs(query_text, "&")
    cookies = pairs(cookie_line[len("cookie:") :], ";")
    entry = next((candidate for candidate in site["pages"] if candidate["path"] == path), site["not_found"])
    body = filled(entry["body"], query, cookies).encode("utf-8")
    headers = [
        f"HTTP/1.1 {entry['status']}",
        f"Content-Type: {entry['type']}; charset=utf-8",
        f"Content-Length: {len(body)}",
        "Connection: close",
    ] + [filled(header, query, cookies) for header in entry["headers"]]
    return ("\r\n".join(headers) + "\r\n\r\n").encode("latin-1") + body, entry["delay_ms"] / 1000


class Handler(socketserver.BaseRequestHandler):
    def handle(self):
        received = b""
        while b"\r\n\r\n" not in received:
            chunk = self.request.recv(4096)
            if not chunk:
                return
            received += chunk
        answer, delay = respond(self.server.site, received.split(b"\r\n\r\n", 1)[0].decode("latin-1"))
        if delay:
            time.sleep(delay)
        self.request.sendall(answer)


class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


def server(port: int) -> Server:
    made = Server(("127.0.0.1", port), Handler)
    made.site = load()
    return made


def serve(port: int) -> int:
    with server(port) as running:
        print(f"the test site is at http://127.0.0.1:{running.server_address[1]}/ (Ctrl-C stops it)", flush=True)
        try:
            running.serve_forever()
        except KeyboardInterrupt:
            pass
    return 0


def self_test() -> int:
    """What the definition means, checked through a real connection."""
    running = server(0)
    threading.Thread(target=running.serve_forever, daemon=True).start()
    port = running.server_address[1]
    failures = []

    def get(target: str, cookie: str = ""):
        connection = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
        connection.request("GET", target, headers={"Cookie": cookie} if cookie else {})
        answer = connection.getresponse()
        body = answer.read().decode("utf-8")
        connection.close()
        return answer, body

    def expect(what: str, got, wanted):
        if got != wanted:
            failures.append(f"{what}: {got!r}, not {wanted!r}")

    answer, body = get("/")
    expect("/ status", answer.status, 200)
    expect("/ type", answer.getheader("Content-Type"), "text/html; charset=utf-8")
    expect("/ is never cached", answer.getheader("Cache-Control"), "no-store")
    expect("/ body", body, '<!doctype html><title>Home</title><h1>Home</h1><a id="next" href="/landed">next</a>')
    expect("/ length", answer.getheader("Content-Length"), str(len(body.encode("utf-8"))))

    answer, body = get("/set?v=ada")
    expect("/set signs in", answer.getheader("Set-Cookie"), "who=ada; Path=/; Max-Age=3600")
    answer, body = get("/set")
    expect("/set without a value", answer.getheader("Set-Cookie"), "who=someone; Path=/; Max-Age=3600")

    answer, body = get("/whoami", cookie="theme=dark; who=ada")
    expect("/whoami with the cookie", body, "<!doctype html><title>who=ada</title><h1>who=ada</h1>")
    answer, body = get("/whoami")
    expect("/whoami without it", body, "<!doctype html><title>who=none</title><h1>who=none</h1>")

    answer, body = get("/redirect")
    expect("/redirect status", answer.status, 302)
    expect("/redirect goes to", answer.getheader("Location"), "/landed")
    expect("/redirect body", body, "")

    answer, body = get("/worker.js")
    expect("/worker.js type", answer.getheader("Content-Type"), "text/javascript; charset=utf-8")

    answer, body = get("/session?v=ada")
    expect("/session signs in with no expiry", answer.getheader("Set-Cookie"), "who=ada; Path=/")

    answer, body = get("/cached.js")
    expect("/cached.js may be kept", answer.getheader("Cache-Control"), "max-age=3600")

    answer, body = get("/storage?v=ada")
    expect("/storage keeps its lines", body.count("\n"), 4)
    expect("/storage is a page", body.startswith("<!doctype html><title>Storage</title>"), True)

    answer, body = get("/nowhere")
    expect("a path no page has", answer.status, 404)
    expect("its body", body, "<title>Not found</title>")

    running.shutdown()
    running.server_close()
    for failure in failures:
        print(f"test site: {failure}", file=sys.stderr)
    if not failures:
        print("the test site answers as its definition says")
    return 1 if failures else 0


def main(arguments: list) -> int:
    if not arguments:
        return generate(check=False)
    if arguments == ["--check"]:
        return generate(check=True)
    if arguments[0] == "serve" and len(arguments) <= 2:
        return serve(int(arguments[1]) if len(arguments) == 2 else 8765)
    if arguments == ["self-test"]:
        return self_test()
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
