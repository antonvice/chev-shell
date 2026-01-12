# 🐕 Chev Shell Command Guide

Welcome to the **Chev Shell** v0.1.0-alpha. Below is a list of common commands you can use. Note that for performance and a modern experience, many "standard" commands are transparently mapped to faster, Rust-based alternatives.

## 📁 Navigation & File Management

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `ls` | **eza** | `ls -la`, `ls --tree` |
| `cd` | **zoxide** | `cd Documents`, `cd project_name` (jump) |
| `tree` | **broot** | `tree` (interactive) |
| `cp` | **xcp** | `cp file.txt backup/` |
| `rm` | **rip** | `rm file.txt` (sends to graveyard) |
| `find` | **fd** | `find patterns` |
| `du` | **dua** | `du -h` |
| `fselect` | **fselect** | `fselect size, path FROM . WHERE name LIKE '%.png'` |

## ✍️ Text Processing & Viewing

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `cat` | **bat** | `cat main.rs` (with syntax highlighting) |
| `rg` | **ripgrep** | `rg "search_term" src/` |
| `sed` | **sd** | `sed 's/old/new/g' file.txt` |
| `diff` | **difftastic** | `diff file1.rs file2.rs` |
| `jq` | **jql** | `jq '.name' data.json` |
| `cut` / `awk` | **choose** | `cut 0:3 file.csv` (using slicing) |
| `man` / `tldr` | **tealdeer** | `man ls` or `tldr ls` (instant examples) |

## ⚖️ System & Monitoring

| Command | Modern Alternative | Examples |
| :--- | :--- | :--- |
| `sudo` | **sudo-rs** | `sudo ls` |
| `top` / `htop`| **bottom (btm)** | `top` (modern TUI) |
| `ps` | **procs** | `ps aux` |
| `time` | **hyperfine** | `time ./script.sh` (benchmark) |
| `watch` | **hwatch** | `watch ls` (records history) |
| `dig` | **doggo** | `dig google.com` (colorful DNS) |
| `make` | **just** | `make build` (running Justfile commands) |

## 🌍 Environment & Directory Control

| Command | Description | Examples |
| :--- | :--- | :--- |
| `set` | Set environment variable or list all | `set KEY VALUE`, `set KEY=VALUE`, `set` |
| `unset` | Remove environment variable | `unset GREETING` |
| `path` | Smart management of $PATH | `path add /bin`, `path prepend ./node_modules/.bin` |
| `pushd` | Save current dir and move | `pushd /tmp` |
| `popd` | Return to saved dir | `popd` |
| `dirs` | Show directory stack | `dirs` |

## 🕹 Job Control

| Command | Description | Examples |
| :--- | :--- | :--- |
| `&` | Run command in background | `sleep 60 &` |
| `jobs` | List background/stopped jobs | `jobs` (shows 'active for' time) |
| `fg <id>` | Bring job to foreground | `fg 1` |
| `bg <id>` | Resume job in background | `bg 2` |
| `Ctrl+Z` | Suspend foreground task | (Keyboard shortcut) |

## 🛰 Modern Advanced
*Note: The command shown in parentheses or bold is what actually executes on your system.*
