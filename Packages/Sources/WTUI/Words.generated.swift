// Generated from Localization/strings.json by Core/tools/strings-gen (Tools/strings/generate.sh). Do not edit.

/// Every string of the catalogue, by name (040): `Text(Words.tabsNew)`. A string that is
/// deleted or renamed in the catalogue stops the build at every place that asked for it.
enum Words {
    /// Not secure
    ///
    /// Accessibility label of the warning shown for an http page.
    static let addressInsecure = Word(key: "address.insecure")

    /// Address and search
    ///
    /// Accessibility label of the address field.
    static let addressLabel = Word(key: "address.label")

    /// Search or enter address
    ///
    /// Placeholder in the address field.
    static let addressPlaceholder = Word(key: "address.placeholder")

    /// Secure connection
    ///
    /// Accessibility label of the padlock shown for an https page.
    static let addressSecure = Word(key: "address.secure")

    /// WebTiger
    ///
    /// The app's name. Not translated.
    static let appName = Word(key: "app.name")

    /// Forget All…
    ///
    /// Button in the Archive that asks whether to forget every tab in it.
    static let archiveClear = Word(key: "archive.clear")

    /// Cancel
    ///
    /// Choice: leave the Archive as it is.
    static let archiveClearCancel = Word(key: "archive.clear.cancel")

    /// Tabs that are forgotten can’t be brought back.
    ///
    /// Under the question of whether to forget every tab in the Archive.
    static let archiveClearDetail = Word(key: "archive.clear.detail")

    /// Forget All
    ///
    /// Choice: forget every tab in the Archive.
    static let archiveClearEverything = Word(key: "archive.clear.everything")

    /// Forget every tab in this Archive?
    ///
    /// Question asked by the Forget All button of the Archive.
    static let archiveClearTitle = Word(key: "archive.clear.title")

    /// Done
    ///
    /// Button that closes the Archive.
    static let archiveDone = Word(key: "archive.done")

    /// Tabs that are tidied away will be listed here.
    ///
    /// Shown in the Archive when it lists nothing.
    static let archiveEmpty = Word(key: "archive.empty")

    /// Nothing is tidied until you say so. To have this Space’s tabs tidied by themselves, edit the Space and choose after how long.
    ///
    /// Under the words of an Archive that lists nothing: how a Space comes to be tidied.
    static let archiveEmptyDetail = Word(key: "archive.empty.detail")

    /// Forget This Tab
    ///
    /// Accessibility label of the button beside a tab in the Archive that removes it for good.
    static let archiveForget = Word(key: "archive.forget")

    /// Show Older Tabs
    ///
    /// Button at the end of the Archive that shows the next, older page of tidied tabs.
    static let archiveOlder = Word(key: "archive.older")

    /// Tidied %@
    ///
    /// Under a tab in the Archive: when it was tidied away. %@ is a date and a time, such as '1 Jan 2026 at 12:00'.
    static let archiveTidied = Word(key: "archive.tidied")

    /// Archive
    ///
    /// Title of the list of tabs that were tidied away, and the name of its part of the switch beside History, Downloads and Permissions.
    static let archiveTitle = Word(key: "archive.title")

    /// Asleep: %d
    ///
    /// Under the list of tabs that are awake: how many tabs are asleep. %d is the number.
    static let awakeAsleep = Word(key: "awake.asleep")

    /// Done
    ///
    /// Button that closes the list of tabs that are awake.
    static let awakeDone = Word(key: "awake.done")

    /// No tab is awake.
    ///
    /// Shown in the list of tabs that are awake when it lists nothing: every tab is asleep, or has no page.
    static let awakeEmpty = Word(key: "awake.empty")

    /// Awake in private windows: %d
    ///
    /// Under the list of tabs that are awake, in an ordinary window: how many tabs are awake in private windows. They are counted and not named. %d is the number.
    static let awakePrivate = Word(key: "awake.private")

    /// Sleep All Others
    ///
    /// Button in the list of tabs that are awake: puts to sleep every tab that is not on screen and is not doing something that would be lost. 'Others' are all but the tabs on screen.
    static let awakeSleepOthers = Word(key: "awake.sleep_others")

    /// Awake
    ///
    /// Title of the list of tabs that are awake (that have a page loaded, and so are using memory), and the name of its part of the switch beside History, Downloads, Permissions and the Archive. A tab that is not awake is asleep: it keeps its place, and its page is loaded again when it is gone to.
    static let awakeTitle = Word(key: "awake.title")

    /// Using the camera or microphone
    ///
    /// Why a tab is awake, under its name in the list of tabs that are awake. Also said of a tab in the row of tabs, to a screen reader, with the tab's name: the tab has a small recording mark after its name.
    static let awakeWhyCapture = Word(key: "awake.why.capture")

    /// Downloading
    ///
    /// Why a tab is awake, under its name in the list of tabs that are awake: a download its page started is still coming.
    static let awakeWhyDownload = Word(key: "awake.why.download")

    /// On screen
    ///
    /// Why a tab is awake, under its name in the list of tabs that are awake: it is the tab a window is showing.
    static let awakeWhyInSight = Word(key: "awake.why.in_sight")

    /// Loading
    ///
    /// Why a tab is awake, under its name in the list of tabs that are awake: its page is still on its way.
    static let awakeWhyLoading = Word(key: "awake.why.loading")

    /// Asking you something
    ///
    /// Why a tab is awake, under its name in the list of tabs that are awake: its page asked a question that has not been answered, such as whether it may use the camera.
    static let awakeWhyQuestion = Word(key: "awake.why.question")

    /// Playing sound
    ///
    /// Why a tab is awake, under its name in the list of tabs that are awake: its page is playing something that can be heard. Also said of a tab in the row of tabs, to a screen reader, with the tab's name: the tab has a small speaker after its name.
    static let awakeWhySound = Word(key: "awake.why.sound")

    /// Has text you typed
    ///
    /// Why a tab is awake, under its name in the list of tabs that are awake: something was typed into its page, which would be lost if it slept.
    static let awakeWhyTyped = Word(key: "awake.why.typed")

    /// Will sleep by itself
    ///
    /// Under a tab's name in the list of tabs that are awake, when nothing keeps it awake: it is put to sleep when it has been out of sight for half an hour.
    static let awakeWhyWaiting = Word(key: "awake.why.waiting")

    /// Colours
    ///
    /// Design catalogue: heading of the colour tokens.
    static let catalogueColours = Word(key: "catalogue.colours")

    /// Radius
    ///
    /// Design catalogue: heading of the corner-radius tokens.
    static let catalogueRadius = Word(key: "catalogue.radius")

    /// Size
    ///
    /// Design catalogue: heading of the size tokens.
    static let catalogueSize = Word(key: "catalogue.size")

    /// Space
    ///
    /// Design catalogue: heading of the spacing tokens.
    static let catalogueSpace = Word(key: "catalogue.space")

    /// The core would not start
    ///
    /// Design catalogue: heading of what is shown instead of a window when the core refuses to start.
    static let catalogueStartFailed = Word(key: "catalogue.start_failed")

    /// The Archive, nothing tidied
    ///
    /// Design catalogue: the Library with the Archive shown, before anything has been tidied.
    static let catalogueStateArchiveEmpty = Word(key: "catalogue.state.archive_empty")

    /// The Archive
    ///
    /// Design catalogue: the Library with the Archive shown, with tidied tabs in it.
    static let catalogueStateArchiveList = Word(key: "catalogue.state.archive_list")

    /// What is awake, nothing
    ///
    /// Design catalogue: the Library with the list of tabs that are awake shown, when no tab is awake.
    static let catalogueStateAwakeEmpty = Word(key: "catalogue.state.awake_empty")

    /// What is awake
    ///
    /// Design catalogue: the Library with the list of tabs that are awake shown, with tabs in it.
    static let catalogueStateAwakeList = Word(key: "catalogue.state.awake_list")

    /// Tabs playing sound and using the camera
    ///
    /// Design catalogue: a state of the window, in which some tabs have a mark after their names for what their pages are doing.
    static let catalogueStateDoing = Word(key: "catalogue.state.doing")

    /// Downloads: none
    ///
    /// Design catalogue: a state of the Downloads list.
    static let catalogueStateDownloadsEmpty = Word(key: "catalogue.state.downloads_empty")

    /// Downloads: coming, finished and failed
    ///
    /// Design catalogue: a state of the Downloads list.
    static let catalogueStateDownloadsList = Word(key: "catalogue.state.downloads_list")

    /// An empty tab
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateEmpty = Word(key: "catalogue.state.empty")

    /// A page that failed
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateFailed = Word(key: "catalogue.state.failed")

    /// A certificate that isn’t trusted
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateFailedCertificate = Word(key: "catalogue.state.failed_certificate")

    /// The warning before going on past a certificate
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateFailedCertificateWarning = Word(key: "catalogue.state.failed_certificate_warning")

    /// Favourites, as tiles
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateFavourites = Word(key: "catalogue.state.favourites")

    /// Find on the page, with a count
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateFind = Word(key: "catalogue.state.find")

    /// Find on the page, not found
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateFindNone = Word(key: "catalogue.state.find_none")

    /// Editing a Group, refused
    ///
    /// Design catalogue: a state of the page of a Group.
    static let catalogueStateGroupEdit = Word(key: "catalogue.state.group_edit")

    /// Making a Group
    ///
    /// Design catalogue: a state of the page of a Group.
    static let catalogueStateGroupNew = Word(key: "catalogue.state.group_new")

    /// Groups, open and shut
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateGroups = Word(key: "catalogue.state.groups")

    /// Back and forward offered
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateHistory = Word(key: "catalogue.state.history")

    /// The History list, nothing visited
    ///
    /// Design catalogue: the History list before anything has been visited.
    static let catalogueStateHistoryEmpty = Word(key: "catalogue.state.history_empty")

    /// The History list
    ///
    /// Design catalogue: the History list with visits in it.
    static let catalogueStateHistoryList = Word(key: "catalogue.state.history_list")

    /// The History list, nothing found
    ///
    /// Design catalogue: the History list when the words typed find nothing.
    static let catalogueStateHistoryNoneFound = Word(key: "catalogue.state.history_none_found")

    /// A page that is not secure
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateInsecure = Word(key: "catalogue.state.insecure")

    /// A link was pressed and held
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateLinkMenu = Word(key: "catalogue.state.link_menu")

    /// A page loading
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateLoading = Word(key: "catalogue.state.loading")

    /// An address too long for its field
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateLong = Word(key: "catalogue.state.long")

    /// Permissions: none
    ///
    /// Design catalogue: a state of the Permissions list.
    static let catalogueStatePermissionsEmpty = Word(key: "catalogue.state.permissions_empty")

    /// Permissions: sites that were allowed
    ///
    /// Design catalogue: a state of the Permissions list.
    static let catalogueStatePermissionsList = Word(key: "catalogue.state.permissions_list")

    /// A picture that is a link was pressed and held
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePictureLinkMenu = Word(key: "catalogue.state.picture_link_menu")

    /// A picture was pressed and held
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePictureMenu = Word(key: "catalogue.state.picture_menu")

    /// A page asks to use the camera and the microphone
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePrompt = Word(key: "catalogue.state.prompt")

    /// A page says something
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePromptAlert = Word(key: "catalogue.state.prompt_alert")

    /// A page asks yes or no, not for the first time
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePromptConfirm = Word(key: "catalogue.state.prompt_confirm")

    /// A page asks to download a file
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePromptDownload = Word(key: "catalogue.state.prompt_download")

    /// A server asks who is there
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePromptSignIn = Word(key: "catalogue.state.prompt_sign_in")

    /// A server asks again, over a connection that is not private
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePromptSignInAgain = Word(key: "catalogue.state.prompt_sign_in_again")

    /// A page asks for a word
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStatePromptText = Word(key: "catalogue.state.prompt_text")

    /// Changing where searches go
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateSearchChange = Word(key: "catalogue.state.search_change")

    /// The first run asks where searches go
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateSearchQuestion = Word(key: "catalogue.state.search_question")

    /// A secure page
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateSecure = Word(key: "catalogue.state.secure")

    /// Tabs at the side
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateSidebar = Word(key: "catalogue.state.sidebar")

    /// Sites, awake and asleep
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateSites = Word(key: "catalogue.state.sites")

    /// The list of Spaces
    ///
    /// Design catalogue: a state of the list of Spaces.
    static let catalogueStateSpaces = Word(key: "catalogue.state.spaces")

    /// Making a Space
    ///
    /// Design catalogue: a state of the list of Spaces.
    static let catalogueStateSpacesNew = Word(key: "catalogue.state.spaces_new")

    /// Making a Space that shares logins
    ///
    /// Design catalogue: a state of the list of Spaces.
    static let catalogueStateSpacesNewShared = Word(key: "catalogue.state.spaces_new_shared")

    /// A Space the core would not make
    ///
    /// Design catalogue: a state of the list of Spaces.
    static let catalogueStateSpacesRefused = Word(key: "catalogue.state.spaces_refused")

    /// Editing a Space
    ///
    /// Design catalogue: a state of the list of Spaces.
    static let catalogueStateSpacesRename = Word(key: "catalogue.state.spaces_rename")

    /// Places offered as an address is typed
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateSuggestions = Word(key: "catalogue.state.suggestions")

    /// The phone’s switcher
    ///
    /// Design catalogue: the sheet that lists every tab of the row, in a narrow window.
    static let catalogueStateSwitcher = Word(key: "catalogue.state.switcher")

    /// Several tabs
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateTabs = Word(key: "catalogue.state.tabs")

    /// A tab waking, with its page as it was
    ///
    /// Design catalogue: a state of the window, in which a tab that was asleep is loading and a picture of its page as it was stands where the page will be.
    static let catalogueStateWaking = Word(key: "catalogue.state.waking")

    /// The zoom bar, on a page made bigger
    ///
    /// Design catalogue: a state of the window.
    static let catalogueStateZoom = Word(key: "catalogue.state.zoom")

    /// The Window
    ///
    /// Design catalogue: heading of the browser window's states.
    static let catalogueStates = Word(key: "catalogue.states")

    /// Design Catalogue
    ///
    /// Title of the design catalogue: a screen for designers that shows every design token and every state of the window.
    static let catalogueTitle = Word(key: "catalogue.title")

    /// Advanced…
    ///
    /// Button on the page shown when a site's certificate isn't trusted, under 'Try Again': shows the warning, and under it the way to go on to the site all the same.
    static let certificateAdvanced = Word(key: "certificate.advanced")

    /// Go On to %@ (Not Safe)
    ///
    /// Button under the warning: loads the site although its certificate isn't trusted. %@ is the site's host name. The words in brackets say what the button does and must stay.
    static let certificateProceed = Word(key: "certificate.proceed")

    /// WebTiger can’t confirm that this is really %@. Someone could be pretending to be it, and could read or change what you send and receive: passwords, messages, card numbers.
    ///
    /// The warning shown before a person can go on to a site whose certificate isn't trusted. %@ is the site's host name, such as router.example.com.
    static let certificateWarning = Word(key: "certificate.warning")

    /// Go on only if you know why this site’s certificate isn’t trusted, such as a router or a device of your own.
    ///
    /// Second sentence of the warning before going on to a site whose certificate isn't trusted.
    static let certificateWarningOnlyIf = Word(key: "certificate.warning.only_if")

    /// Going on isn’t safe
    ///
    /// Heading of the warning shown before a person can go on to a site whose certificate isn't trusted.
    static let certificateWarningTitle = Word(key: "certificate.warning.title")

    /// Downloading
    ///
    /// Accessibility value of the History and Downloads button while a download is coming.
    static let downloadsActive = Word(key: "downloads.active")

    /// Clear List
    ///
    /// Button in the Downloads list that takes every download off the list. The files stay where they are.
    static let downloadsClear = Word(key: "downloads.clear")

    /// %1$@ of %2$@
    ///
    /// Under a download that is still coming, when its size is known: how much has arrived, and how much there is. Both are sizes with their units, such as “1.2 MB”.
    static let downloadsComing = Word(key: "downloads.coming")

    /// %@ so far
    ///
    /// Under a download that is still coming, when its size is not known: how much has arrived, as a size with its unit.
    static let downloadsComingSoFar = Word(key: "downloads.coming_so_far")

    /// Done
    ///
    /// Button that closes the Downloads list.
    static let downloadsDone = Word(key: "downloads.done")

    /// Files you download will be listed here.
    ///
    /// Shown in the Downloads list when nothing has been downloaded.
    static let downloadsEmpty = Word(key: "downloads.empty")

    /// Remove from List
    ///
    /// Accessibility label of the button beside a download that has ended, which takes it off the list. The file stays.
    static let downloadsForget = Word(key: "downloads.forget")

    /// Show Older Downloads
    ///
    /// Button at the end of the Downloads list that shows the next, older page of downloads.
    static let downloadsOlder = Word(key: "downloads.older")

    /// Open
    ///
    /// What pressing a finished download in the list does, as a screen reader says it: opens the file with whatever the device has for its kind.
    static let downloadsOpen = Word(key: "downloads.open")

    /// The file couldn’t be opened.
    ///
    /// Under a download in the list, after a person pressed it to open its file, when it did not open for a reason this build has no words for.
    static let downloadsOpenFailed = Word(key: "downloads.open_failed")

    /// Stop Download
    ///
    /// Accessibility label of the button beside a download that is still coming, which stops it.
    static let downloadsStop = Word(key: "downloads.stop")

    /// Downloads
    ///
    /// Title of the list of downloads, and the name of its half of the switch beside History.
    static let downloadsTitle = Word(key: "downloads.title")

    /// Redo
    ///
    /// Command in the Mac's Edit menu (Shift-Command-Z): do again what was undone in the field that has the keyboard.
    static let editRedo = Word(key: "edit.redo")

    /// Stopped
    ///
    /// Under a download that was stopped.
    static let errorDownloadCancelled = Word(key: "error.download.cancelled")

    /// The file couldn’t be saved.
    ///
    /// Under a download that failed because its file could not be written.
    static let errorDownloadDisk = Word(key: "error.download.disk")

    /// The download failed.
    ///
    /// Under a download that failed for a reason this build has no words for.
    static let errorDownloadFailed = Word(key: "error.download.failed")

    /// WebTiger was closed before it finished.
    ///
    /// Under a download that was still coming when the app ended.
    static let errorDownloadInterrupted = Word(key: "error.download.interrupted")

    /// The file isn’t where it was saved any more.
    ///
    /// Under a download in the list, after a person pressed it to open its file: the file was moved, renamed or removed since it was downloaded.
    static let errorDownloadMissing = Word(key: "error.download.missing")

    /// The connection was lost.
    ///
    /// Under a download that failed because of the network.
    static let errorDownloadNetwork = Word(key: "error.download.network")

    /// Private windows aren’t available on this device yet.
    ///
    /// The engine cannot make a login set that is kept in memory only.
    static let errorEngineNoPrivate = Word(key: "error.engine.no_private")

    /// The web engine on this device can’t keep each Space’s logins separate. Update Android System WebView and try again.
    ///
    /// Android: the System WebView is too old to give each login set a profile of its own.
    static let errorEngineNoProfiles = Word(key: "error.engine.no_profiles")

    /// This page’s engine isn’t available.
    ///
    /// The web engine the tab asked for is not installed.
    static let errorEngineUnavailable = Word(key: "error.engine.unavailable")

    /// The Group couldn’t be changed.
    ///
    /// Said on the page that edits a Group when it could not be changed and no more is known.
    static let errorGroupChangeFailed = Word(key: "error.group.change_failed")

    /// That colour isn’t one a Group can have.
    ///
    /// Said on the page that makes or edits a Group when the app does not know the colour that was chosen.
    static let errorGroupColourUnknown = Word(key: "error.group.colour_unknown")

    /// The Group couldn’t be made.
    ///
    /// Said on the page that makes a Group when it could not be made and no more is known.
    static let errorGroupFailed = Word(key: "error.group.failed")

    /// A Group needs a name.
    ///
    /// Said on the page that makes or edits a Group when the name is empty.
    static let errorGroupNameEmpty = Word(key: "error.group.name_empty")

    /// There are as many Groups as there can be.
    ///
    /// Said on the page that makes a Group when the row has as many as it can.
    static let errorGroupTooMany = Word(key: "error.group.too_many")

    /// That Group isn’t there any more.
    ///
    /// Said on the page that edits a Group when the Group has been removed meanwhile.
    static let errorGroupUnknown = Word(key: "error.group.unknown")

    /// The site couldn’t be reached.
    ///
    /// A load failed: connection refused or lost.
    static let errorNetConnect = Word(key: "error.net.connect")

    /// The site’s address couldn’t be found.
    ///
    /// A load failed: DNS.
    static let errorNetDns = Word(key: "error.net.dns")

    /// Something went wrong while loading the page.
    ///
    /// A load failed for a reason without words of its own.
    static let errorNetFailed = Word(key: "error.net.failed")

    /// You’re not connected to the internet.
    ///
    /// A load failed: no network.
    static let errorNetOffline = Word(key: "error.net.offline")

    /// The site took too long to answer.
    ///
    /// A load failed: timed out.
    static let errorNetTimeout = Word(key: "error.net.timeout")

    /// A secure connection to the site couldn’t be made.
    ///
    /// A load failed: TLS or certificate problem.
    static let errorNetTls = Word(key: "error.net.tls")

    /// This site’s certificate isn’t valid today. It may have run out, or this device’s date may be wrong.
    ///
    /// A load failed: the site's certificate has expired or is not valid yet, by this device's clock.
    static let errorNetTlsDate = Word(key: "error.net.tls.date")

    /// This site’s certificate is for a different site.
    ///
    /// A load failed: the certificate the site presented names another site.
    static let errorNetTlsName = Word(key: "error.net.tls.name")

    /// This device doesn’t trust this site’s certificate.
    ///
    /// A load failed: the site's certificate signs itself, or was signed by an authority this device does not know.
    static let errorNetTlsUntrusted = Word(key: "error.net.tls.untrusted")

    /// The page stopped unexpectedly.
    ///
    /// The process drawing the page died.
    static let errorPageRendererGone = Word(key: "error.page.renderer_gone")

    /// Try Again
    ///
    /// Button on the failed-page view: reload.
    static let errorRetry = Word(key: "error.retry")

    /// That colour isn’t one a Space can have.
    ///
    /// Said where a Space is made or edited when the app does not know the colour that was chosen.
    static let errorSpaceColourUnknown = Word(key: "error.space.colour_unknown")

    /// The Space couldn’t be made.
    ///
    /// Said on the page that makes a Space when it could not be made and no more is known.
    static let errorSpaceFailed = Word(key: "error.space.failed")

    /// That icon isn’t one a Space can have.
    ///
    /// Said where a Space is made or edited when the app does not know the icon that was chosen.
    static let errorSpaceIconUnknown = Word(key: "error.space.icon_unknown")

    /// A Space needs a name.
    ///
    /// Said on the page that makes a Space when the name is empty.
    static let errorSpaceNameEmpty = Word(key: "error.space.name_empty")

    /// The Space couldn’t be changed.
    ///
    /// Said on the page that edits a Space when it could not be changed and no more is known.
    static let errorSpaceRenameFailed = Word(key: "error.space.rename_failed")

    /// There are as many Spaces as there can be.
    ///
    /// Said on the page that makes a Space when the app will not make another: it has its limit of them.
    static let errorSpaceTooMany = Word(key: "error.space.too_many")

    /// That Space isn’t there any more.
    ///
    /// Said on the page that makes a Space when the Space whose logins it was to share is gone.
    static let errorSpaceUnknown = Word(key: "error.space.unknown")

    /// Error code: %@
    ///
    /// Shown under the start failure heading. %@ is a code such as store.open.
    static let errorStartDetail = Word(key: "error.start.detail")

    /// WebTiger couldn’t start
    ///
    /// Heading when the core refused to start.
    static let errorStartTitle = Word(key: "error.start.title")

    /// Nothing on this device opens this kind of file.
    ///
    /// Under a download in the list, after a person pressed it to open its file: no app on the device opens files of its kind.
    static let errorSystemNoHandler = Word(key: "error.system.no_handler")

    /// That isn’t a time tabs can be tidied after.
    ///
    /// Said on the page that edits a Space when the app does not know the time that was chosen.
    static let errorTidyTimeUnknown = Word(key: "error.tidy.time_unknown")

    /// This page didn’t load
    ///
    /// Heading shown over a page that failed to load.
    static let errorTitle = Word(key: "error.title")

    /// Add to Favourites
    ///
    /// Command in a tab's own menu (on a Mac in the File menu, for the tab in front): the tab becomes one of the row's Favourites, the small tiles at its beginning. A tab that is no Site is kept as one by it. Grey when the row has as many Favourites as it can.
    static let favouritesAdd = Word(key: "favourites.add")

    /// Favourite
    ///
    /// Said of a tab that is a Favourite, to a screen reader, with the tab's name: its tile shows only a letter.
    static let favouritesFavourite = Word(key: "favourites.favourite")

    /// Remove from Favourites
    ///
    /// The same command for a tab that is a Favourite: it is a Site like any other again.
    static let favouritesRemove = Word(key: "favourites.remove")

    /// %ld of %ld
    ///
    /// Shown in the find bar where the engine counts: which match is shown, and how many there are. First %ld is the match's number, second the total.
    static let findCount = Word(key: "find.count")

    /// Done
    ///
    /// Button that closes the find bar.
    static let findDone = Word(key: "find.done")

    /// Find on page
    ///
    /// Placeholder and accessibility label of the find bar's field.
    static let findField = Word(key: "find.field")

    /// Next Match
    ///
    /// Button in the find bar: go to the next match.
    static let findNext = Word(key: "find.next")

    /// Not found
    ///
    /// Shown in the find bar when the text is not on the page.
    static let findNone = Word(key: "find.none")

    /// Find on Page
    ///
    /// Label of the button, beside the tabs, that opens the find bar; and the command (Command-F on a Mac).
    static let findOpen = Word(key: "find.open")

    /// Previous Match
    ///
    /// Button in the find bar: go to the match before.
    static let findPrevious = Word(key: "find.previous")

    /// Cancel
    ///
    /// Button on the page that makes or edits a Group: back to the window, with nothing made or changed.
    static let groupCancel = Word(key: "group.cancel")

    /// Edit Group…
    ///
    /// In a Group's own menu (a long press on its chip; a right-click on a Mac): opens the page that gives it another name or colour.
    static let groupEdit = Word(key: "group.edit")

    /// Edit Group
    ///
    /// Heading of the page that gives a Group another name or colour.
    static let groupEditTitle = Word(key: "group.edit.title")

    /// Make Group
    ///
    /// Button that makes the new Group and puts the tab in it.
    static let groupMake = Word(key: "group.make")

    /// Name
    ///
    /// Label of the field for a Group's name.
    static let groupName = Word(key: "group.name")

    /// New Group
    ///
    /// Heading of the page that makes a Group.
    static let groupNewTitle = Word(key: "group.new.title")

    /// Remove Group
    ///
    /// In a Group's own menu: the Group is taken away. Its Sites stay, in no Group.
    static let groupRemove = Word(key: "group.remove")

    /// Group, open
    ///
    /// Said of a Group's chip to a screen reader, with the Group's name: its Sites are shown after it. Pressing it shuts it.
    static let groupSaidOpen = Word(key: "group.said.open")

    /// Group, shut, Sites: %d
    ///
    /// Said of a Group's chip to a screen reader, with the Group's name: its Sites are out of the way. %d is how many it holds. Pressing it opens it.
    static let groupSaidShut = Word(key: "group.said.shut")

    /// Save
    ///
    /// Button that gives the Group the name in the field and the colour chosen.
    static let groupSave = Word(key: "group.save")

    /// %1$@ (%2$d)
    ///
    /// A shut Group's chip in the row of tabs: the Group's name, and how many Sites it holds.
    static let groupShutLabel = Word(key: "group.shut_label")

    /// Move to Group
    ///
    /// A menu within a tab's own menu (on a Mac in the File menu, for the tab in front), and on Android the heading over the same choices in the tab's menu: the Groups of the row by name, the one the tab is in marked. Choosing one puts the tab in that Group; a tab that is no Site is kept as one by it.
    static let groupsMove = Word(key: "groups.move")

    /// New Group…
    ///
    /// In 'Move to Group': opens the page that makes a Group, and puts the tab in it.
    static let groupsNew = Word(key: "groups.new")

    /// No Group
    ///
    /// In 'Move to Group', after the Groups: the choice for a tab that is in none. Chosen for a Site that is in a Group, it takes it out; it stays a Site.
    static let groupsNone = Word(key: "groups.none")

    /// Clear…
    ///
    /// Button in the History list that asks which visits to forget.
    static let historyClear = Word(key: "history.clear")

    /// Cancel
    ///
    /// Choice: leave history as it is.
    static let historyClearCancel = Word(key: "history.clear.cancel")

    /// Visits that are forgotten can’t be brought back.
    ///
    /// Under the question of which visits to forget.
    static let historyClearDetail = Word(key: "history.clear.detail")

    /// Everything
    ///
    /// Choice: forget every visit.
    static let historyClearEverything = Word(key: "history.clear.everything")

    /// The Last Hour
    ///
    /// Choice: forget the visits of the last hour.
    static let historyClearHour = Word(key: "history.clear.hour")

    /// Forget which visits?
    ///
    /// Question asked by the Clear button of the History list, above the choices.
    static let historyClearTitle = Word(key: "history.clear.title")

    /// Today
    ///
    /// Choice: forget the visits since midnight.
    static let historyClearToday = Word(key: "history.clear.today")

    /// Done
    ///
    /// Button that closes the History list.
    static let historyDone = Word(key: "history.done")

    /// Pages you visit will be listed here.
    ///
    /// Shown in the History list when nothing has been visited yet.
    static let historyEmpty = Word(key: "history.empty")

    /// Forget This Visit
    ///
    /// Accessibility label of the button beside a visit that removes it from history.
    static let historyForget = Word(key: "history.forget")

    /// No visits match.
    ///
    /// Shown in the History list when the words typed find no visit.
    static let historyNoneFound = Word(key: "history.none_found")

    /// Show Older Visits
    ///
    /// Button at the end of the History list that shows the next, older page of visits.
    static let historyOlder = Word(key: "history.older")

    /// History and Downloads
    ///
    /// Label of the button, beside the tabs, that opens the lists of pages visited and of files downloaded.
    static let historyOpen = Word(key: "history.open")

    /// Search history
    ///
    /// Placeholder and accessibility label of the field for finding a visit by words of its title or address.
    static let historySearch = Word(key: "history.search")

    /// History
    ///
    /// Title of the list of pages visited.
    static let historyTitle = Word(key: "history.title")

    /// Tabs at the Side
    ///
    /// Command in the Mac's View menu (Control-Command-S) while the window's tabs are along its top: they are drawn down its side instead, in a sidebar. Nothing else changes.
    static let layoutSide = Word(key: "layout.side")

    /// Tabs at the Top
    ///
    /// The same command while the tabs are down the side: they are drawn along the top again.
    static let layoutTop = Word(key: "layout.top")

    /// Cancel
    ///
    /// In a link's menu: close the menu and do nothing.
    static let linkCancel = Word(key: "link.cancel")

    /// Copy Link
    ///
    /// In a link's menu: copy the link's address.
    static let linkCopy = Word(key: "link.copy")

    /// Download Linked File
    ///
    /// In a link's menu: save what the link leads to as a file, with the person's downloads.
    static let linkDownload = Word(key: "link.download")

    /// Download Image
    ///
    /// In a picture's menu: save the picture as a file, with the person's downloads.
    static let linkDownloadImage = Word(key: "link.download_image")

    /// Image
    ///
    /// Accessibility name of the menu shown for a picture of a page that was pressed and held.
    static let linkImageMenu = Word(key: "link.image_menu")

    /// Link
    ///
    /// Accessibility name of the menu shown for a link of a page that was pressed and held.
    static let linkMenu = Word(key: "link.menu")

    /// Open Image in New Tab
    ///
    /// In a picture's menu (on a Mac beside the system's own items such as Copy Image): open the picture by itself in a new tab.
    static let linkOpenImageNewTab = Word(key: "link.open_image_new_tab")

    /// Open Link in New Tab
    ///
    /// In the menu a Mac shows for a link (right-click), beside the system's own items such as Copy Link: open the link in a new tab.
    static let linkOpenLinkNewTab = Word(key: "link.open_link_new_tab")

    /// Open in New Tab
    ///
    /// In a link's menu: open the link in a new tab, behind the one being read.
    static let linkOpenNewTab = Word(key: "link.open_new_tab")

    /// Close Window
    ///
    /// Command in the Mac's File menu (Shift-Command-W): close the window, however many tabs it has.
    static let menuCloseWindow = Word(key: "menu.close_window")

    /// History
    ///
    /// Name of a menu in the Mac's menu bar: going back and forward, and the list of pages visited.
    static let menuHistory = Word(key: "menu.history")

    /// Search with WebTiger
    ///
    /// Android: the item WebTiger adds to other apps' menu of selected text. Choosing it opens the words as a search in WebTiger, with the search engine the person chose. Short: it sits in a small bar.
    static let outsideSearch = Word(key: "outside.search")

    /// The page as it was
    ///
    /// Said by a screen reader of the picture that stands where a page will be while a tab that was asleep loads it again: a picture of the page as it was when the tab went to sleep. Not a button.
    static let pagePicture = Word(key: "page.picture")

    /// Remove All
    ///
    /// Button in the Permissions list that takes back what every site was allowed. Each will ask again.
    static let permissionsClear = Word(key: "permissions.clear")

    /// Done
    ///
    /// Button that closes the Permissions list.
    static let permissionsDone = Word(key: "permissions.done")

    /// Sites you allow to download, or to use your camera, microphone or location, will be listed here.
    ///
    /// Shown in the Permissions list when no site was allowed anything.
    static let permissionsEmpty = Word(key: "permissions.empty")

    /// Stop Allowing
    ///
    /// Accessibility label of the button beside a site in the Permissions list, which takes back what it was allowed. The site will ask again.
    static let permissionsForget = Word(key: "permissions.forget")

    /// Camera
    ///
    /// What a site was allowed to use, under its name in the Permissions list. Several are joined as a list by the system.
    static let permissionsKindCamera = Word(key: "permissions.kind.camera")

    /// Downloads
    ///
    /// What a site was allowed to do, under its name in the Permissions list: download files without asking each time.
    static let permissionsKindDownloads = Word(key: "permissions.kind.downloads")

    /// Location
    ///
    /// What a site was allowed to use, under its name in the Permissions list: where the device is.
    static let permissionsKindLocation = Word(key: "permissions.kind.location")

    /// Microphone
    ///
    /// What a site was allowed to use, under its name in the Permissions list.
    static let permissionsKindMicrophone = Word(key: "permissions.kind.microphone")

    /// Permissions
    ///
    /// Title of the list of sites that were allowed to download, or to use the camera, the microphone or the location, and the name of its part of the switch beside History and Downloads.
    static let permissionsTitle = Word(key: "permissions.title")

    /// Back to %@
    ///
    /// Button on the page behind a private window's chip on a phone: shows the other window again and leaves the private one open. %@ is the name of the Space the other window shows.
    static let privateBack = Word(key: "private.back")

    /// Private
    ///
    /// Chip in the strip of tabs of a private window, where an ordinary window names its Space. Short: on a narrow strip only its first letter is shown.
    static let privateChip = Word(key: "private.chip")

    /// Close Private Window
    ///
    /// Button on the page behind a private window's chip, and command: closes the private window, and everything of it is gone.
    static let privateClose = Word(key: "private.close")

    /// It does not hide you from the sites you visit, or from your network.
    ///
    /// What a private window does not do: follows the sentence that says what it means.
    static let privateHides = Word(key: "private.hides")

    /// Nothing you visit in this window is kept on this device: no history, and nothing a site stored once the window is closed. Files you save stay.
    ///
    /// What a private window means: in an empty tab of one, and on the page behind its chip.
    static let privateKeeps = Word(key: "private.keeps")

    /// New Private Window
    ///
    /// Command in the Mac's File menu (Shift-Command-N): open a private window.
    static let privateNew = Word(key: "private.new")

    /// Nothing it visits is kept on this device.
    ///
    /// Under 'Private Window' in the list of Spaces.
    static let privateRowDetail = Word(key: "private.row.detail")

    /// Open
    ///
    /// Said beside 'Private Window' in the list of Spaces when a private window is open already: the row then shows it.
    static let privateRowOpen = Word(key: "private.row.open")

    /// Private Window
    ///
    /// Name of a private window: the last row of the list of Spaces on a phone (it opens one), the heading of the page that explains it, and what a screen reader says for the chip.
    static let privateWindow = Word(key: "private.window")

    /// Allow
    ///
    /// Button in a tab's prompt: lets the site use what it asked for, now and when it asks again.
    static let promptAllow = Word(key: "prompt.allow")

    /// Your answer
    ///
    /// Accessibility label of the field in the bar where a page asks for a line of text.
    static let promptAnswer = Word(key: "prompt.answer")

    /// Cancel
    ///
    /// Button in the bar that shows what a page asks: refuses it.
    static let promptCancel = Word(key: "prompt.cancel")

    /// %1$@ wants to download “%2$@”.
    ///
    /// The sentence in the bar a tab shows when a page starts a download and its site was not yet allowed to. %1$@ is the site, as the address field shows it; %2$@ is the name the file would be saved under, in quotation marks so that where it begins and ends can be seen.
    static let promptDownload = Word(key: "prompt.download")

    /// your camera
    ///
    /// What a page asks to use, in the prompt.permission sentence.
    static let promptKindCamera = Word(key: "prompt.kind.camera")

    /// your location
    ///
    /// What a page asks to use, in the prompt.permission sentence: where the device is.
    static let promptKindLocation = Word(key: "prompt.kind.location")

    /// your microphone
    ///
    /// What a page asks to use, in the prompt.permission sentence.
    static let promptKindMicrophone = Word(key: "prompt.kind.microphone")

    /// OK
    ///
    /// Button in the bar that shows what a page says or asks: accepts it.
    static let promptOk = Word(key: "prompt.ok")

    /// %1$@ wants to use %2$@.
    ///
    /// The sentence in the bar a tab shows when a page asks to use something of the device's. %1$@ is the site, as the address field shows it; %2$@ is what it wants: one or more of the prompt.kind strings, joined as a list by the system (in English: “your camera and your microphone”).
    static let promptPermission = Word(key: "prompt.permission")

    /// Don’t Allow
    ///
    /// Button in a tab's prompt: refuses what the page asked for, this time.
    static let promptRefuse = Word(key: "prompt.refuse")

    /// %@ says:
    ///
    /// The first line of the bar a tab shows when a page says something or asks (alert, confirm, prompt). %@ is the site, as the address field shows it; the page's own words follow on the next lines.
    static let promptSays = Word(key: "prompt.says")

    /// Sign in to %@
    ///
    /// The first line of the bar a tab shows when a server asks for a name and a password before it answers (HTTP authentication). %@ is the server, as the address field shows a site: it is who gets what is typed.
    static let promptSignIn = Word(key: "prompt.sign_in")

    /// That name or password was not accepted.
    ///
    /// In the sign-in bar, when the server refused what was sent before and asks again.
    static let promptSignInAgain = Word(key: "prompt.sign_in.again")

    /// Sign In
    ///
    /// Button in the sign-in bar: sends the name and the password to the server that asked.
    static let promptSignInGo = Word(key: "prompt.sign_in.go")

    /// Name
    ///
    /// The sign-in bar's first field: the name to sign in with. Shown in the empty field and read as its label.
    static let promptSignInName = Word(key: "prompt.sign_in.name")

    /// This connection is not private. What you type could be read by others.
    ///
    /// In the sign-in bar, when the server is not reached over https: the name and password travel unprotected.
    static let promptSignInNotPrivate = Word(key: "prompt.sign_in.not_private")

    /// Password
    ///
    /// The sign-in bar's second field. Shown in the empty field and read as its label.
    static let promptSignInPassword = Word(key: "prompt.sign_in.password")

    /// Stop This Page’s Messages
    ///
    /// Button in the bar that shows what a page says, from the page's second message on: this page is shown no more messages until another page loads.
    static let promptSilence = Word(key: "prompt.silence")

    /// Change…
    ///
    /// Button beside 'Searches go to …' on an empty tab: opens the list of search engines.
    static let searchChange = Word(key: "search.change")

    /// Default
    ///
    /// Said of the search engine that searches go to when nobody has chosen another, in the list of search engines at the first run.
    static let searchDefault = Word(key: "search.default")

    /// Searches go to %@.
    ///
    /// On an empty tab, under its hint. %@ is a search engine's own name, such as Ecosia.
    static let searchGoesTo = Word(key: "search.goes_to")

    /// In use
    ///
    /// Said of the search engine that searches go to now, in the list of search engines when it is opened to change it.
    static let searchInUse = Word(key: "search.in_use")

    /// Cancel
    ///
    /// Button that closes the list of search engines without changing where searches go. Not shown at the first run, when one has to be chosen.
    static let searchKeep = Word(key: "search.keep")

    /// Keep %@
    ///
    /// Button under the list of search engines at the first run: leaves searches going to the default engine. %@ is that engine's own name, such as DuckDuckGo.
    static let searchKeepDefault = Word(key: "search.keep_default")

    /// What you type in the address field that is not an address is sent to the search engine you choose. Nothing is sent until you search.
    ///
    /// Under the heading of the list of search engines.
    static let searchQuestionDetail = Word(key: "search.question.detail")

    /// Where should your searches go?
    ///
    /// Heading of the first run's question, and of the same list when it is opened again: the person chooses a search engine.
    static let searchQuestionTitle = Word(key: "search.question.title")

    /// Search with %@
    ///
    /// In the menu of words that are selected in a page: search for those words, in a new tab. %@ is the search engine's own name, such as Ecosia.
    static let selectionSearchWith = Word(key: "selection.search_with")

    /// Asleep
    ///
    /// Said of a tab that has no page loaded now, to a screen reader, with the tab's name: a Site that was closed, or a tab that came back after a quit. Opening it loads it.
    static let sitesAsleep = Word(key: "sites.asleep")

    /// Away
    ///
    /// Said of a Site that is awake and shows a page that is not its home, to a screen reader, with the tab's name.
    static let sitesAway = Word(key: "sites.away")

    /// Keep as Site
    ///
    /// Command (Command-D on a Mac, Control-D with a keyboard on Android; in a tab's own menu on a phone): keep the tab as a Site. A Site stays first in the row, and is put to sleep, not closed, when it is closed.
    static let sitesKeep = Word(key: "sites.keep")

    /// Stop Keeping
    ///
    /// The same command for a tab that is a Site: it becomes a tab like any other again.
    static let sitesLetGo = Word(key: "sites.let_go")

    /// Return to %@
    ///
    /// Command for a Site that has wandered from its home (Shift-Command-H on a Mac, Alt-Home with a keyboard on Android; in the tab's own menu on a phone): its home is loaded in it again. %@ is the name the Site was kept with.
    static let sitesReturn = Word(key: "sites.return")

    /// Return to Site
    ///
    /// The same command in the Mac's menu bar while there is nothing for it to do (the tab in front is no Site, or is at its home): shown grey.
    static let sitesReturnSite = Word(key: "sites.return_site")

    /// Site
    ///
    /// Said of a tab that is kept, to a screen reader, with the tab's name.
    static let sitesSite = Word(key: "sites.site")

    /// Put to Sleep
    ///
    /// Label of the button at the end of a Site's tab, where another tab has Close Tab: the Site's page is closed and the Site stays in the row.
    static let sitesSleep = Word(key: "sites.sleep")

    /// If you choose none, it gets one that no other Space has.
    ///
    /// Under the colours on the page that makes a Space.
    static let spaceAutomaticColour = Word(key: "space.automatic_colour")

    /// Space: %@
    ///
    /// Accessibility name of the button at the start of the strip of tabs that says which Space the window shows, and opens the list of Spaces. %@ is the Space's name.
    static let spaceChip = Word(key: "space.chip")

    /// Colour: %@
    ///
    /// The same heading when a colour is chosen: it names the colour in words, so that it is not told by the colour alone. %@ is the colour's name, such as Blue.
    static let spaceChosenColour = Word(key: "space.chosen_colour")

    /// Colour
    ///
    /// Heading over the colours to choose among for a Space or a Group, where one is made and where one is edited.
    static let spaceColour = Word(key: "space.colour")

    /// Blue
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourBlue = Word(key: "space.colour.blue")

    /// Green
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourGreen = Word(key: "space.colour.green")

    /// Grey
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourGrey = Word(key: "space.colour.grey")

    /// Orange
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourOrange = Word(key: "space.colour.orange")

    /// Pink
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourPink = Word(key: "space.colour.pink")

    /// Purple
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourPurple = Word(key: "space.colour.purple")

    /// Red
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourRed = Word(key: "space.colour.red")

    /// Teal
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourTeal = Word(key: "space.colour.teal")

    /// Yellow
    ///
    /// Name of a colour a Space can have: read by a screen reader for its swatch, and shown beside the heading when it is chosen.
    static let spaceColourYellow = Word(key: "space.colour.yellow")

    /// Home
    ///
    /// Name shown for the Space the app made itself, until the person gives it a name. A Space is a set of tabs kept apart from the others, such as home and work.
    static let spaceHome = Word(key: "space.home")

    /// Icon
    ///
    /// Heading over the icons to choose among for a Space, where one is made and where one is edited. The icons are emoji.
    static let spaceIcon = Word(key: "space.icon")

    /// Back
    ///
    /// Button on the page that makes a Space: back to the list of Spaces, with nothing made.
    static let spaceNewBack = Word(key: "space.new.back")

    /// Logins
    ///
    /// Heading over the choice of whose logins a new Space has: its own, or another Space's.
    static let spaceNewLogins = Word(key: "space.new.logins")

    /// Make Space
    ///
    /// Button that makes the new Space and shows it in the window.
    static let spaceNewMake = Word(key: "space.new.make")

    /// Name
    ///
    /// Label of the field for a new Space's name.
    static let spaceNewName = Word(key: "space.new.name")

    /// Its own
    ///
    /// Choice for a new Space: it is signed in to nothing, apart from every other Space.
    static let spaceNewOwn = Word(key: "space.new.own")

    /// Signed out everywhere to begin with. What you sign in to in this Space stays in it.
    ///
    /// Under the choice 'Its own'.
    static let spaceNewOwnDetail = Word(key: "space.new.own.detail")

    /// Share with
    ///
    /// Label of the choice of which Space a new Space shares its logins with.
    static let spaceNewShareWith = Word(key: "space.new.share_with")

    /// The same as another Space
    ///
    /// Choice for a new Space: it is signed in to sites as another Space is.
    static let spaceNewShared = Word(key: "space.new.shared")

    /// Signed in wherever %@ is. The two share history and downloads.
    ///
    /// Under the choice 'The same as another Space'. %@ is the name of the Space whose logins are shared.
    static let spaceNewSharedDetail = Word(key: "space.new.shared.detail")

    /// New Space
    ///
    /// Heading of the page that makes a Space.
    static let spaceNewTitle = Word(key: "space.new.title")

    /// None
    ///
    /// The choice of no colour for a Space or a Group, beside the colours.
    static let spaceNoColour = Word(key: "space.no_colour")

    /// No icon
    ///
    /// Accessibility label of the first choice among a Space's icons: none. The other choices are emoji, which a screen reader names itself.
    static let spaceNoIcon = Word(key: "space.no_icon")

    /// Save
    ///
    /// Button that gives the Space the name in the field.
    static let spaceRenameSave = Word(key: "space.rename.save")

    /// Edit Space
    ///
    /// Heading of the page that gives a Space another name or colour.
    static let spaceRenameTitle = Word(key: "space.rename.title")

    /// Tidy tabs away
    ///
    /// Heading over the choices of when a Space's tabs are tidied away, on the page that edits a Space.
    static let spaceTidy = Word(key: "space.tidy")

    /// Never
    ///
    /// A choice of when a Space's tabs are tidied away, on the page that edits a Space. Also what the Space is said to have.
    static let spaceTidy0 = Word(key: "space.tidy.0")

    /// After 30 days
    ///
    /// A choice of when a Space's tabs are tidied away, on the page that edits a Space. Also what the Space is said to have.
    static let spaceTidy2592000 = Word(key: "space.tidy.2592000")

    /// After 12 hours
    ///
    /// A choice of when a Space's tabs are tidied away, on the page that edits a Space. Also what the Space is said to have.
    static let spaceTidy43200 = Word(key: "space.tidy.43200")

    /// After a week
    ///
    /// A choice of when a Space's tabs are tidied away, on the page that edits a Space. Also what the Space is said to have.
    static let spaceTidy604800 = Word(key: "space.tidy.604800")

    /// After a day
    ///
    /// A choice of when a Space's tabs are tidied away, on the page that edits a Space. Also what the Space is said to have.
    static let spaceTidy86400 = Word(key: "space.tidy.86400")

    /// Tabs you haven’t looked at for this long go to this Space’s Archive, where you can get them back. Sites and Favourites always stay.
    ///
    /// Under the heading of the choices of when a Space's tabs are tidied away.
    static let spaceTidyDetail = Word(key: "space.tidy.detail")

    /// A Space keeps its tabs apart from the others. Choose one to show it in this window.
    ///
    /// Under the heading of the list of Spaces.
    static let spacesDetail = Word(key: "spaces.detail")

    /// Done
    ///
    /// Button that closes the list of Spaces without changing which is shown.
    static let spacesDone = Word(key: "spaces.done")

    /// New Space…
    ///
    /// Button at the end of the list of Spaces: opens the page that makes one.
    static let spacesNew = Word(key: "spaces.new")

    /// Edit…
    ///
    /// Button in the list of Spaces, beside 'New Space…': opens the page that gives the Space the window shows another name or colour.
    static let spacesRename = Word(key: "spaces.rename")

    /// Shares logins with %@
    ///
    /// Under a Space's name in the list of Spaces, when it is signed in to sites as another Space is. %@ is that other Space's name.
    static let spacesShares = Word(key: "spaces.shares")

    /// Shown
    ///
    /// Said of the Space the window shows now, in the list of Spaces.
    static let spacesShown = Word(key: "spaces.shown")

    /// Spaces
    ///
    /// Heading of the list of Spaces.
    static let spacesTitle = Word(key: "spaces.title")

    /// Command
    ///
    /// At the end of a row of that list which is a command of the window's, found by its words: choosing the row does it.
    static let suggestionsCommand = Word(key: "suggestions.command")

    /// Suggestions
    ///
    /// Accessibility label of the list of places offered under the address field as it is typed in.
    static let suggestionsList = Word(key: "suggestions.list")

    /// Space
    ///
    /// At the end of a row of that list which is a Space, found by its name: choosing the row shows that Space in the window.
    static let suggestionsSpace = Word(key: "suggestions.space")

    /// Switch to Tab
    ///
    /// At the end of a row of that list which is a tab that is already open: choosing the row brings that tab in front instead of loading the page again.
    static let suggestionsSwitchTab = Word(key: "suggestions.switch_tab")

    /// Done
    ///
    /// Button that closes the switcher without changing which tab is in front.
    static let switcherDone = Word(key: "switcher.done")

    /// Tabs: %d
    ///
    /// Accessibility label of a button in the toolbar of a narrow window (a phone's): it opens the switcher, a sheet with every tab of the row. %d is how many tabs the row has; the button itself shows that number alone.
    static let switcherOpen = Word(key: "switcher.open")

    /// Tabs
    ///
    /// Title at the head of the switcher, the sheet on a phone that lists every tab of the row.
    static let switcherTitle = Word(key: "switcher.title")

    /// Type an address or a search above.
    ///
    /// Hint under the heading of an empty tab.
    static let tabEmptyHint = Word(key: "tab.empty.hint")

    /// Where to?
    ///
    /// Heading of a tab that has no page yet.
    static let tabEmptyTitle = Word(key: "tab.empty.title")

    /// Close Tab
    ///
    /// Button on a tab in the strip, and command (Command-W on a Mac): close the tab.
    static let tabsClose = Word(key: "tabs.close")

    /// Tabs
    ///
    /// Accessibility label of the strip of the window's tabs.
    static let tabsList = Word(key: "tabs.list")

    /// Move Tab Down
    ///
    /// The same command with the tabs down the side of the window: the tab changes places with the one below it.
    static let tabsMoveDown = Word(key: "tabs.move.down")

    /// Move Tab Left
    ///
    /// Command for a tab drawn in a row along the top, or a Favourite's tile (in the Mac's Window menu, for the tab in front; in a tab's own menu on a touch screen): the tab changes places with the one on its left. Grey when no tab of its kind is on that side.
    static let tabsMoveLeft = Word(key: "tabs.move.left")

    /// Move Tab Right
    ///
    /// The same command the other way: the tab changes places with the one on its right.
    static let tabsMoveRight = Word(key: "tabs.move.right")

    /// Move Tab Up
    ///
    /// The same command with the tabs down the side of the window: the tab changes places with the one above it.
    static let tabsMoveUp = Word(key: "tabs.move.up")

    /// New Tab
    ///
    /// Button at the end of the strip of tabs, and command (Command-T on a Mac): open an empty tab.
    static let tabsNew = Word(key: "tabs.new")

    /// Show Next Tab
    ///
    /// Command in the Mac's Window menu (Control-Tab): bring the next tab in the row in front; from the last, the first.
    static let tabsNext = Word(key: "tabs.next")

    /// Show Previous Tab
    ///
    /// Command in the Mac's Window menu (Control-Shift-Tab): bring the tab before in the row in front; from the first, the last.
    static let tabsPrevious = Word(key: "tabs.previous")

    /// Reopen Closed Tab
    ///
    /// Command (Shift-Command-T on a Mac): bring back the tab that was last closed, where it was.
    static let tabsReopen = Word(key: "tabs.reopen")

    /// New Tab
    ///
    /// The name the strip gives a tab that has no page yet.
    static let tabsUntitled = Word(key: "tabs.untitled")

    /// Back
    ///
    /// Button: go back one page.
    static let toolbarBack = Word(key: "toolbar.back")

    /// Forward
    ///
    /// Button: go forward one page.
    static let toolbarForward = Word(key: "toolbar.forward")

    /// Open Location
    ///
    /// Command (Command-L on a Mac): put the keyboard in the address field with the whole address selected.
    static let toolbarOpenLocation = Word(key: "toolbar.open_location")

    /// Reload
    ///
    /// Button: reload the page.
    static let toolbarReload = Word(key: "toolbar.reload")

    /// Stop
    ///
    /// Button: stop loading the page.
    static let toolbarStop = Word(key: "toolbar.stop")

    /// Undo
    ///
    /// Command (Command-Z on a Mac, in the Edit menu; at the top of a tab's or a Group's own menu on a touch screen): undo the newest thing done to arrange the row of tabs, when the app has no words for what that was; and on a Mac the same item when it undoes typing in a field.
    static let undoArrangement = Word(key: "undo.arrangement")

    /// Undo Add to Favourites
    ///
    /// The same command when what it undoes is a tab having been made a Favourite.
    static let undoFavouriteAdded = Word(key: "undo.favourite.added")

    /// Undo Remove from Favourites
    ///
    /// The same command when what it undoes is a Favourite having been made an ordinary Site again.
    static let undoFavouriteRemoved = Word(key: "undo.favourite.removed")

    /// Undo Group Colour
    ///
    /// The same command when what it undoes is a Group having been given another colour.
    static let undoGroupColoured = Word(key: "undo.group.coloured")

    /// Undo New Group
    ///
    /// The same command when what it undoes is a Group having been made.
    static let undoGroupMade = Word(key: "undo.group.made")

    /// Undo Move to Group
    ///
    /// The same command when what it undoes is a tab having been put in a Group, or taken out of one.
    static let undoGroupMoved = Word(key: "undo.group.moved")

    /// Undo Remove Group
    ///
    /// The same command when what it undoes is a Group having been removed.
    static let undoGroupRemoved = Word(key: "undo.group.removed")

    /// Undo Rename Group
    ///
    /// The same command when what it undoes is a Group having been given another name.
    static let undoGroupRenamed = Word(key: "undo.group.renamed")

    /// Undo Move Group
    ///
    /// The same command when what it undoes is a Group having been moved among the Groups.
    static let undoGroupReordered = Word(key: "undo.group.reordered")

    /// Undo Keep as Site
    ///
    /// The same command when what it undoes is a tab having been kept as a Site.
    static let undoSiteKept = Word(key: "undo.site.kept")

    /// Undo Stop Keeping
    ///
    /// The same command when what it undoes is a Site having been let go.
    static let undoSiteLetGo = Word(key: "undo.site.let_go")

    /// Undo Move Tab
    ///
    /// The same command when what it undoes is a tab having been moved to another place in its row.
    static let undoTabMoved = Word(key: "undo.tab.moved")

    /// Done
    ///
    /// Button that closes the zoom bar.
    static let zoomDone = Word(key: "zoom.done")

    /// Zoom In
    ///
    /// Button in the zoom bar, and the command (Command-plus on a Mac): make the page bigger.
    static let zoomIn = Word(key: "zoom.in")

    /// %ld%%
    ///
    /// The zoom bar's level, in per cent. %ld is the number; %% is a per cent sign.
    static let zoomLevel = Word(key: "zoom.level")

    /// Zoom
    ///
    /// Label of the button, beside the tabs, that opens the zoom bar.
    static let zoomOpen = Word(key: "zoom.open")

    /// Zoom Out
    ///
    /// Button in the zoom bar, and the command (Command-minus on a Mac): make the page smaller.
    static let zoomOut = Word(key: "zoom.out")

    /// Actual Size
    ///
    /// Accessibility label of the zoom bar's level, which puts the page back to 100% when pressed; and the command (Command-0 on a Mac).
    static let zoomReset = Word(key: "zoom.reset")
}

extension Word {
    /// Every key the catalogue has: `Word(named:)` answers for these, and for no other.
    static let keys: Set<String> = [
        "address.insecure",
        "address.label",
        "address.placeholder",
        "address.secure",
        "app.name",
        "archive.clear",
        "archive.clear.cancel",
        "archive.clear.detail",
        "archive.clear.everything",
        "archive.clear.title",
        "archive.done",
        "archive.empty",
        "archive.empty.detail",
        "archive.forget",
        "archive.older",
        "archive.tidied",
        "archive.title",
        "awake.asleep",
        "awake.done",
        "awake.empty",
        "awake.private",
        "awake.sleep_others",
        "awake.title",
        "awake.why.capture",
        "awake.why.download",
        "awake.why.in_sight",
        "awake.why.loading",
        "awake.why.question",
        "awake.why.sound",
        "awake.why.typed",
        "awake.why.waiting",
        "catalogue.colours",
        "catalogue.radius",
        "catalogue.size",
        "catalogue.space",
        "catalogue.start_failed",
        "catalogue.state.archive_empty",
        "catalogue.state.archive_list",
        "catalogue.state.awake_empty",
        "catalogue.state.awake_list",
        "catalogue.state.doing",
        "catalogue.state.downloads_empty",
        "catalogue.state.downloads_list",
        "catalogue.state.empty",
        "catalogue.state.failed",
        "catalogue.state.failed_certificate",
        "catalogue.state.failed_certificate_warning",
        "catalogue.state.favourites",
        "catalogue.state.find",
        "catalogue.state.find_none",
        "catalogue.state.group_edit",
        "catalogue.state.group_new",
        "catalogue.state.groups",
        "catalogue.state.history",
        "catalogue.state.history_empty",
        "catalogue.state.history_list",
        "catalogue.state.history_none_found",
        "catalogue.state.insecure",
        "catalogue.state.link_menu",
        "catalogue.state.loading",
        "catalogue.state.long",
        "catalogue.state.permissions_empty",
        "catalogue.state.permissions_list",
        "catalogue.state.picture_link_menu",
        "catalogue.state.picture_menu",
        "catalogue.state.prompt",
        "catalogue.state.prompt_alert",
        "catalogue.state.prompt_confirm",
        "catalogue.state.prompt_download",
        "catalogue.state.prompt_sign_in",
        "catalogue.state.prompt_sign_in_again",
        "catalogue.state.prompt_text",
        "catalogue.state.search_change",
        "catalogue.state.search_question",
        "catalogue.state.secure",
        "catalogue.state.sidebar",
        "catalogue.state.sites",
        "catalogue.state.spaces",
        "catalogue.state.spaces_new",
        "catalogue.state.spaces_new_shared",
        "catalogue.state.spaces_refused",
        "catalogue.state.spaces_rename",
        "catalogue.state.suggestions",
        "catalogue.state.switcher",
        "catalogue.state.tabs",
        "catalogue.state.waking",
        "catalogue.state.zoom",
        "catalogue.states",
        "catalogue.title",
        "certificate.advanced",
        "certificate.proceed",
        "certificate.warning",
        "certificate.warning.only_if",
        "certificate.warning.title",
        "downloads.active",
        "downloads.clear",
        "downloads.coming",
        "downloads.coming_so_far",
        "downloads.done",
        "downloads.empty",
        "downloads.forget",
        "downloads.older",
        "downloads.open",
        "downloads.open_failed",
        "downloads.stop",
        "downloads.title",
        "edit.redo",
        "error.download.cancelled",
        "error.download.disk",
        "error.download.failed",
        "error.download.interrupted",
        "error.download.missing",
        "error.download.network",
        "error.engine.no_private",
        "error.engine.no_profiles",
        "error.engine.unavailable",
        "error.group.change_failed",
        "error.group.colour_unknown",
        "error.group.failed",
        "error.group.name_empty",
        "error.group.too_many",
        "error.group.unknown",
        "error.net.connect",
        "error.net.dns",
        "error.net.failed",
        "error.net.offline",
        "error.net.timeout",
        "error.net.tls",
        "error.net.tls.date",
        "error.net.tls.name",
        "error.net.tls.untrusted",
        "error.page.renderer_gone",
        "error.retry",
        "error.space.colour_unknown",
        "error.space.failed",
        "error.space.icon_unknown",
        "error.space.name_empty",
        "error.space.rename_failed",
        "error.space.too_many",
        "error.space.unknown",
        "error.start.detail",
        "error.start.title",
        "error.system.no_handler",
        "error.tidy.time_unknown",
        "error.title",
        "favourites.add",
        "favourites.favourite",
        "favourites.remove",
        "find.count",
        "find.done",
        "find.field",
        "find.next",
        "find.none",
        "find.open",
        "find.previous",
        "group.cancel",
        "group.edit",
        "group.edit.title",
        "group.make",
        "group.name",
        "group.new.title",
        "group.remove",
        "group.said.open",
        "group.said.shut",
        "group.save",
        "group.shut_label",
        "groups.move",
        "groups.new",
        "groups.none",
        "history.clear",
        "history.clear.cancel",
        "history.clear.detail",
        "history.clear.everything",
        "history.clear.hour",
        "history.clear.title",
        "history.clear.today",
        "history.done",
        "history.empty",
        "history.forget",
        "history.none_found",
        "history.older",
        "history.open",
        "history.search",
        "history.title",
        "layout.side",
        "layout.top",
        "link.cancel",
        "link.copy",
        "link.download",
        "link.download_image",
        "link.image_menu",
        "link.menu",
        "link.open_image_new_tab",
        "link.open_link_new_tab",
        "link.open_new_tab",
        "menu.close_window",
        "menu.history",
        "outside.search",
        "page.picture",
        "permissions.clear",
        "permissions.done",
        "permissions.empty",
        "permissions.forget",
        "permissions.kind.camera",
        "permissions.kind.downloads",
        "permissions.kind.location",
        "permissions.kind.microphone",
        "permissions.title",
        "private.back",
        "private.chip",
        "private.close",
        "private.hides",
        "private.keeps",
        "private.new",
        "private.row.detail",
        "private.row.open",
        "private.window",
        "prompt.allow",
        "prompt.answer",
        "prompt.cancel",
        "prompt.download",
        "prompt.kind.camera",
        "prompt.kind.location",
        "prompt.kind.microphone",
        "prompt.ok",
        "prompt.permission",
        "prompt.refuse",
        "prompt.says",
        "prompt.sign_in",
        "prompt.sign_in.again",
        "prompt.sign_in.go",
        "prompt.sign_in.name",
        "prompt.sign_in.not_private",
        "prompt.sign_in.password",
        "prompt.silence",
        "search.change",
        "search.default",
        "search.goes_to",
        "search.in_use",
        "search.keep",
        "search.keep_default",
        "search.question.detail",
        "search.question.title",
        "selection.search_with",
        "sites.asleep",
        "sites.away",
        "sites.keep",
        "sites.let_go",
        "sites.return",
        "sites.return_site",
        "sites.site",
        "sites.sleep",
        "space.automatic_colour",
        "space.chip",
        "space.chosen_colour",
        "space.colour",
        "space.colour.blue",
        "space.colour.green",
        "space.colour.grey",
        "space.colour.orange",
        "space.colour.pink",
        "space.colour.purple",
        "space.colour.red",
        "space.colour.teal",
        "space.colour.yellow",
        "space.home",
        "space.icon",
        "space.new.back",
        "space.new.logins",
        "space.new.make",
        "space.new.name",
        "space.new.own",
        "space.new.own.detail",
        "space.new.share_with",
        "space.new.shared",
        "space.new.shared.detail",
        "space.new.title",
        "space.no_colour",
        "space.no_icon",
        "space.rename.save",
        "space.rename.title",
        "space.tidy",
        "space.tidy.0",
        "space.tidy.2592000",
        "space.tidy.43200",
        "space.tidy.604800",
        "space.tidy.86400",
        "space.tidy.detail",
        "spaces.detail",
        "spaces.done",
        "spaces.new",
        "spaces.rename",
        "spaces.shares",
        "spaces.shown",
        "spaces.title",
        "suggestions.command",
        "suggestions.list",
        "suggestions.space",
        "suggestions.switch_tab",
        "switcher.done",
        "switcher.open",
        "switcher.title",
        "tab.empty.hint",
        "tab.empty.title",
        "tabs.close",
        "tabs.list",
        "tabs.move.down",
        "tabs.move.left",
        "tabs.move.right",
        "tabs.move.up",
        "tabs.new",
        "tabs.next",
        "tabs.previous",
        "tabs.reopen",
        "tabs.untitled",
        "toolbar.back",
        "toolbar.forward",
        "toolbar.open_location",
        "toolbar.reload",
        "toolbar.stop",
        "undo.arrangement",
        "undo.favourite.added",
        "undo.favourite.removed",
        "undo.group.coloured",
        "undo.group.made",
        "undo.group.moved",
        "undo.group.removed",
        "undo.group.renamed",
        "undo.group.reordered",
        "undo.site.kept",
        "undo.site.let_go",
        "undo.tab.moved",
        "zoom.done",
        "zoom.in",
        "zoom.level",
        "zoom.open",
        "zoom.out",
        "zoom.reset",
    ]
}
