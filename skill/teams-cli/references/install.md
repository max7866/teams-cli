# Installation

## CLI Binary

Verify the binary exists:

```sh
command -v teams
```

If missing, build and install from the repo (requires Rust toolchain):

```sh
# Clone if needed
git clone https://github.com/max7866/teams-cli.git
cd teams-cli

# Build and install
cargo install --path .
```

### Platform prerequisites

- **macOS**: No additional dependencies (WebKit via WKWebView).
- **Linux**: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`.
- **Windows**: WebView2 (pre-installed on Windows 10+).

## Skill Installation

To install this skill globally so it's always available in Claude Code:

```sh
mkdir -p ~/.claude/skills/teams-cli
cp -r skill/teams-cli/* ~/.claude/skills/teams-cli/
```

The cache script also requires `jq`:

```sh
# macOS
brew install jq

# Linux
apt install jq
```
