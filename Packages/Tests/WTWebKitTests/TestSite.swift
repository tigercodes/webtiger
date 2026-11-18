// A web site on this machine's loopback address, for the adapter suite: pages that set and
// read a cookie, redirect, and change their own address. (Tools/testsite will serve the same
// behaviours to every shell's suite; this one lives in the test process so the suite needs
// nothing running beside it.)
import Foundation
import Network

final class TestSite: @unchecked Sendable {
    /// The port the site listens on. 0 when it could not start.
    let port: UInt16
    private let listener: NWListener?
    private let queue = DispatchQueue(label: "webtiger.testsite")

    init() {
        let parameters = NWParameters.tcp
        parameters.requiredLocalEndpoint = .hostPort(host: .ipv4(.loopback), port: .any)
        parameters.allowLocalEndpointReuse = true
        guard let listener = try? NWListener(using: parameters) else {
            listener = nil
            port = 0
            return
        }
        self.listener = listener
        let ready = DispatchSemaphore(value: 0)
        listener.stateUpdateHandler = { state in
            switch state {
            case .ready, .failed, .cancelled: ready.signal()
            default: break
            }
        }
        listener.newConnectionHandler = { [queue] connection in
            connection.start(queue: queue)
            TestSite.serve(connection, received: Data())
        }
        listener.start(queue: queue)
        ready.wait()
        port = listener.port?.rawValue ?? 0
    }

    deinit {
        listener?.cancel()
    }

    /// The address of a path on the site.
    func url(_ path: String) -> String {
        "http://127.0.0.1:\(port)\(path)"
    }

    private static func serve(_ connection: NWConnection, received: Data) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 16384) { data, _, finished, error in
            var received = received
            if let data {
                received.append(data)
            }
            guard let end = received.range(of: Data("\r\n\r\n".utf8)) else {
                if finished || error != nil {
                    connection.cancel()
                } else {
                    serve(connection, received: received)
                }
                return
            }
            let head = String(bytes: received[..<end.lowerBound], encoding: .utf8) ?? ""
            connection.send(content: respond(to: head), completion: .contentProcessed { _ in connection.cancel() })
        }
    }

    private static func respond(to head: String) -> Data {
        let lines = head.components(separatedBy: "\r\n")
        let target = lines.first?.split(separator: " ").dropFirst().first.map(String.init) ?? "/"
        let path = target.split(separator: "?", maxSplits: 1).first.map(String.init) ?? "/"
        let query = target.split(separator: "?", maxSplits: 1).dropFirst().first.map(String.init) ?? ""
        let cookies = lines.first { $0.lowercased().hasPrefix("cookie:") }?.dropFirst("cookie:".count)
            .trimmingCharacters(in: .whitespaces) ?? ""
        let who = cookies.split(separator: ";").map { $0.trimmingCharacters(in: .whitespaces) }
            .first { $0.hasPrefix("who=") }?.dropFirst("who=".count)

        switch path {
        case "/":
            return page(title: "Home", body: "<a id=\"next\" href=\"/landed\">next</a>")
        case "/set":
            // ?v=<value>: remembers it in a cookie.
            let value = query.split(separator: "&").first { $0.hasPrefix("v=") }?.dropFirst(2) ?? "someone"
            return page(title: "Set", headers: ["Set-Cookie: who=\(value); Path=/; Max-Age=3600"])
        case "/whoami":
            return page(title: who.map { "who=\($0)" } ?? "who=none")
        case "/redirect":
            return response(status: "302 Found", headers: ["Location: /landed"], body: "")
        case "/landed":
            return page(title: "Landed")
        case "/push":
            return page(title: "Push", body: "<script>history.pushState({}, '', '/pushed')</script>")
        case "/worker.js":
            // A service worker that does nothing but exist.
            return response(
                status: "200 OK", headers: [], body: "self.addEventListener('install', () => self.skipWaiting());",
                type: "text/javascript"
            )
        case "/tall":
            // Taller than any window, with a field to type in: for scroll and form restoration.
            return page(title: "Tall", body: "<input id=\"note\"><div style=\"height: 6000px\"></div><p id=\"end\">end</p>")
        case "/storage":
            // Remembers ?v= in localStorage, or says what was remembered.
            let script = """
            <script>
            const v = new URLSearchParams(location.search).get('v');
            if (v) localStorage.setItem('who', v);
            document.title = 'stored=' + (localStorage.getItem('who') || 'none');
            </script>
            """
            return page(title: "Storage", body: script)
        default:
            return response(status: "404 Not Found", headers: [], body: "<title>Not found</title>")
        }
    }

    private static func page(title: String, headers: [String] = [], body: String = "") -> Data {
        response(status: "200 OK", headers: headers, body: "<!doctype html><title>\(title)</title><h1>\(title)</h1>\(body)")
    }

    private static func response(status: String, headers: [String], body: String, type: String = "text/html") -> Data {
        let bytes = Data(body.utf8)
        var head = [
            "HTTP/1.1 \(status)",
            "Content-Type: \(type); charset=utf-8",
            "Content-Length: \(bytes.count)",
            "Cache-Control: no-store",
            "Connection: close",
        ]
        head.append(contentsOf: headers)
        return Data((head.joined(separator: "\r\n") + "\r\n\r\n").utf8) + bytes
    }
}
