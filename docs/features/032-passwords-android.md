# 032 — Passwords on Android: the system's Autofill and a page's sign-in form (W1)

| | |
|---|---|
| **Status** | Built and tested on an emulator. Two things in WebTiger were in the way, and are put right: the window's own view kept the system from reaching the web view, so no password manager was ever asked about a page; and, since [023](023-search-selection.md), a web view was no longer lent the window's activity, so a page's own list of choices never opened. By hand, with a service of the tests' own, a name and a password were offered by the system and put in a page's form. Not run on a phone, nor with a real password manager (§7) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "platform password autofill and passkeys" (WT-D13: the platform's service, no vault of our own). The Android half. Apple's needs the browser entitlement (000 §13), and waits for it |
| **Contract** | No change |
| **Builds on** | [003](003-android-shell.md): the System WebView adapter, and the window that shows its web views |
| **Last updated** | 2026-11-21 |

## 1. User story

I keep my passwords in a password manager: the one I chose in Android's settings. I open a site's sign-in page in WebTiger and put the cursor in its field. My password manager is asked about this form, of this site, and offers me what it has; I pick one, and the name and the password are in the form. WebTiger keeps no passwords, and is not the one that offers them.

## 2. What is in it, and what is not

**In:** on Android, a page's sign-in form in WebTiger's window reaches the system's Autofill framework: the service the person chose is asked when they put the cursor in a field, and is told which site the form is of and which fields are a name and a password. That it is so is tested, on a device, and what in WebTiger was in its way is put right.

**Not in:**

- **Apple's systems.** A browser's forms reach the system's passwords there only with Apple's browser entitlement.
- **Passkeys** (the system's Credential Manager, for a page that asks for one). Its own feature: a browser must be let stand for any site, which the system grants to browsers it knows.
- **What a password manager offers, saves or asks.** Its own, and the system's to show. WebTiger draws none of it.
- **A vault of WebTiger's own**, ever (WT-D13).
- **Private windows** (W2), where nothing should be offered to be saved.
- **Other things a service fills** (an address, a card): the same road, not looked at here.

## 3. The rule

**WebTiger is not in it.** The WebView tells the system that a form's field has the cursor, with what the form is (its fields, and what each says it is for) and whose it is (the site of the page, or of the frame, that the form is in). The system asks the person's service, shows what it offers, and hands what the person picks to the WebView, which puts it in the form. The core is not asked, the contract says nothing of it, and the shell neither reads nor writes what is filled.

**What must hold, and is the WebView's to keep:**

- **The service is told the form's own site**, by the WebView and not by the page: a password manager finds what to offer by it. A page cannot name another site as its own.
- **Nothing is filled until the person picks.** A page gets nothing by having a form.
- **The service is told of the form the cursor is in**, not of other tabs or other Spaces.

**A Space's logins and a password manager.** A password manager is the person's, not a Space's: it offers the same in every Space. What a Space keeps apart is what sites remember (cookies and the rest, [003](003-android-shell.md) §5.2), not what the person knows.

## 4. What was in the way

Found by running it, with a service of the tests' own in the place of a password manager, and the system's own log of what it did.

**The window's own view hid the web view from the system.** The system finds what a service can fill by asking the window's views to describe themselves, each asking the views in it. The view Compose draws the window in describes the window's text fields as parts of its own; and a view that has described parts of its own is not asked about the views inside it. So with the address field described, the web view under it was never asked: the WebView told the system that a form's field had the cursor, the system asked the service, and what the service was shown was the address field and nothing of the page. No password manager could have offered anything for any page.

- **Put right in the app** (`ui/FillableWindow.kt`): the window now has a view of the app's own around what Compose draws in. Asked to describe itself, it also has the web view that is on the screen describe itself. The service is then shown the form, with its site and its fields.
- **It is the Compose this app is built with** (Compose UI 1.11). The next (1.12) asks the views inside it itself, as its own code shows; the app is not built with it because it needs the next level of Android's SDK (`Android/gradle/libs.versions.toml`). Built with 1.12, the web view would be described twice: the app's test says how many times the form's fields are described, and fails then, which is when the app's own view goes.

**A web view was no longer lent the window's activity** (a fault of 023's making, found here). A web view outlives the window that shows it, so its context is one that is lent the window's activity while the web view is on the screen (003). 023 wrapped that context in one of the adapter's own, which hears the WebView ask for a search of selected words; the window looked for what to lend the activity to only at the top, did not find it under the wrapping, and lent nothing. What the WebView puts over a page in a window of its own needs that activity: **a page's list of choices (a `select`) did not open at all** in what was built since 023. Now the window finds what to lend to however it is wrapped.

**What was not in the way**, though it looked as if it was: where a web view is made. Android's advice is that a WebView be made on an activity for Autofill to work, and the adapter makes its web views on the application. Tried both ways, each in a run of its own: in the app, the service was asked just the same. The web views are still made on the application.

## 5. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | No contract, and nothing in the core | Nothing about browsing is decided: the system, its service and the WebView do it between them, for the form the person put the cursor in |
| D2 | A view of the app's own around the window, until the app is built with a Compose that does it | The other course was to wait for the next SDK level. Meanwhile no password manager works at all, which is a browser nobody can use for a day |
| D3 | A password manager offers the same in every Space | It is the person's. A Space keeps apart what sites remember |
| D4 | The tests' own service, in the tests' package, in Java | It must be something the system can start by itself; the tests' package has no Kotlin of its own to run it with. It offers nothing unless a form asks by a name only a check by hand uses, so that no test brings up the system's list |

## 6. Tests

| Where | What |
|---|---|
| The test site | A sign-in form (a field that says it is for a name, one for a password; once something is in it the page's title says the name and how long the password is), and a page with a list of choices that says in its title what was chosen |
| The Android app, on the emulator (`AppAutofillTest`, 1) | With a service of the tests' own set as the device's Autofill service, and the device's own put back afterwards: the app in fake mode, the sign-in form's address typed in the address field (for which the service may be asked too: that question has no form in it, and no site); a finger in the name's field; the service is asked about the form, once, and is told the site (`127.0.0.1`, over `http`) and the two fields, each once: a name, with the cursor in it, and a password, each by the page's own word for it. The service offers nothing, so nothing of the system's is shown. Passed five runs one after another, and in the whole suite twice |
| The Android app, on the emulator (`AppChoicesTest`, 1) | A finger on a page's list of choices: its choices come up over the page, in a window of the WebView's own; one is chosen, the list goes, and the page says which. Passed three runs one after another |
| The window (`WebViewHostTest`, 3, on the JVM) | What is lent an activity is found inside whatever it is wrapped in: by itself, inside the adapter's own context, and deeper; a view whose context can be lent nothing has none; lent an activity, what the view asks its context for comes from it, and then from the application again |
| By breaking it | Each remedy taken out, each of the two app tests then run on the emulator in a process of its own. (The app's core starts once in a process, with the first window's start address: the two tests at first asked for start addresses of their own, and in the whole suite every app test after them opened on the wrong page. They now start where every app test starts, and type their page's address.) Without the window's own view: the service is shown no form; the list of choices still opens. Not lent the activity: the list of choices never opens; the service is still asked. With neither, as the app was: both fail. Looking for what to lend to only at the top: the JVM's test fails |
| By hand (2026-11-21, the emulator, the app in fake mode, the test site served from the Mac, the tests' own service set as the device's and offering a name and a password of its own to this one form) | A finger in the name's field: the system's own list came up over the page, at the field, with the one thing offered. Picked: the page's title said the name and that the password had eight letters, which is what was offered; the field showed the name. The device's own service was then put back, the tests' package removed, and the app's data cleared |

## 7. What is not verified

- **A real password manager**, and one with something saved for a real site. The service was the tests' own. What a real one makes of what it is told (the site without its port, the fields by the page's own words) was not seen.
- **Saving.** A service may ask to save what was typed when a form is sent; that is its own, with the system's own question, and nothing here looked at it.
- **A form in a frame of another site**, which a password manager must be told is the frame's and not the page's. The WebView's to say; no test has such a frame.
- **Passkeys**, and anything but a name and a password.
- **A phone**, another WebView (133 here), another version of Android (16 here).
- **What else needed the activity that was not lent** between 023 and this. A list of choices is what was found and is tested. A date to pick, a colour, and whatever else a WebView shows in a window of its own were not tried, then or now.
- **Where a web view is made.** On the application, against Android's advice for Autofill; asked and filled all the same on this emulator. Whether every WebView is so forgiving is not known.

## 8. Done checklist

- [x] The test site's sign-in form, and its list of choices.
- [x] On the emulator: the system's Autofill service is asked about the form in WebTiger's window, with its site and its fields.
- [x] What was in the way, put right: the window's own view; the activity lent again.
- [x] By hand: offered by the system and filled.
- [x] `Tools/test.sh all` green, and `android-device`, on 2026-11-21: `lint`, `core` (505), `unit` (265 on the Mac), `build`, the Mac's UI suite (29), `ios` (247 on the iPhone simulator, one skipped as before), the iPhone's UI suite (26), `android` (351 and 32 on the JVM) and `android-device` (147 on the emulator).
