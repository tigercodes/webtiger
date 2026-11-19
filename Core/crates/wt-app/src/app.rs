//! The app: what a shell's requests and reports do to the models.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use wt_core::{Address, Input, TabId, WebViewId, WindowId, classify, search_address};
use wt_proto::v1::{
    Ack, ActivateTab, CloseTab, CloseWindow, EngineEvent, EngineKind, LoginSet, NavigationCommand,
    OpenTab, SubmitAddress, TabCommand, TabOpened, WebViewCommand, WebViewCreate, WebViewDestroy,
    WebViewLoad, WindowOpened, WindowSnapshot, host_request, request, response,
};
use wt_store::Space;

use crate::error::AppError;
use crate::host::{Host, HostFuture};
use crate::model::{LoadTarget, State};

/// The search a query loads until the first-run flow lets the user choose a
/// provider (WT-D17, W1). Chosen for W0 because it needs no account and no
/// agreement; it is a placeholder, not a default the product has decided on.
pub const DEFAULT_SEARCH_TEMPLATE: &str = "https://duckduckgo.com/?q={query}";

/// A request's answer, when it is ready. Whatever the request changes in the
/// models it changes before this is returned, so requests take effect in the
/// order they arrive; the future is only the wait for the shell.
pub type AppFuture = Pin<Box<dyn Future<Output = Result<response::Kind, AppError>>>>;

/// What the app is started with.
#[derive(Clone, Debug)]
pub struct AppOptions {
    /// The engine new tabs use: the shell's first.
    pub engine: EngineKind,
    /// A search provider's address with `{query}` for the words.
    pub search_template: String,
    /// Fake mode: what every new window's first tab opens.
    pub fake_start: Option<String>,
}

/// The models and what changes them.
#[derive(Clone)]
pub struct App {
    state: Rc<RefCell<State>>,
    host: Rc<dyn Host>,
    search_template: Rc<str>,
    fake_start: Option<Rc<str>>,
}

impl App {
    /// An app with no windows, whose tabs open in `space`.
    #[must_use]
    pub fn new(options: AppOptions, space: Space, host: Rc<dyn Host>) -> App {
        App {
            state: Rc::new(RefCell::new(State::new(space, options.engine))),
            host,
            search_template: options.search_template.into(),
            fake_start: options.fake_start.map(Into::into),
        }
    }

    /// Handles one of the app's requests: a window's (`OpenWindow`,
    /// `CloseWindow`) or a tab's (`OpenTab`, `CloseTab`, `ActivateTab`,
    /// `SubmitAddress`, `TabCommand`). The dispatcher answers the rest.
    pub fn request(&self, kind: request::Kind) -> AppFuture {
        match kind {
            request::Kind::OpenWindow(_) => self.open_window(),
            request::Kind::CloseWindow(close) => self.close_window(&close),
            request::Kind::OpenTab(open) => self.open_tab(&open),
            request::Kind::CloseTab(close) => self.close_tab(&close),
            request::Kind::ActivateTab(activate) => self.activate_tab(&activate),
            request::Kind::SubmitAddress(submit) => self.submit_address(&submit),
            request::Kind::TabCommand(command) => self.tab_command(&command),
            request::Kind::Ping(_)
            | request::Kind::Subscribe(_)
            | request::Kind::Unsubscribe(_)
            | request::Kind::EngineEvent(_) => refuse(AppError::new("request.unsupported")),
        }
    }

    /// Takes a shell's report about a web view.
    pub fn notify(&self, event: EngineEvent) {
        if let Some(kind) = event.kind {
            self.state
                .borrow_mut()
                .apply(WebViewId(event.web_view_id), kind);
        }
    }

    /// A window as its subscribers see it, or `None` when there is none.
    #[must_use]
    pub fn window_snapshot(&self, window: WindowId) -> Option<WindowSnapshot> {
        self.state.borrow().window_snapshot(window)
    }

    /// The windows that may have changed since this was last called.
    #[must_use]
    pub fn take_dirty(&self) -> BTreeSet<WindowId> {
        self.state.borrow_mut().take_dirty()
    }

    /// The windows closed since this was last called.
    #[must_use]
    pub fn take_closed(&self) -> Vec<WindowId> {
        self.state.borrow_mut().take_closed()
    }

    fn open_window(&self) -> AppFuture {
        let (window, tab) = self.state.borrow_mut().open_window();
        let opened = response::Kind::WindowOpened(WindowOpened {
            window_id: window.get(),
            tab_id: tab.get(),
        });
        let start = self
            .fake_start
            .as_deref()
            .and_then(|text| self.address_for(text));
        let Some(loading) = start.and_then(|address| self.begin_load(tab, address)) else {
            return Box::pin(std::future::ready(Ok(opened)));
        };
        Box::pin(async move {
            // The window opened whatever became of its first page: a failed
            // load shows in the tab.
            let _ = loading.await;
            Ok(opened)
        })
    }

    fn close_window(&self, close: &CloseWindow) -> AppFuture {
        let closed = self
            .state
            .borrow_mut()
            .close_window(WindowId(close.window_id));
        let Some(web_views) = closed else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        let destroying: Vec<HostFuture> = web_views
            .into_iter()
            .map(|web_view| {
                self.host
                    .request(host_request::Kind::WebViewDestroy(WebViewDestroy {
                        web_view_id: web_view.get(),
                    }))
            })
            .collect();
        Box::pin(async move {
            // The window is closed in the models already. A shell that fails
            // to destroy a web view keeps a leak, not a window.
            for destroyed in destroying {
                let _ = destroyed.await;
            }
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    fn open_tab(&self, open: &OpenTab) -> AppFuture {
        let after = (open.after_tab_id != 0).then_some(TabId(open.after_tab_id));
        let opened =
            self.state
                .borrow_mut()
                .open_tab(WindowId(open.window_id), after, open.background);
        let Some(tab) = opened else {
            return refuse(AppError::new("window.unknown").field("window_id"));
        };
        let opened = response::Kind::TabOpened(TabOpened { tab_id: tab.get() });
        let page = self
            .address_for(&open.text)
            .and_then(|address| self.begin_load(tab, address));
        let Some(loading) = page else {
            return Box::pin(std::future::ready(Ok(opened)));
        };
        Box::pin(async move {
            // The tab opened whatever became of its page: a failed load
            // shows in the tab.
            let _ = loading.await;
            Ok(opened)
        })
    }

    fn close_tab(&self, close: &CloseTab) -> AppFuture {
        let closed = self.state.borrow_mut().close_tab(TabId(close.tab_id));
        let Some(web_view) = closed else {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        };
        let destroying = web_view.map(|web_view| {
            self.host
                .request(host_request::Kind::WebViewDestroy(WebViewDestroy {
                    web_view_id: web_view.get(),
                }))
        });
        Box::pin(async move {
            // The tab is closed in the models already. A shell that fails to
            // destroy its web view keeps a leak, not a tab.
            if let Some(destroyed) = destroying {
                let _ = destroyed.await;
            }
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    fn activate_tab(&self, activate: &ActivateTab) -> AppFuture {
        if self.state.borrow_mut().activate_tab(TabId(activate.tab_id)) {
            Box::pin(std::future::ready(Ok(response::Kind::Ack(Ack {}))))
        } else {
            refuse(AppError::new("tab.unknown").field("tab_id"))
        }
    }

    fn submit_address(&self, submit: &SubmitAddress) -> AppFuture {
        let tab = TabId(submit.tab_id);
        if self.state.borrow().web_view_of(tab) == Err(false) {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        }
        let Some(address) = self.address_for(&submit.text) else {
            return refuse(AppError::new("address.empty").field("text"));
        };
        let Some(loading) = self.begin_load(tab, address) else {
            return refuse(AppError::new("tab.unknown").field("tab_id"));
        };
        Box::pin(async move {
            loading.await?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    fn tab_command(&self, command: &TabCommand) -> AppFuture {
        let web_view = match self.state.borrow().web_view_of(TabId(command.tab_id)) {
            Ok(web_view) => web_view,
            Err(false) => return refuse(AppError::new("tab.unknown").field("tab_id")),
            Err(true) => return refuse(AppError::new("tab.no_page").field("tab_id")),
        };
        if !matches!(
            command.command(),
            NavigationCommand::Back
                | NavigationCommand::Forward
                | NavigationCommand::Reload
                | NavigationCommand::Stop
        ) {
            return refuse(AppError::new("tab.bad_command").field("command"));
        }
        let commanded = self
            .host
            .request(host_request::Kind::WebViewCommand(WebViewCommand {
                web_view_id: web_view.get(),
                command: command.command,
            }));
        Box::pin(async move {
            commanded.await?;
            Ok(response::Kind::Ack(Ack {}))
        })
    }

    /// Where typed text goes: the address it is, or a search for it. `None`
    /// when it is empty.
    fn address_for(&self, text: &str) -> Option<Address> {
        match classify(text)? {
            Input::Address(address) => Some(address),
            Input::Search(words) => search_address(&self.search_template, &words),
        }
    }

    /// Sends a tab to an address: the models change and the shell is asked
    /// before this returns, the web view's creation and its load one after
    /// the other, so two loads asked in a row reach the shell in that order.
    /// The future is the wait for the shell's answers. `None` when there is
    /// no such tab.
    fn begin_load(
        &self,
        tab: TabId,
        address: Address,
    ) -> Option<impl Future<Output = Result<(), AppError>> + use<>> {
        let url = address.as_str().to_owned();
        let target = self.state.borrow_mut().begin_user_load(tab, address)?;
        let LoadTarget {
            web_view,
            create,
            engine,
            login_set,
        } = target;
        let creating = create.then(|| {
            self.host
                .request(host_request::Kind::WebViewCreate(WebViewCreate {
                    web_view_id: web_view.get(),
                    engine: engine as i32,
                    login_set: Some(LoginSet {
                        id: login_set.to_string(),
                        ephemeral: false,
                    }),
                }))
        });
        let loading = self
            .host
            .request(host_request::Kind::WebViewLoad(WebViewLoad {
                web_view_id: web_view.get(),
                url,
            }));
        let state = Rc::clone(&self.state);
        Some(async move {
            if let Some(creating) = creating
                && let Err(error) = creating.await
            {
                state.borrow_mut().web_view_failed(web_view, error.code());
                // The load was asked of a web view that never came to be;
                // its answer, whatever it is, has nothing to add.
                let _ = loading.await;
                return Err(error.into());
            }
            if let Err(error) = loading.await {
                state.borrow_mut().load_failed(web_view, error.code());
                return Err(error.into());
            }
            Ok(())
        })
    }
}

fn refuse(error: AppError) -> AppFuture {
    Box::pin(std::future::ready(Err(error)))
}
