// The test site on this machine's loopback address, for the adapter suite and the apps' UI
// tests. Its pages are defined once for every shell, in Fixtures/testsite/site.json, and
// written into TestSitePages.generated.swift by Tools/testsite/site.py; this file only serves
// them. It lives in the test's process, so a suite needs nothing running beside it.
import Foundation
import Network

/// A piece of a header or a body: words as they are, or a value the request brought.
enum TestSitePiece: Sendable {
    case text(String)
    /// The request's query value of this name.
    case query(String, or: String)
    /// The request's cookie of this name.
    case cookie(String, or: String)
}

/// One page of the site: what a request for its path is answered with.
struct TestSitePage: Sendable {
    let path: String
    let status: String
    let type: String
    let headers: [[TestSitePiece]]
    let body: [TestSitePiece]
    let delayMilliseconds: Int
}

final class TestSite: @unchecked Sendable {
    /// The port the site listens on. 0 when it could not start.
    let port: UInt16
    private let listener: NWListener?
    private let queue = DispatchQueue(label: "webtiger.testsite")
    private let asked = Asked()

    /// The paths asked for, kept apart from the site so the listener can hold them before the
    /// site itself is whole.
    private final class Asked: @unchecked Sendable {
        private let lock = NSLock()
        private var paths: [String] = []

        func add(_ path: String) {
            lock.withLock { paths.append(path) }
        }

        var all: [String] {
            lock.withLock { paths }
        }
    }

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
        listener.newConnectionHandler = { [queue, asked] connection in
            connection.start(queue: queue)
            TestSite.serve(connection, received: Data(), on: queue, asked: asked)
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

    /// Every path asked for, in order, without its query.
    var requests: [String] {
        asked.all
    }

    private static func serve(_ connection: NWConnection, received: Data, on queue: DispatchQueue, asked: Asked) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 16384) { data, _, finished, error in
            var received = received
            if let data {
                received.append(data)
            }
            guard let end = received.range(of: Data("\r\n\r\n".utf8)) else {
                if finished || error != nil {
                    connection.cancel()
                } else {
                    serve(connection, received: received, on: queue, asked: asked)
                }
                return
            }
            let head = String(bytes: received[..<end.lowerBound], encoding: .utf8) ?? ""
            let (answer, delay) = respond(to: head, asked: asked)
            queue.asyncAfter(deadline: .now() + .milliseconds(delay)) {
                connection.send(content: answer, completion: .contentProcessed { _ in connection.cancel() })
            }
        }
    }

    /// The whole answer to a request's head, and how long to wait before giving it.
    private static func respond(to head: String, asked: Asked) -> (Data, delayMilliseconds: Int) {
        let lines = head.components(separatedBy: "\r\n")
        let target = lines.first?.split(separator: " ").dropFirst().first.map(String.init) ?? "/"
        let path = target.split(separator: "?", maxSplits: 1, omittingEmptySubsequences: false).first.map(String.init) ?? "/"
        let query = target.split(separator: "?", maxSplits: 1).dropFirst().first.map(String.init) ?? ""
        let cookies = lines.dropFirst().first { $0.lowercased().hasPrefix("cookie:") }?.dropFirst("cookie:".count) ?? ""
        asked.add(path)

        let request = (query: pairs(query, separator: "&"), cookies: pairs(String(cookies), separator: ";"))
        let page = TestSitePages.all.first { $0.path == path } ?? TestSitePages.notFound
        let body = Data(filled(page.body, from: request).utf8)
        let head = [
            "HTTP/1.1 \(page.status)",
            "Content-Type: \(page.type); charset=utf-8",
            "Content-Length: \(body.count)",
            "Connection: close",
        ] + page.headers.map { filled($0, from: request) }
        return (Data((head.joined(separator: "\r\n") + "\r\n\r\n").utf8) + body, page.delayMilliseconds)
    }

    /// `name=value` pairs. The first of a name is the one kept.
    private static func pairs(_ text: String, separator: Character) -> [String: String] {
        var found: [String: String] = [:]
        for pair in text.split(separator: separator) {
            let parts = pair.trimmingCharacters(in: .whitespaces).split(separator: "=", maxSplits: 1, omittingEmptySubsequences: false)
            if parts.count == 2, found[String(parts[0])] == nil {
                found[String(parts[0])] = String(parts[1])
            }
        }
        return found
    }

    private static func filled(_ pieces: [TestSitePiece], from request: (query: [String: String], cookies: [String: String])) -> String {
        pieces.map { piece in
            switch piece {
            case let .text(text): text
            case let .query(name, fallback): request.query[name] ?? fallback
            case let .cookie(name, fallback): request.cookies[name] ?? fallback
            }
        }.joined()
    }
}
