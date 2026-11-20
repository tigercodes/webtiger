//! Going on past a certificate that is not trusted (026): the person's
//! choice, after the shell's warning, for a tab whose failed load may be
//! gone on with.

use std::rc::Rc;

use wt_core::TabId;
use wt_proto::v1::{Ack, ProceedPastCertificate, WebViewTrustCertificate, host_request, response};

use crate::app::{App, AppFuture, refuse};
use crate::error::AppError;

impl App {
    /// Makes the exception, has the shell let the certificate through, and
    /// loads the address that failed again: in that order, at once (026 §3).
    pub(crate) fn proceed_past_certificate(&self, proceed: &ProceedPastCertificate) -> AppFuture {
        let tab = TabId(proceed.tab_id);
        let not_passable = || AppError::new("certificate.not_passable").field("tab_id");
        let (web_view, address, login_set) = match self.state.borrow().passable(tab) {
            Ok(found) => found,
            Err(false) => return refuse(AppError::new("tab.unknown").field("tab_id")),
            Err(true) => return refuse(not_passable()),
        };
        let Some(origin) = address.origin() else {
            return refuse(not_passable());
        };
        // From now every page of that origin, in that login set, is marked
        // not secure: before the page that is about to load can be shown.
        self.state.borrow_mut().except(login_set, origin.clone());
        let trusting = self
            .host
            .request(host_request::Kind::WebViewTrustCertificate(
                WebViewTrustCertificate {
                    web_view_id: web_view.get(),
                },
            ));
        let loading = self.begin_load(tab, address);
        let state = Rc::clone(&self.state);
        Box::pin(async move {
            let trusted = trusting.await;
            if trusted.is_err() {
                // The adapter held nothing to let through: there is no
                // exception, and the load fails as it did.
                state.borrow_mut().unexcept(login_set, &origin);
            }
            // The tab shows whatever becomes of the load.
            if let Some(loading) = loading {
                let _ = loading.await;
            }
            trusted?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }
}
