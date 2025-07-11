# ASIMOV Chromium Module

[![License](https://img.shields.io/badge/license-Public%20Domain-blue.svg)](https://unlicense.org)
[![Compatibility](https://img.shields.io/badge/rust-1.85%2B-blue)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
[![Package on Crates.io](https://img.shields.io/crates/v/asimov-chromium-module)](https://crates.io/crates/asimov-chromium-module)
[![Documentation](https://docs.rs/asimov-chromium-module/badge.svg)](https://docs.rs/asimov-chromium-module)

[ASIMOV] module for Chromium (and Brave, Google Chrome) bookmark import.

## ✨ Features

- Extracts bookmarks from Chromium-based browsers (currently Brave and Google Chrome).
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

#### Importing bookmarks from Chrome

```bash
asimov-chromium-importer chrome://bookmarks
```

#### Importing bookmarks from Brave

```bash
asimov-chromium-importer brave://bookmarks
```

### Import of Bookmarks Files

#### Parsing bookmarks files on macOS

```bash
asimov-chromium-reader < $HOME/Library/Application\ Support/Google/Chrome/Profile\ 1/Bookmarks
asimov-chromium-reader < $HOME/Library/Application\ Support/BraveSoftware/Brave-Browser/Default/Bookmarks
```

#### Parsing bookmarks files on Linux

```bash
asimov-chromium-reader < $HOME/.config/google-chrome/Profile\ 1/Bookmarks
asimov-chromium-reader < $HOME/.config/BraveSoftware/Brave-Browser/Default/Bookmarks
```

#### Parsing bookmarks files on Windows

```powershell
Get-Content "$env:LOCALAPPDATA\Google\Chrome\User Data\Profile 1\Bookmarks" | asimov-chromium-reader
Get-Content "$env:LOCALAPPDATA\BraveSoftware\Brave-Browser\User Data\Default\Bookmarks" | asimov-chromium-reader
```

## ⚙ Configuration

This module requires no configuration.

## 📚 Reference

### `asimov-chromium-importer`

```
```

### `asimov-chromium-reader`

```
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
