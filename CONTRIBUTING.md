# Contributing to Speak

## Commits

Please use descriptive PR titles. We loosely follow the [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) style, but this is not a requirement to follow exactly. PRs will be addressed more quickly if it is clear what the intention is.

## Picking up work

Planned work is listed in the [roadmap](docs/roadmap.md) as `SPK-` tickets rather than GitHub issues. Start the PR title and branch name with the ticket ID, e.g. `SPK-103: lexical scoping and real closures`, and check the ticket's "Needs" column first.

Two rules apply to every change:

- No hard-coded English names. Keywords, builtins, std names, type names and error messages go through the locale files in `core/locales`.
- Keep the core building for `wasm32`. Features that need the OS, such as raw sockets or processes, go behind feature flags so the browser and mobile builds keep working.

## Helping without writing Rust

You don't need to know Rust to help Speak:

- **Translate:** add or improve a language by editing the YAML files in `core/locales`.
- **Teach:** write or test lessons and sample programs in your language.
- **Test in classrooms:** report what confuses learners, especially error messages.

Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Before opening a PR

Ensure the following are satisfied before opening a PR:

- [ ] The `git-hooks.sh` script has been run to install the git hooks.
- [ ] Ensure any new functionality is adequately tested.
- [ ] If any new public types or functions are added, ensure they have appropriate [rustdoc](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html) documentation
