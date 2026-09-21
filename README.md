<div align="center">
  <img alt="boring commit logo" src="assets/logo.svg" width="80%" height="80%">
</div>

---

**Boring Commit** uses LLMs to generate concise Git commit messages from your staged changes.

It can work with local models or API-based providers, so you can choose between keeping everything on your machine or using a hosted model.

Written in Rust.

## Install

Prebuilt binaries for macOS (Apple Silicon), Linux (x64), and Windows (x64).
Releases, checksums, and direct downloads live under
[GitHub Releases](https://github.com/iamprasadraju/Boring-Commit/releases).

macOS / Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/iamprasadraju/Boring-Commit/main/scripts/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/iamprasadraju/Boring-Commit/main/scripts/install.ps1 | iex
```

Or [download directly](https://github.com/iamprasadraju/Boring-Commit/releases/latest)
(`bcommit-macos-aarch64.tar.gz`, `bcommit-linux-x86_64.tar.gz`,
`bcommit-windows-x86_64.zip`). Requires Windows 10 or later.
Then verify with `bcommit --version` and run `bcommit config`.

Alternative (any platform with Rust): `cargo install bcommit`.

https://doc.rust-lang.org/beta/std/process/struct.Command.html
https://rust-cli.github.io/book/tutorial/cli-args.htmľ

https://ollama.com/download
