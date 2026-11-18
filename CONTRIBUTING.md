# Contributing to WebTiger

Thank you for wanting to help. WebTiger is early: the design is still moving, and the fastest way to waste your time is to build something the plan is about to change. Please read this first.

## Before you write code

1. **Read the plan.** [`docs/features/000-master-plan.md`](docs/features/000-master-plan.md) says what WebTiger is and is not. Each feature has its own document in `docs/features/`, written before its code.
2. **Open an issue** for anything larger than a small fix, and say what you intend. A feature without a plan document will be asked for one.
3. **Check the scope.** Some things are deliberately out for 1.0 (000 §2.2).

## The rules of the code

- **The core decides, the shell draws, the engine renders.** Browser logic lives in the Rust core, once. A shell that needs to decide something is missing a request in the contract.
- **The core speaks codes; the shells speak words.** No user-visible text crosses the contract. Strings live in `Localization/strings.json`.
- **Rust:** no `unwrap`, `expect` or `panic` outside tests. `unsafe` only in `wt-ffi`, each block with a `SAFETY` comment and a test.
- **No private platform APIs**, in any shell.
- **No addresses, titles or typed text in logs or errors.** Ever.
- **Tests come with the change.** A change to an engine adapter runs the adapter suite. A change to a screen has a UI or snapshot test.
- **Generated files are checked in.** After changing the contract, the design tokens or the strings, run the generator; `Tools/test.sh lint` fails when they are stale.
- **Dependencies are a decision.** A new one is discussed in the issue first. Rust dependencies must pass `Core/deny.toml`.
- British spelling in documents and comments.

## Checks

```sh
Tools/test.sh all
```

A pull request must pass it. See the README for the individual steps.

## Commits

[Conventional commits](https://www.conventionalcommits.org/): `feat(guard): …`, `fix(tabs): …`, `test(adapter): …`, `docs: …`.

## Developer Certificate of Origin

WebTiger does not ask you to sign a contributor licence agreement. Instead, every commit must be signed off, which certifies the [Developer Certificate of Origin](https://developercertificate.org/): in short, that you wrote the change or have the right to submit it under the project's licence.

```sh
git commit -s
```

adds the line `Signed-off-by: Your Name <you@example.com>`. Commits without it cannot be merged.

Your contribution is licensed under the [Mozilla Public License 2.0](LICENSE), like the rest of the project.

## Security problems

Not here, please. See [SECURITY.md](SECURITY.md).
