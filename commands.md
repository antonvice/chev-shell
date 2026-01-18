# 🐕 Chev Shell Command Guide

Welcome to the **Chev Shell** v0.1.0-alpha. Below is a list of common commands you can use. Many "standard" commands are transparently mapped to faster, Rust-based alternatives for a modern experience.

---

## 📁 Navigation & File Management

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `ls` | **eza** | `ls -la`, `ls --tree` |
| `cd` | **zoxide** | `cd Documents`, `cd project_name` (jump) |
| `tree` | **broot** | `tree` (interactive, triggers IDE split) |
| `cp` | **xcp** | `cp file.txt backup/` |
| `rm` | **rip** | `rm file.txt` (sends to graveyard) |
| `find` | **fd** | `find patterns` |
| `du` | **dust** | `du -h` |
| `df` | **lfs** | `df` |
| `ouch` | **ouch** | `ouch compress src/ archive.zip` |
| `serve` | **miniserve** | `serve .` |
| `preview` | **Native QuickLook** | `preview image.png`, `preview doc.pdf` |

---

## ✍️ Text Processing & Viewing

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `cat` | **bat** / **mdcat** | `cat main.rs` (code) or `cat README.md` (renders Markdown) |
| `rg` | **ripgrep** | `rg "search_term" src/` |
| `sed` | **sd** | `sed 's/old/new/g' file.txt` |
| `diff` | **delta** | `diff file1.rs file2.rs` |
| `jq` | **jql** | `jq '.name' data.json` |
| `csv` | **qsv** | `csv data.csv` |
| `cut` / `awk` | **choose** | `cut 0:3 file.csv` (using slicing) |
| `man` / `tldr` | **tealdeer** | `man ls` |
| `hex` | **heh** | `hex binary_file` |
| `strings` | **lemmeknow** | `strings binary` |
| `nano` | **kibi** | `nano file.txt` |

---

## ⚖️ System & Monitoring

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `sudo` | **sudo-rs** | `sudo ls` |
| `top` / `htop` | **bottom (btm)** | `top` |
| `ps` | **procs** | `ps aux` |
| `time` | **hyperfine** | `time ./script.sh` |
| `watch` | **hwatch** | `watch ls` |
| `dig` | **doggo** | `dig google.com` |
| `make` | **just** | `make build` |
| `ping` | **gping** | `ping google.com` |
| `curl` / `http` | **xh** | `http google.com` |
| `calc` / `bc` | **fend** | `calc "10 miles to km"` |

---

## 🌍 Environment & Directory Control

| Command | Description | Examples |
| :--- | :--- | :--- |
| `set` | Set environment variable or list all | `set KEY VALUE` |
| `unset` | Remove environment variable | `unset GREETING` |
| `path` | Smart management of $PATH | `path add /bin` |
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
| `ai browse` | Browse and summarize a webpage in sidebar | `ai browse https://rust-lang.org` |
| `ai chat` | Open a persistent AI chat sidebar | `ai chat` |
| `ai status` | Check Ollama and model health | `ai status` |
| `ai setup` | Global onboarding (Install model + power-ups) | `ai setup` |

---

## 🌊 Rio Terminal Control

| Command | Description | Examples |
| :--- | :--- | :--- |
| `rio notify` | Send a native macOS notification | `rio notify "Done" "Build finished"` |
| `rio opacity` | Adjust terminal transparency (0.0-1.0) | `rio opacity 0.8` |
| `rio badge` | Set a text badge on the current tab | `rio badge DEV` |
| `history toggle` | Toggle the "Holographic History" timeline | `history toggle on` |
| `minimap` | Toggle scrollback mini-map overlay | `minimap off` |
| `effect` | Trigger GPU background shaders | `effect matrix`, `effect vibe`, `effect none` |
| `progress` | Render a native GPU progress bar | `progress 0.5 "Loading..."` |
| `voice setup` | Setup Metal-accelerated voice control | `voice setup` |

---

## 🐚 Macros & Abbreviations

| Command | Description | Examples |
| :--- | :--- | :--- |
| `macro set` | Create a shortcut with args ($1, $) | `macro set ll ls -la` |
| `macro unset` | Remove a saved macro | `macro unset ll` |
| `macro` | List all active macros | `macro` |
| `abbr` | Create a visual expansion shortcut | `abbr gco git checkout` |

---

## 🛠️ Lifecycle & Management

| Command | Description | Examples |
| :--- | :--- | :--- |
| `chev install` | Symlink shell to `/usr/local/bin/chev` | `chev install` |
| `chev uninstall` | Wipe EVERYTHING (tools, configs, link) | `chev uninstall` |
| `chev cleanup` | Reset history, suggestions, and macros | `chev cleanup` |
| `chev build` | Recompile shell from current source | `chev build` |

---

## 🕹️ Job Control

| Command | Description | Examples |
| :--- | :--- | :--- |
| `&` | Run command in background | `sleep 60 &` |
| `jobs` | List background/stopped jobs | `jobs` |
| `fg <id>` | Bring job to foreground | `fg 1` |
| `bg <id>` | Resume job in background | `bg 2` |
| `Ctrl+Z` | Suspend foreground task | (Keyboard shortcut) |
