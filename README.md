# ASIMOV Chromium Module

[![License](https://img.shields.io/badge/license-Public%20Domain-blue.svg)](https://unlicense.org)
[![Compatibility](https://img.shields.io/badge/rust-1.85%2B-blue)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
[![Package on Crates.io](https://img.shields.io/crates/v/asimov-chromium-module)](https://crates.io/crates/asimov-chromium-module)
[![Documentation](https://docs.rs/asimov-chromium-module/badge.svg)](https://docs.rs/asimov-chromium-module)

[ASIMOV] module for Chromium (and Brave, Google Chrome) bookmark import.

## ✨ Features

- Extracts bookmarks from Chromium-based browsers (Chromium, Brave, Google Chrome, and Microsoft Edge).
- Constructs a semantic knowledge graph based on the [KNOW] ontology.
- Supports [RDF] linked data output in the form of [JSON-LD].
- Distributed as a standalone static binary with zero runtime dependencies.

## 🛠️ Prerequisites

- [Rust] 1.85+ (2024 edition) if building from source code

## ⬇️ Installation

### Installation with the [ASIMOV CLI]

```bash
asimov module install chromium -v
```

### Installation from Source Code

```bash
cargo install asimov-chromium-module
```

## 👉 Examples

### Import of Browser Bookmarks

#### Importing bookmarks from Chromium

```bash
asimov-chromium-fetcher chromium://bookmarks
asimov-chromium-fetcher chromium://bookmarks/Profile\ 1
```

#### Importing bookmarks from Chrome

```bash
asimov-chromium-fetcher chrome://bookmarks
asimov-chromium-fetcher chrome://bookmarks/Default
```

#### Importing bookmarks from Brave

```bash
asimov-chromium-fetcher brave://bookmarks
asimov-chromium-fetcher brave://bookmarks/Profile\ 2
```

#### Importing bookmarks from Microsoft Edge

```bash
asimov-chromium-fetcher edge://bookmarks
asimov-chromium-fetcher edge://bookmarks/Profile\ 1
```

### Import of Bookmarks Files

#### Parsing bookmarks files on macOS

```bash
asimov-chromium-reader < $HOME/Library/Application\ Support/Chromium/Profile\ 1/Bookmarks
asimov-chromium-reader < $HOME/Library/Application\ Support/Google/Chrome/Profile\ 1/Bookmarks
asimov-chromium-reader < $HOME/Library/Application\ Support/BraveSoftware/Brave-Browser/Default/Bookmarks
asimov-chromium-reader < $HOME/Library/Application\ Support/Microsoft\ Edge/Profile\ 1/Bookmarks
```

#### Parsing bookmarks files on Linux

```bash
asimov-chromium-reader < $HOME/.config/chromium/Profile\ 1/Bookmarks
asimov-chromium-reader < $HOME/.config/google-chrome/Profile\ 1/Bookmarks
asimov-chromium-reader < $HOME/.config/BraveSoftware/Brave-Browser/Default/Bookmarks
asimov-chromium-reader < $HOME/.config/microsoft-edge/Profile\ 1/Bookmarks
```

#### Parsing bookmarks files on Windows

```powershell
Get-Content "$env:LOCALAPPDATA\Chromium\User Data\Profile 1\Bookmarks" | asimov-chromium-reader
Get-Content "$env:LOCALAPPDATA\Google\Chrome\User Data\Profile 1\Bookmarks" | asimov-chromium-reader
Get-Content "$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data\Default\Bookmarks" | asimov-chromium-reader
Get-Content "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Profile 1\Bookmarks" | asimov-chromium-reader
```

## ⚙ Configuration

This module requires no configuration.

## 📚 Reference

### `asimov-chromium-fetcher`

```
asimov-chromium-fetcher

Usage: asimov-chromium-fetcher [OPTIONS] <URL>

Arguments:
  <URL>  The `chromium://bookmarks`, `chrome://bookmarks`, `brave://bookmarks`, or `edge://bookmarks` URL to fetch

Options:
  -d, --debug       Enable debugging output
      --license     Show license information
  -v, --verbose...  Enable verbose output (may be repeated for more verbosity)
  -V, --version     Print version information
  -h, --help        Print help
```

### `asimov-chromium-reader`

```
asimov-chromium-reader

Usage: asimov-chromium-reader [OPTIONS]

Options:
  -d, --debug       Enable debugging output
      --license     Show license information
  -v, --verbose...  Enable verbose output (may be repeated for more verbosity)
  -V, --version     Print version information
  -h, --help        Print help
```

## 👨‍💻 Development

```bash
git clone https://github.com/asimov-modules/asimov-chromium-module.git
```

---

[![Share on X](https://img.shields.io/badge/share%20on-x-03A9F4?logo=x)](https://x.com/intent/post?url=https://github.com/asimov-modules/asimov-chromium-module&text=asimov-chromium-module)
[![Share on Reddit](https://img.shields.io/badge/share%20on-reddit-red?logo=reddit)](https://reddit.com/submit?url=https://github.com/asimov-modules/asimov-chromium-module&title=asimov-chromium-module)
[![Share on Hacker News](https://img.shields.io/badge/share%20on-hn-orange?logo=ycombinator)](https://news.ycombinator.com/submitlink?u=https://github.com/asimov-modules/asimov-chromium-module&t=asimov-chromium-module)
[![Share on Facebook](https://img.shields.io/badge/share%20on-fb-1976D2?logo=facebook)](https://www.facebook.com/sharer/sharer.php?u=https://github.com/asimov-modules/asimov-chromium-module)
[![Share on LinkedIn](https://img.shields.io/badge/share%20on-linkedin-3949AB?logo=linkedin)](https://www.linkedin.com/sharing/share-offsite/?url=https://github.com/asimov-modules/asimov-chromium-module)

[ASIMOV]: https://asimov.sh
[ASIMOV CLI]: https://cli.asimov.sh
[JSON-LD]: https://json-ld.org
[KNOW]: https://know.dev
[Maildir]: https://en.wikipedia.org/wiki/Maildir
[RDF]: https://www.w3.org/TR/rdf12-primer/
[Rust]: https://rust-lang.org
