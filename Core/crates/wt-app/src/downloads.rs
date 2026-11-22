//! Downloads (013): the core decides whether one starts, names its file,
//! and keeps its record. The engine, or the system for it, moves the bytes.

use std::time::Duration;

use tokio::time::Instant;
use wt_core::{Address, LoginSetId, WebViewId, WindowId, download_file_name};
use wt_proto::v1::{
    Ack, CancelDownload, DownloadCancel, DownloadEntry, DownloadProgress, DownloadRequested,
    DownloadStart, DownloadState, DownloadsPage, ForgetDownloads, OpenDownload, OpenFile,
    PermissionKind, QueryDownloads, engine_event, forget_downloads, host_request, response,
};
use wt_store::{COMING, Download, FINISHED};

use crate::app::{App, AppFuture, refuse};
use crate::error::{AppError, is_code};
use crate::model::{AskedFor, Asker};

/// A web view starts no more than one download in this long (013 §2): until
/// a site's first download is asked about, a page cannot fill a folder
/// faster than a person can see it doing so.
pub const DOWNLOAD_INTERVAL: Duration = Duration::from_secs(1);

/// How many downloads a page of the list holds when the shell does not say.
pub const DOWNLOADS_PAGE: u32 = 100;

/// A download ended because somebody stopped it.
const CANCELLED: &str = "download.cancelled";
/// A download ended, and the shell did not say why in a code.
const FAILED: &str = "download.failed";

/// To download, as something a site is allowed (015).
const DOWNLOADS: PermissionKind = PermissionKind::Downloads;

/// A download a page asked for, as it would be started: kept while the
/// person is asked whether its site may download (015).
#[derive(Clone, Debug)]
pub(crate) struct Wanted {
    web_view: WebViewId,
    login_set: LoginSetId,
    url: String,
    /// The name its file is saved under: the core's (013 §4).
    name: String,
    mime_type: String,
    /// 0 when the site did not say.
    expected: u64,
}

/// A download that is coming. The shell's name for it is what it is kept
/// by; the store's is what a list shows.
#[derive(Clone, Debug)]
pub(crate) struct Downloading {
    record: i64,
    login_set: LoginSetId,
    received: u64,
    /// 0 while the size is not known.
    expected: u64,
}

impl App {
    /// A shell's word about a download. False when the event was not one.
    pub(crate) fn download_event(&self, web_view: WebViewId, kind: &engine_event::Kind) -> bool {
        match kind {
            engine_event::Kind::DownloadRequested(asked) => {
                self.download_requested(web_view, asked);
            }
            engine_event::Kind::DownloadProgress(progress) => self.download_progress(progress),
            engine_event::Kind::DownloadFinished(finished) => {
                // A name of the shell's is the name it used; one it left out
                // is the name it was given.
                let name = Some(finished.file_name.as_str()).filter(|name| !name.is_empty());
                self.download_ended(finished.download_id, FINISHED, name);
            }
            engine_event::Kind::DownloadFailed(failed) => {
                // The core speaks codes: what a shell says that is not one
                // is not passed on.
                let code = if is_code(&failed.code) {
                    failed.code.as_str()
                } else {
                    FAILED
                };
                self.download_ended(failed.download_id, code, None);
            }
            _ => return false,
        }
        true
    }

    /// A page asked for a download: it starts, under a name the core makes;
    /// or it waits while the person is asked whether the site may download
    /// (015); or the shell is told to drop it.
    fn download_requested(&self, web_view: WebViewId, asked: &DownloadRequested) {
        let shell_id = asked.download_id;
        // Said twice: the download it names is the one that is coming, or
        // the one that waits.
        if self.downloads.borrow().contains_key(&shell_id)
            || self.waiting.borrow().contains_key(&shell_id)
        {
            return;
        }
        // A file the person chose in a menu (022 §3) is no page's: it is
        // not held to a page's one download a second, and is not counted
        // against it.
        let login_set = if asked.user_asked {
            self.state.borrow().login_set_of(web_view)
        } else {
            self.may_download(web_view)
        };
        let Some(login_set) = login_set else {
            self.drop_download(shell_id);
            return;
        };
        let wanted = Wanted {
            web_view,
            login_set,
            url: asked.url.clone(),
            name: download_file_name(&asked.suggested_filename, &asked.url, &asked.mime_type),
            mime_type: asked.mime_type.clone(),
            expected: asked.expected_bytes,
        };
        // And nobody is asked whether the site may download: no page asked.
        // Nothing is written about the site, and what the person typed is
        // still good for its own download (015 §3).
        if asked.user_asked {
            self.start_download(shell_id, wanted);
            return;
        }
        match self.may_start(web_view, shell_id, &wanted) {
            Some(true) => self.start_download(shell_id, wanted),
            Some(false) => self.drop_download(shell_id),
            // The person is asked: it waits for their answer.
            None => {
                self.waiting.borrow_mut().insert(shell_id, wanted);
            }
        }
    }

    /// Whether a download starts with nobody asked (015 §3): yes, no, or
    /// `None` when the person has been asked.
    fn may_start(&self, web_view: WebViewId, shell_id: u64, wanted: &Wanted) -> Option<bool> {
        let Some(who) = self.state.borrow_mut().downloader(web_view, &wanted.url) else {
            return Some(false);
        };
        // An address the person sent the tab to themselves: whatever it
        // turned out to be, they asked for it.
        if who.own {
            return Some(true);
        }
        let allowed = who
            .origin
            .as_ref()
            .is_some_and(|origin| self.allowed(who.login_set, origin).contains(&DOWNLOADS));
        if allowed {
            return Some(true);
        }
        // The person said no to this page already.
        if self.state.borrow().refused_here(web_view, &[DOWNLOADS]) {
            return Some(false);
        }
        let asker = Asker {
            login_set: who.login_set,
            origin: who.origin.unwrap_or_default(),
            site: who.site,
        };
        let asking = self.state.borrow_mut().ask(
            web_view,
            shell_id,
            asker,
            vec![DOWNLOADS],
            AskedFor::Download {
                file_name: wanted.name.clone(),
            },
        );
        // A tab that asks as much as it may asks no more.
        (!asking).then_some(false)
    }

    /// The person let a download that waited start.
    pub(crate) fn start_waiting(&self, shell_id: u64) {
        if let Some(wanted) = self.waiting.borrow_mut().remove(&shell_id) {
            self.start_download(shell_id, wanted);
        }
    }

    /// A download that waited is not to start: the shell is told to drop it.
    pub(crate) fn drop_waiting(&self, shell_id: u64) {
        if self.waiting.borrow_mut().remove(&shell_id).is_some() {
            self.drop_download(shell_id);
        }
    }

    /// The downloads that waited in a tab that is gone go with it: nobody is
    /// left to be asked. (Those that waited on a page that left were dropped
    /// with its prompts.)
    pub(crate) fn drop_orphaned_downloads(&self) {
        let orphaned: Vec<u64> = {
            let state = self.state.borrow();
            self.waiting
                .borrow()
                .iter()
                .filter(|(_, wanted)| state.login_set_of(wanted.web_view).is_none())
                .map(|(shell_id, _)| *shell_id)
                .collect()
        };
        for shell_id in orphaned {
            self.drop_waiting(shell_id);
        }
    }

    /// Starts a download: its record is written, and the shell is asked to
    /// fetch it under the core's name.
    fn start_download(&self, shell_id: u64, wanted: Wanted) {
        let Wanted {
            login_set,
            url,
            name,
            mime_type,
            expected,
            ..
        } = wanted;
        let total = (expected > 0).then_some(expected);
        let written = self.store_of(login_set).borrow_mut().add_download(
            login_set,
            &url,
            &name,
            &mime_type,
            total,
            self.clock.now(),
        );
        let Ok(record) = written else {
            // Nothing is saved that the list cannot say.
            self.drop_download(shell_id);
            return;
        };
        self.downloads.borrow_mut().insert(
            shell_id,
            Downloading {
                record,
                login_set,
                received: 0,
                expected,
            },
        );
        self.publish_downloads(login_set);
        let starting = self
            .host
            .request(host_request::Kind::DownloadStart(DownloadStart {
                download_id: shell_id,
                file_name: name,
            }));
        // Asked already. A shell that will not start it has no download:
        // it ended, and the list says so.
        let app = self.clone();
        tokio::task::spawn_local(async move {
            if starting.await.is_err() {
                app.download_ended(shell_id, FAILED, None);
            }
        });
    }

    /// Whether a web view's page may start a download now: it must be a
    /// tab's, and not have started one within [`DOWNLOAD_INTERVAL`]. The
    /// login set the download is then of.
    fn may_download(&self, web_view: WebViewId) -> Option<LoginSetId> {
        let login_set = self.state.borrow().login_set_of(web_view)?;
        let now = Instant::now();
        let mut started = self.downloaded.borrow_mut();
        started.retain(|_, at| now.duration_since(*at) < DOWNLOAD_INTERVAL);
        if started.contains_key(&web_view) {
            return None;
        }
        started.insert(web_view, now);
        Some(login_set)
    }

    /// How far a download has got: kept for the list, and written nowhere.
    fn download_progress(&self, progress: &DownloadProgress) {
        if let Some(coming) = self.downloads.borrow_mut().get_mut(&progress.download_id) {
            coming.received = progress.received_bytes;
            if progress.expected_bytes > 0 {
                coming.expected = progress.expected_bytes;
            }
        }
    }

    /// A download ended: as finished, under the name the shell used, or
    /// with a code. One the core does not have as coming (it was stopped,
    /// or never started) has ended already.
    fn download_ended(&self, shell_id: u64, state: &str, file_name: Option<&str>) {
        let Some(ended) = self.downloads.borrow_mut().remove(&shell_id) else {
            return;
        };
        // A store that refuses leaves the record coming until the next
        // start, when it is interrupted: there is nobody to tell.
        let _ = self.store_of(ended.login_set).borrow_mut().end_download(
            ended.record,
            state,
            file_name,
            self.clock.now(),
        );
        self.publish_downloads(ended.login_set);
    }

    /// Tells the shell to drop a download: one that is not to start, or one
    /// that is to stop.
    fn drop_download(&self, shell_id: u64) {
        let dropping = self
            .host
            .request(host_request::Kind::DownloadCancel(DownloadCancel {
                download_id: shell_id,
            }));
        // Asked already; this only waits for the answer, so that a shell
        // that never gives one is given up on like any other.
        tokio::task::spawn_local(async move {
            let _ = dropping.await;
        });
    }

    /// A login set's windows are told how many of its downloads are coming.
    fn publish_downloads(&self, login_set: LoginSetId) {
        let coming = self
            .downloads
            .borrow()
            .values()
            .filter(|download| download.login_set == login_set)
            .count();
        self.state
            .borrow_mut()
            .set_downloads_active(login_set, u32::try_from(coming).unwrap_or(u32::MAX));
    }

    /// The shell's name for a download that is coming, by its record's id.
    fn coming(&self, record: i64) -> Option<u64> {
        self.downloads
            .borrow()
            .iter()
            .find(|(_, download)| download.record == record)
            .map(|(shell_id, _)| *shell_id)
    }

    /// Stops a download that is coming, and has the shell drop it.
    fn stop(&self, shell_id: u64) {
        self.download_ended(shell_id, CANCELLED, None);
        self.drop_download(shell_id);
    }

    /// A page of a window's downloads: its Space's login set's, newest
    /// first, those that are coming with how far they have got.
    pub(crate) fn query_downloads(
        &self,
        query: &QueryDownloads,
    ) -> Result<response::Kind, AppError> {
        let login_set = self.downloads_of(query.window_id)?;
        let limit = if query.limit == 0 {
            DOWNLOADS_PAGE
        } else {
            query.limit
        };
        // An id beyond any the store could have given is "from the newest".
        let before = i64::try_from(query.before_id)
            .ok()
            .filter(|before| *before > 0);
        let page = self
            .store_of(login_set)
            .borrow()
            .downloads(login_set, before, limit)
            .map_err(|error| AppError::new(error.code()))?;
        let coming = self.downloads.borrow();
        let downloads = page
            .downloads
            .into_iter()
            .map(|download| {
                let progress = coming.values().find(|one| one.record == download.id);
                entry(download, progress)
            })
            .collect();
        Ok(response::Kind::DownloadsPage(DownloadsPage {
            downloads,
            more: page.more,
        }))
    }

    /// Stops a download that is coming.
    pub(crate) fn cancel_download(
        &self,
        cancel: &CancelDownload,
    ) -> Result<response::Kind, AppError> {
        let shell_id = i64::try_from(cancel.download_id)
            .ok()
            .and_then(|record| self.coming(record))
            .ok_or_else(|| AppError::new("download.unknown").field("download_id"))?;
        self.stop(shell_id);
        Ok(response::Kind::Ack(Ack {}))
    }

    /// A person pressed a download in the list to open its file (030). The
    /// core says whether it may be: one on the window's own list, that has
    /// finished. The shell's System service then opens it, and its answer
    /// is the answer.
    pub(crate) fn open_download(&self, open: &OpenDownload) -> AppFuture {
        let download = match self.finished_download(open) {
            Ok(download) => download,
            Err(refused) => return refuse(refused),
        };
        // The file and nothing else: not where it came from.
        let opening = self.host.request(host_request::Kind::OpenFile(OpenFile {
            file_name: download.file_name,
            mime_type: download.mime_type,
        }));
        Box::pin(async move {
            opening.await?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    /// The download a request to open one names, when it may be opened.
    fn finished_download(&self, open: &OpenDownload) -> Result<Download, AppError> {
        let login_set = self.downloads_of(open.window_id)?;
        let unknown = || AppError::new("download.unknown").field("download_id");
        // An id beyond any the store could have given is nobody's.
        let record = i64::try_from(open.download_id).map_err(|_| unknown())?;
        let download = self
            .store_of(login_set)
            .borrow()
            .download(login_set, record)
            .map_err(|error| AppError::new(error.code()))?
            .ok_or_else(unknown)?;
        // One that is coming has no whole file, and one that failed has none.
        if download.state != FINISHED {
            return Err(AppError::new("download.not_finished").field("download_id"));
        }
        Ok(download)
    }

    /// Takes downloads off a window's list: one, or all of its login set's.
    /// One that is coming is stopped first. No file is touched.
    pub(crate) fn forget_downloads(
        &self,
        forget: &ForgetDownloads,
    ) -> Result<response::Kind, AppError> {
        let login_set = self.downloads_of(forget.window_id)?;
        let forgotten = match forget.what {
            Some(forget_downloads::What::DownloadId(id)) => match i64::try_from(id) {
                Ok(record) => {
                    let mine = self
                        .downloads
                        .borrow()
                        .iter()
                        .find(|(_, one)| one.record == record && one.login_set == login_set)
                        .map(|(shell_id, _)| *shell_id);
                    if let Some(shell_id) = mine {
                        self.stop(shell_id);
                    }
                    self.store_of(login_set)
                        .borrow_mut()
                        .forget_download(login_set, record)
                        .map(|_| ())
                }
                // No download has such an id: there is nothing to forget.
                Err(_) => Ok(()),
            },
            Some(forget_downloads::What::Everything(true)) => {
                let mine: Vec<u64> = self
                    .downloads
                    .borrow()
                    .iter()
                    .filter(|(_, one)| one.login_set == login_set)
                    .map(|(shell_id, _)| *shell_id)
                    .collect();
                for shell_id in mine {
                    self.stop(shell_id);
                }
                self.store_of(login_set)
                    .borrow_mut()
                    .forget_all_downloads(login_set)
            }
            Some(forget_downloads::What::Everything(false)) | None => {
                return Err(AppError::new("download.nothing_named").field("what"));
            }
        };
        forgotten.map_err(|error| AppError::new(error.code()))?;
        Ok(response::Kind::Ack(Ack {}))
    }

    /// The login set whose downloads a window lists.
    fn downloads_of(&self, window_id: u64) -> Result<LoginSetId, AppError> {
        self.state
            .borrow()
            .history_of(WindowId(window_id))
            .ok_or_else(|| AppError::new("window.unknown").field("window_id"))
    }
}

/// A download as a list shows it: the record, and how far it has got when
/// it is coming.
fn entry(download: Download, progress: Option<&Downloading>) -> DownloadEntry {
    let (state, error_code) = match download.state.as_str() {
        COMING => (DownloadState::Coming, String::new()),
        FINISHED => (DownloadState::Finished, String::new()),
        code => (DownloadState::Failed, code.to_owned()),
    };
    // The size the shell has learned since, when the site did not say.
    let total = download
        .total_bytes
        .or_else(|| progress.map(|coming| coming.expected))
        .unwrap_or(0);
    DownloadEntry {
        id: u64::try_from(download.id).unwrap_or_default(),
        host: Address::parse(&download.url)
            .ok()
            .and_then(|address| address.host().map(str::to_owned))
            .unwrap_or_default(),
        url: download.url,
        file_name: download.file_name,
        mime_type: download.mime_type,
        total_bytes: total,
        received_bytes: progress.map_or(0, |coming| coming.received),
        state: state as i32,
        error_code,
        started_ms: download.started_at.unix_millis(),
    }
}
