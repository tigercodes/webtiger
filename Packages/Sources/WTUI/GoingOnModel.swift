// Going on past a certificate that is not trusted (026 §6), the model's part: whether the core
// says the load in front may be gone on with, whether the warning is open, and the request that
// goes on. The shell decides nothing about the certificate: it warns, and asks the core.
import WTContract

public extension BrowserModel {
    /// The load in front failed for its site's certificate, and the core says it may be gone
    /// on with.
    var canGoOnPastCertificate: Bool {
        hasFailed && tab.certificatePassable
    }

    /// Shows the warning. Nothing goes on by this.
    func warnOfCertificate() {
        if canGoOnPastCertificate {
            isWarnedOfCertificate = true
        }
    }

    /// The person goes on, having been warned: the core is asked, and what comes of it is the
    /// core's to say. Without the warning on the screen, nothing is asked.
    func goOnPastCertificate() {
        guard let frontTabID, canGoOnPastCertificate, isWarnedOfCertificate else { return }
        isWarnedOfCertificate = false
        var proceed = WTProceedPastCertificate()
        proceed.tabID = frontTabID
        send(.proceedPastCertificate(proceed))
    }
}

extension BrowserModel {
    /// What the failed page offers of going on: nothing, unless the core says the load may be.
    var goingOn: GoingOn? {
        guard canGoOnPastCertificate else { return nil }
        return GoingOn(
            host: tab.certificateHost,
            isWarned: isWarnedOfCertificate,
            advanced: { [weak self] in self?.warnOfCertificate() },
            goOn: { [weak self] in self?.goOnPastCertificate() }
        )
    }

    /// The warning is one failure's: another tab, another load or another site starts again
    /// from "Advanced…". Called with each snapshot, before it is taken. (The window's first
    /// snapshot is no other failure than the one a picture was made of.)
    func keepWarning(for next: WTTabView, switched: Bool) {
        let first = tab.tabID == 0
        let goesOn = next.loadState == .failed && next.certificatePassable
        let sameFailure = goesOn && (first || (!switched && next.certificateHost == tab.certificateHost))
        if !sameFailure {
            isWarnedOfCertificate = false
        }
    }
}
