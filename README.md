# 🐕 Chev Shell (chev-shell)

<p align="center">
  <img src="https://raw.githubusercontent.com/catppuccin/catppuccin/main/assets/logos/exports/154x154_circle.png" width="100" />
</p>

<h3 align="center">The AI-Native, Rust-Powered Command Line for the Modern Age</h3>

<p align="center">
  <strong>Chev Shell</strong> is not just another shell. It's a high-performance, GPU-accelerated co-processor for your terminal that redefines how you interact with your operating system. Built entirely in Rust, it merges the reliability of classic POSIX shells with the intelligence of modern AI.
</p>

---

## ✨ Why Chev?

Most shells stay out of your way. **Chev** works *with* you. It is designed to be **Transparent**, **Intelligent**, and **Fast**.

### 🚀 Novel Features

#### 1. **Modern-First Command Mapping**
Chev transparently "upgrades" your legacy POSIX commands to their modern, multi-threaded, and colorful Rust-based counterparts. You don't need to configure aliases; Chev does it by default:
- `ls` ⮕ `eza`
- `cat` ⮕ `bat`
- `find` ⮕ `fd`
- `top` ⮕ `btm`
- `man` ⮕ `tldr`
- *...and [many more](commands.md).*

#### 2. **AI-Native Core**
Built-in bridge to local LLMs (via Ollama). Chev is designed to explain complex flags, suggest command corrections, and generate one-liners using models like `qwen2.5-coder:7b`, all while keeping your data local and private.

#### 3. **Smart Semantic Navigation**
The `cd` command is powered by `zoxide` logic. It learns your habits, allowing you to "jump" to frequently used directories without typing full paths (e.g., `cd proj` might take you straight to `~/Documents/work/rust/project-alpha`).

#### 4. **Intelligent Auto-suggestions**
Leverages a high-performance **Trie-based engine** to provide ghost-text suggestions from your history. Accept suggestions instantly with `Tab` or `Right Arrow`.

#### 5. **Advanced Macro & Abbreviation Engine**
Support for Fish-style abbreviations and powerful macros with argument expansion (`$1`, `$@`, etc.), allowing you to build complex workflows that feel like native commands.

#### 6. **Robust Job Control**
Modernized job management that tracks "Active For" time for background processes, with native `fg`, `bg`, and `jobs` implementation built on Nix.

---

## 🛠 Installation

### Prerequisites
Chev expects several modern Rust tools to be installed on your system for the full experience:
```bash
cargo install eza bat fd-find bottom tealdeer zoxide xcp ripgrep sd
```

### Build from Source
```bash
git clone https://github.com/antonvice/chev-shell.git
cd chev-shell/chev-shell
cargo build --release
```

---

## 🗺 Roadmap

- [ ] **Phase 1: Foundations (Current)**
  - [x] Rust-based execution engine
  - [x] Transparent tool mapping
  - [x] Basic job control & environment management
  - [x] Trie-based suggestions

- [ ] **Phase 2: Intelligence**
  - [ ] Deep integration with Ollama for command explanation
  - [ ] `ctrl-f` (Find AI) to search history semantically
  - [ ] Natural language to CLI (e.g., `? find all large logs`)

- [ ] **Phase 3: Visual & UX**
  - [ ] GPU-accelerated UI rendering components
  - [ ] Interactive file picker (`fselect` / `broot` integration)
  - [ ] Custom theme engine (Catppuccin by default)

- [ ] **Phase 4: Ecosystem**
  - [ ] Plugin system (WASM or Rust)
  - [ ] Cross-platform parity (Full Windows support)

---

## 📖 Documentation
- [Command Reference](commands.md) - Full list of mappings and built-ins.

## 🐕 Behind the Name
The shell is named after my girlfriend, **Chev**—designed to be as elegant, smart, and reliable as she is.

---

<p align="center">Built with 🦀 and ❤️ by <a href="https://github.com/antonvice">Anton Vice</a></p>
