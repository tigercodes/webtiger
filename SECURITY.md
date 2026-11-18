# Security policy

A browser is a security product. Reports are welcome, and are read.

## Reporting a vulnerability

**Please do not open a public issue for a security problem.**

Report it privately through GitHub: on this repository's **Security** tab, choose **Report a vulnerability**. That opens a private advisory only the maintainers can see.

Please include:

- what the problem is and what an attacker could do with it;
- the platform, the app version, and the web engine and its version;
- steps or a page that reproduce it, if you have them.

You will get an acknowledgement, and then a decision on whether it is accepted, with a plan for the fix. Please give us a reasonable time to ship a fix before you publish.

## What counts

Especially:

- the address field showing an address other than the page's (spoofing);
- a prompt that names the wrong site;
- data crossing between Spaces that have separate logins;
- a plugin doing something its granted permissions do not cover;
- anything that makes the browser load or run what the user did not ask for.

## What is out of scope

- Vulnerabilities in the web engines themselves (WebKit, Chromium, WebView2, Android System WebView). Please report those to their vendors; WebTiger ships each system's engine and picks up its fixes with system updates.
- Problems that need a device already under the attacker's control.

## Supported versions

WebTiger is in early development. Until a first release, only the `main` branch is supported.
