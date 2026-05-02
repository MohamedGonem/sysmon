# Contributing to SysMon

First off, thanks for taking the time to contribute! ❤️

All types of contributions are encouraged and valued. Please read the relevant section before contributing — it makes life easier for everyone involved. 🎉

> Don't have time to contribute? No worries! Other ways to support the project:
> - Star the repo
> - Tell friends/colleagues about it
> - Mention it at meetups

---

## Table of Contents

- [I Have a Question](#i-have-a-question)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Enhancements](#suggesting-enhancements)
- [Your First Code Contribution](#your-first-code-contribution)
- [Improving the Documentation](#improving-the-documentation)
- [Commit Message Style](#commit-message-style)
- [Join the Project Team](#join-the-project-team)

---

## I Have a Question

Before opening an issue, please:

1. Search existing [Issues](https://github.com/MohamedGonem/sysmon/issues) — your question may already be answered.
2. Search the internet / Stack Overflow.

If you still need help, open an [Issue](https://github.com/MohamedGonem/sysmon/issues/new) and include:
- What you were trying to do
- What you expected vs what actually happened
- Your Rust toolchain version (`rustc --version`, `cargo --version`) and OS

---

## Reporting Bugs

> **Security bugs:** Do NOT open a public issue. Email us at `<sysmon.dev@proton.me>` instead.

### Before submitting

- Confirm you're on the latest version.
- Check the [bug tracker](https://github.com/MohamedGonem/sysmon/issues?q=label%3Abug) for duplicates.
- Collect:
  - Panic output / stack trace
  - OS and architecture (e.g. Linux x86_64, macOS ARM)
  - Rust toolchain version (`rustc --version`, `cargo --version`)
  - Steps to reliably reproduce the issue

### Submitting the report

Open an [Issue](https://github.com/MohamedGonem/sysmon/issues/new) and include:

- A clear, descriptive title
- Expected vs actual behavior
- Reproduction steps (minimal example preferred)
- All info collected above

Once filed, the team will label it and attempt to reproduce it. Issues that can't be reproduced will be tagged `needs-repro` and paused until they are.

---

## Suggesting Enhancements

Before submitting, please:

- Check you're on the latest version.
- Search [existing issues](https://github.com/MohamedGonem/sysmon/issues) to avoid duplicates — if one exists, comment on it instead.
- Consider whether the feature fits SysMon's scope and would benefit most users.

When submitting, open an Issue and include:

- A clear, descriptive title
- Step-by-step description of the proposed behavior
- Why this would be useful to most SysMon users
- Any existing tools or projects that implement something similar

---

## Your First Code Contribution

### Setup

SysMon is built in Rust. Getting started is straightforward:

```bash
git clone https://github.com/MohamedGonem/sysmon.git
cd sysmon
cargo build
cargo run
```

No extra dependencies required beyond a standard Rust toolchain. If you don't have Rust installed, get it at [rustup.rs](https://rustup.rs).

### Finding something to work on

Look for issues tagged:
- `good first issue` — beginner-friendly tasks
- `help wanted` — contributions actively needed
- `todo` — placeholder functions waiting to be implemented

### Submitting a pull request

1. Fork the repo and create a branch: `git checkout -b feat/your-feature`
2. Make your changes
3. Make sure everything compiles and tests pass: `cargo test`
4. Make sure there are no warnings: `cargo clippy`
5. Format your code: `cargo fmt`
6. Open a pull request with a clear description of what you changed and why

---

## Improving the Documentation

Good documentation is just as valuable as good code. You can help by:

- Fixing typos or unclear wording in any `.md` file
- Adding or improving doc comments (`///`) on public functions
- Improving the README
- Adding examples

Same process as code contributions — fork, edit, and open a pull request.

---

## Commit Message Style

We use **Conventional Commits**. This makes the history readable and enables automated changelogs.

### Format

```
<type>: <short summary>

[optional body]
[optional footer]
```

### Types

| Type | When to use |
|------|-------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation changes only |
| `refactor` | Code change that isn't a fix or feature |
| `test` | Adding or updating tests |
| `chore` | Build process, tooling, dependencies |

### Examples

```
feat: add CPU usage monitor
fix: prevent panic on missing /proc/stat
docs: add setup instructions to README
refactor: extract metric parsing into separate module
```

**Tips:**
- Use the imperative mood: "add feature" not "added feature"
- Keep the summary under 72 characters
- Reference issues in the footer: `Closes #42`

---

## Join the Project Team

Interested in becoming a maintainer? Contribute consistently, engage respectfully with the community, and reach out by opening an issue or sending an email to `sysmon.dev@proton.me`. We're happy to welcome active contributors to the team.

---

## Attribution

This guide is based on [contributing.md](https://contributing.md/generator).