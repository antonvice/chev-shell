# 🐕 Chev Shell Command Guide

Welcome to the **Chev Shell** v0.1.0-alpha. Below is a list of common commands you can use. Note that for performance and a modern experience, many "standard" commands are transparently mapped to faster, Rust-based alternatives.

---

## 📁 Navigation & File Management

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `ls` | **eza** | `ls -la`, `ls --tree` |
| `cd` | **zoxide** | `cd Documents`, `cd project_name` (jump) |
| `tree` | **broot** | `tree` (interactive) |
| `cp` | **xcp** | `cp file.txt backup/` |
| `rm` | **rip** | `rm file.txt` (sends to graveyard) |
| `find` | **fd** | `find patterns` |
| `du` | **dust** | `du -h` |
| `df` | **lfs** | `df` (Better disk view) |
| `ouch` | **ouch** | `ouch compress src/ archive.zip`, `ouch decompress archive.tar.gz` |
| `serve` | **miniserve** | `serve .` (Serve current dir over HTTP) |
| `fselect` | **fselect** | `fselect size, path FROM . WHERE name LIKE '%.png'` |

---

## ✍️ Text Processing & Viewing

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `cat` | **bat** / **mdcat** | `cat main.rs` (code) or `cat README.md` (renders Markdown) |
| `rg` | **ripgrep** | `rg "search_term" src/` |
| `sed` | **sd** | `sed 's/old/new/g' file.txt` |
| `diff" | **delta** | `diff file1.rs file2.rs` |
| `jq` | **jql** | `jq '.name' data.json` |
| `csv` | **qsv** | `csv data.csv` (Slice, dice, and view CSVs) |
| `cut` / `awk` | **choose** | `cut 0:3 file.csv` (using slicing) |
| `man` / `tldr` | **tealdeer** | `man ls` or `tldr ls` (instant examples) |
| `hex` | **heh** | `hex binary_file` (Interactive hex editor) |
| `strings` / `peek` | **lemmeknow** | `strings binary` or `detect "mysterious text"` (Identifies content) |
| `nano` | **kibi** | `nano file.txt` (Minimalist Rust editor) |

---

## ⚖️ System & Monitoring

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `sudo` | **sudo-rs** | `sudo ls` |
| `top` / `htop` | **bottom (btm)** | `top` (modern TUI) |
| `ps` | **procs** | `ps aux` |
| `time` | **hyperfine** | `time ./script.sh` (benchmark) |
| `watch` | **hwatch** | `watch ls` (records history) |
| `dig` | **doggo** | `dig google.com` (colorful DNS) |
| `make` | **just** | `make build` (running Justfile commands) |
| `ping` | **gping** | `ping google.com` (Graphical ping) |
| `curl` / `http` | **xh** | `http google.com`, `curl postman-echo.com/get` |
| `calc` / `bc` | **fend** | `calc "10 miles to km"`, `bc "sin(pi/2)"` |

---

## 🌍 Environment & Directory Control

| Command | Description | Examples |
| :--- | :--- | :--- |
| `set` | Set environment variable or list all | `set KEY VALUE`, `set KEY=VALUE`, `set` |
| `unset` | Remove environment variable | `unset GREETING` |
| `path` | Smart management of $PATH | `path add /bin`, `path prepend ./node_modules/.bin` |
| `pushd` | Save current dir and move | `pushd /tmp` |
| `popd` | Return to saved dir | `popd` |
| `dirs` | Show directory stack | `dirs` |

---

## 🤖 AI Integration (The "🐕 Chev" Brain)

| Command | Description | Examples |
| :--- | :--- | :--- |
| `ai ask` | Ask any coding or terminal question | `ai ask "how to list large files?"` |
| `ai fix` | Fix the last failed command automatically | `ai fix` |
| `ai search` | Semantic history search (uses embeddings) | `ai search "git log from yesterday"` |
| `ai status` | Check Ollama and model health | `ai status` |
| `ai setup` | Global onboarding (Install model + power-ups) | `ai setup` |

---

## 🐚 Macros & Abbreviations

| Command | Description | Examples |
| :--- | :--- | :--- |
| `macro set` | Create a logical shortcut with args ($1, $) | `macro set ll ls -la` |
| `macro unset` | Remove a saved macro | `macro unset ll` |
| `macro` | List all active macros | `macro` |
| `abbr` | Create a "Fish-style" visual expansion | `abbr gco git checkout` |

---

## 🛠️ Lifecycle & Management

| Command | Description | Examples |
| :--- | :--- | :--- |
| `chev install` | Symlink shell to `/usr/local/bin/chev` | `chev install` |
| `chev uninstall` | Wipe EVERYTHING (tools, configs, link) | `chev uninstall` |
| `chev cleanup` | Reset history, suggestions, and macros | `chev cleanup` |
| `chev build` | Recompile shell from current source | `chev build` |
| `chev setup` | Setup advice and environment check | `chev setup` |

---

## 🕹️ Job Control

| Command | Description | Examples |
| :--- | :--- | :--- |
| `&` | Run command in background | `sleep 60 &` |
| `jobs` | List background/stopped jobs | `jobs` |
| `fg <id>` | Bring job to foreground | `fg 1` |
| `bg <id>` | Resume job in background | `bg 2` |
| `Ctrl+Z` | Suspend foreground task | (Keyboard shortcut) |

---

## 🛰️ Modern Advanced

*Note: Commands in **bold** or modern alternatives are automatically executed for a 10x faster experience.*
