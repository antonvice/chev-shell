# 🐕 Chev Shell (chev-shell)

![Chev Shell Logo](https://raw.githubusercontent.com/catppuccin/catppuccin/main/assets/logos/exports/154x154_circle.png)

## The AI-Native, Rust-Powered Command Line for the Modern Age

**Chev Shell** is not just another shell. It's a high-performance, GPU-accelerated co-processor for your terminal that redefines how you interact with your operating system. Built entirely in Rust, it merges the reliability of classic POSIX shells with the intelligence of modern AI.

---

## ✨ Why Chev?

Most shells stay out of your way. **Chev** works *with* you. It is designed to be **Transparent**, **Intelligent**, and **Fast**.

### 🚀 Novel Features

#### 1. **Modern-First Command Mapping**

Chev transparently "upgrades" your legacy POSIX commands to their modern, multi-threaded, and colorful Rust-based counterparts. You don't need to configure aliases; Chev does it by default:

- `ls` ⮕ `eza`
- `cat` ⮕ `bat` / `mdcat`
- `find` ⮕ `fd`
- `top` ⮕ `btm`
- `man` ⮕ `tldr`
- *...and [many more](commands.md).*

#### 2. **AI-Native Core**

Built-in bridge to local LLMs (via Ollama). Chev can explain complex flags, suggest command corrections, perform semantic history searches, and even browse the web for you—all while keeping your data local and private.

#### 3. **Rio Deep Integration**

Designed specifically for the **Rio Terminal**. Chev uses custom OSC sequences to trigger native UI events like notifications, split-panes (IDE mode), background effects (Matrix/Vibe), and macOS QuickLook previews.

#### 4. **Semantic History (Mimic)**

Every command you run is embedded and stored in a local vector database (LanceDB). Find how you did something yesterday using natural language: `ai search "how did I connect to s3?"`.

#### 5. **Advanced UX**

- **Fish-style Autosuggestions**: Gray-text completion based on history.
- **Smart Macros**: Argument expansion ($1, $2, $) for complex workflows.
- **Visual Blocks**: OSC 133 support for traceable command blocks.

---

## 🛠 Setup & Installation

### Prerequisites

1. **Ollama**: [Download here](https://ollama.com) (Required for AI features).
2. **Protobuf**: `brew install protobuf` (Required for LanceDB).
3. **Rio Terminal**: [Download here](https://rioterm.com).

### 🚀 Getting Started

Once you have the prerequisites, follow these steps to build and initialize your workspace using the unified **Root Makefile**:

```bash
# 1. Clone the repository
git clone https://github.com/antonvice/chev-shell.git
cd chev-shell

# 2. Build both Shell and Terminal with one command
make build

# 3. Enter the shell (from the chev-shell directory)
cd chev-shell
./target/release/chev

# 4. Global Setup (Inside Chev)
# This installs all modern Rust tools and pulls the AI model
ai setup
```

To package everything into a single **Rio.app** (macOS) that includes the Chev binary:

```bash
# Return to root and run release
make release-macos
```

---

## 🗺 Roadmap

- [x] **Phase 1: Foundations**
  - [x] Rust-based execution engine & Job Control
  - [x] Lexer & Parser (nom) with Pipes and Redirection
  - [x] Transparent tool mapping (ls -> eza, etc.)
  - [x] Environment management & Directory stack (pushd/popd)

- [x] **Phase 2: Intelligence**
  - [x] Ollama bridge (qwen2.5-coder:7b)
  - [x] `ai ask` & `ai fix` (Proactive error correction)
  - [x] `ai search` (Semantic history via LanceDB)
  - [x] `ai browse` (Web summarization in sidebar)

- [x] **Phase 3: Visual & Rio Integration**
  - [x] Custom OSC 1338 Protocol
  - [x] IDE Mode (Split-pane broot integration)
  - [x] Native macOS QuickLook (`preview`)
  - [x] Holographic History & Background Shaders (Matrix/Vibe)

- [ ] **Phase 4: Future Tech**
  - [ ] GPU Grep (WebGPU offloading)
  - [ ] `yt-term` (Terminal Video streaming)
  - [ ] WASM Plugin System
  - [ ] Cross-platform parity

---

## 📖 Documentation

- [Command Reference](commands.md) - Full list of mappings and built-ins.
- [Dev Guide](../guide.md) - Technical specification and architecture.

## 🐕 Behind the Name

The shell is named after my girlfriend, **Chev**—designed to be as elegant, smart, and reliable as she is.

---

Built with 🦀 and ❤️ by [Anton Vice](https://github.com/antonvice)
