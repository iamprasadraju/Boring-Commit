<div align="center">
  <img alt="boring commit logo" src="assets/logo.svg" width="80%" height="80%">

  **Stop writing boring commit messages.**

  [![build](https://github.com/iamprasadraju/Boring-Commit/actions/workflows/build.yml/badge.svg)](https://github.com/iamprasadraju/Boring-Commit/actions)
  [![release](https://img.shields.io/github/v/release/iamprasadraju/Boring-Commit)](https://github.com/iamprasadraju/Boring-Commit/releases)
  [![license](https://img.shields.io/github/license/iamprasadraju/Boring-Commit)](LICENSE)
</div>

**Boring Commit** uses LLMs to generate concise conventional Git commit messages
from your staged changes. It works with local models or API-based providers, so
you can keep everything on your machine or use a hosted model. Written in Rust.

```sh
$ git add -p
$ bcommit
⠋ Generating with ollama / qwen2.5-coder:3b ...
feat(auth): add offline fallback for commit generation
? What to do? Commit / Edit / Regenerate / Cancel
✔ Committed
```

## Install

Prebuilt binaries for macOS (Apple Silicon), Linux (x64), and Windows (x64).
Releases, checksums, and direct downloads live under
[GitHub Releases](https://github.com/iamprasadraju/Boring-Commit/releases).

macOS / Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/iamprasadraju/Boring-Commit/main/scripts/install.sh | sh
```

Windows (PowerShell, requires Windows 10 or later):

```powershell
irm https://raw.githubusercontent.com/iamprasadraju/Boring-Commit/main/scripts/install.ps1 | iex
```

Or [download directly](https://github.com/iamprasadraju/Boring-Commit/releases/latest)
(`bcommit-macos-aarch64.tar.gz`, `bcommit-linux-x86_64.tar.gz`,
`bcommit-windows-x86_64.zip`) and verify against the published `.sha256` files.
Then confirm with `bcommit --version` and run `bcommit config`.

Alternative (any platform with Rust): `cargo install bcommit`.

## Quickstart

```sh
# 1. Get a model (local, recommended)
ollama serve
ollama pull qwen2.5-coder:3b
# ...or point bcommit at a hosted provider instead (see below)

# 2. Configure provider and model
bcommit config

# 3. Daily use
git add -p
bcommit
```

Ollama itself installs from [ollama.com/download](https://ollama.com/download).
`bcommit` can also offer to install Ollama for you during `bcommit config`.

## Providers

One active provider + model pair at a time. Switch with `bcommit model`.

| Provider     | Endpoint                                        | Notes                        |
| ------------ | ----------------------------------------------- | ---------------------------- |
| `ollama`     | `http://localhost:11434`                        | Local, no API key needed     |
| `openai`     | `https://api.openai.com/v1`                     | Hosted, API key required     |
| `groq`       | `https://api.groq.com/openai/v1`                | Hosted, API key required     |
| `gemini`     | `https://generativelanguage.googleapis.com/v1beta/openai` | Hosted, API key required |
| `openrouter` | `https://openrouter.ai/api/v1`                  | Hosted, API key required     |
| `claude`     | `https://api.anthropic.com/v1`                  | Hosted, API key required     |

Provider and model are verified against the provider's `/models` listing
before anything is saved. API keys live in your OS config directory
(`~/.config/bcommit/config.json` on Linux,
`~/Library/Application Support/bcommit/` on macOS) — never in the repo.

## Commands

```sh
bcommit               # generate (default)
bcommit config        # provider + model + key
bcommit model         # switch active model
bcommit model remove  # delete a saved model
bcommit sysprompt     # show prompt
bcommit sysprompt edit
bcommit sysprompt reset
bcommit sysprompt path
bcommit --yes         # auto-commit, no prompt (for CI)
```

After generating, `bcommit` asks what to do: **Commit**, **Edit** (opens
`$EDITOR`, then asks to confirm), **Regenerate**, or **Cancel**. Nothing
commits silently unless you pass `--yes`.

## Customizing the prompt

Commit style is driven by `instructions.md`, loaded on every run:

```sh
bcommit sysprompt        # show it
bcommit sysprompt edit   # edit in $EDITOR
bcommit sysprompt reset  # restore the default
bcommit sysprompt path   # where the file lives
```

The default enforces conventional commits (`type(scope): subject`,
imperative, ≤72 chars) and prints only the message — no fences, no commentary.

## Privacy

- **Offline option:** Ollama runs fully on-device; diffs never leave your machine.
- **Keys stay local:** stored in your OS config dir, verified before saving.
- **Review gate:** every message is previewed and confirmed; `--yes` is opt-in.
- Models can misread a diff — always review before choosing Commit.

## Contributing

```sh
cargo build              # debug build
cargo build --release --locked   # release build (as CI does)
```

Bug reports and pull requests are welcome at
[github.com/iamprasadraju/Boring-Commit](https://github.com/iamprasadraju/Boring-Commit).
Please keep PRs focused; run `cargo build` (and `cargo test` where tests exist)
before pushing.

## License

MIT — see [LICENSE](LICENSE).
