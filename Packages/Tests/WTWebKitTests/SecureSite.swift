// A site over https on this machine's loopback address, whose certificate signs itself: one no
// device trusts (021, 026). It lives in the test's process, like the test site, and serves one
// page. Its identity is made in memory from TestIdentities.swift: no keychain is touched.
import Foundation
import Network
import Security

final class SecureSite: @unchecked Sendable {
    /// The port the site listens on. 0 when it could not start.
    let port: UInt16
    private let listener: NWListener?
    private let queue = DispatchQueue(label: "webtiger.securesite")
    private let counted = Counted()

    private final class Counted: @unchecked Sendable {
        private let lock = NSLock()
        private var pages = 0
        private var connections: [NWConnection] = []

        func add() {
            lock.withLock { pages += 1 }
        }

        var count: Int {
            lock.withLock { pages }
        }

        func opened(_ connection: NWConnection) {
            lock.withLock { connections.append(connection) }
        }

        /// Ends every connection the site has: a browser keeps one open for the next request,
        /// and a site that has stopped must not go on answering on it.
        func closeAll() {
            let open = lock.withLock {
                defer { connections = [] }
                return connections
            }
            open.forEach { $0.cancel() }
        }
    }

    /// How many requests for the page were answered: none, when the browser would not trust
    /// the certificate. What a browser asks for by itself (the site's icon) is not counted.
    var served: Int {
        counted.count
    }

    /// The site's page: its address, and the title it has.
    var address: String {
        "https://127.0.0.1:\(port)/"
    }

    static let title = "Trusted after all"

    /// A site with `identity`, listening on `port`: any port, or one another such site has just
    /// left, for a host whose certificate changes.
    init(_ identity: TestIdentity = .first, port: UInt16 = 0) {
        guard let made = identity.make(), let local = sec_identity_create(made) else {
            listener = nil
            self.port = 0
            return
        }
        let tls = NWProtocolTLS.Options()
        sec_protocol_options_set_local_identity(tls.securityProtocolOptions, local)
        let parameters = NWParameters(tls: tls)
        parameters.requiredLocalEndpoint = .hostPort(
            host: .ipv4(.loopback),
            port: port == 0 ? .any : NWEndpoint.Port(rawValue: port) ?? .any
        )
        parameters.allowLocalEndpointReuse = true
        guard let listener = try? NWListener(using: parameters) else {
            listener = nil
            self.port = 0
            return
        }
        self.listener = listener
        let counted = counted
        let queue = queue
        listener.newConnectionHandler = { connection in
            counted.opened(connection)
            connection.start(queue: queue)
            Self.answer(connection, counted: counted)
        }
        let ready = DispatchSemaphore(value: 0)
        listener.stateUpdateHandler = { state in
            switch state {
            case .ready, .failed, .cancelled: ready.signal()
            default: break
            }
        }
        listener.start(queue: queue)
        _ = ready.wait(timeout: .now() + 5)
        self.port = listener.port?.rawValue ?? 0
    }

    /// Stops listening and ends every connection: the port is free for another site, and
    /// nothing of this one answers any more.
    func stop() {
        listener?.cancel()
        counted.closeAll()
    }

    deinit {
        listener?.cancel()
        counted.closeAll()
    }

    /// Reads a request and answers it: the page for "/", nothing found for anything else. A
    /// browser that does not trust the certificate never gets this far: its connection ends in
    /// the handshake.
    private static func answer(_ connection: NWConnection, counted: Counted, received: Data = Data()) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 16384) { data, _, isComplete, error in
            var received = received
            received.append(data ?? Data())
            let text = String(bytes: received, encoding: .utf8) ?? ""
            guard text.contains("\r\n\r\n") else {
                if isComplete || error != nil {
                    connection.cancel()
                } else {
                    answer(connection, counted: counted, received: received)
                }
                return
            }
            let isPage = text.hasPrefix("GET / ")
            if isPage {
                counted.add()
            }
            let body = isPage ? "<!doctype html><title>\(title)</title><p>\(title)</p>" : ""
            // Never kept: a page that an engine has kept is shown again without the site being
            // asked, and so without its certificate being looked at.
            let head = "HTTP/1.1 \(isPage ? "200 OK" : "404 Not Found")\r\nContent-Type: text/html\r\nCache-Control: no-store\r\n"
                + "Content-Length: \(body.utf8.count)\r\nConnection: close\r\n\r\n"
            connection.send(content: Data((head + body).utf8), completion: .contentProcessed { _ in connection.cancel() })
        }
    }
}
