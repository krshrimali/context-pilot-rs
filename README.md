
# Context Pilot

Just the tool that answers following questions for you:

1. "What all commits ever touched this piece of code?"
2. "What are the files related to this piece of code?"
3. "Who all touched this piece of code?"

Eventually answering following questions for you: (via your own brain or LLMs)

1. "What could have caused the bug?"
2. "Why was this changed in the last month? What's the reason?"
3. "Where can I find the tests written for this code?"
4. "Where should I make the change while working on this code?"

Gist: Whether you want to find:
- **Who** wrote a particular line (author search 🧑‍💻),
- **Which** files are most related to a given section (context search 📄),
- Or **index** your whole workspace efficiently with Git history 🔥,

Context Pilot gives you **fast**, **powerful**, and **local-first** code intelligence.

## Installation

Context Pilot is available via [homebrew](https://github.com/krshrimali/homebrew-context-pilot):

```shell
brew install krshrimali/context-pilot/context-pilot
```


And via AUR: https://aur.archlinux.org/packages/contextpilot and git package
here: https://aur.archlinux.org/packages/contextpilot-git.

If you're not using homebrew or AUR, please build this project from source for now (we are working on adding this to other package managers):

```bash
git clone https://github.com/krshrimali/context-pilot-rs.git
cd context-pilot-rs
cargo build --release
```

This will generate the binary at `./target/release/contextpilot`.

Move it to a path that's in your `$PATH` to run it globally:

```bash
cp ./target/release/contextpilot /usr/local/bin/contextpilot
```

Once done, you should be able to do: `contextpilot --help`

---

## ✨ Features

- 📈 **History Analysis:** Understand *who* contributed to every line.
- 🔍 **Context Extraction:** Find *related files* automatically based on commit histories.
- 🗂️ **Smart Indexing:** Index your project into a **fast sharded database** for quick queries.
- 🚀 **Rust-Powered:** Extremely **fast** and **lightweight** — no servers needed.
- 🧠 **Multi-level tracing:** Traverses multiple previous commits to capture richer history.
- ⚡ **Editor Integrations:** Works with **Neovim** and **VSCode** extensions.
- 🔒 **Local-first:** Never sends your code outside your machine.

## 🚀 Usage

### Index your workspace

```bash
contextpilot /path/to/workspace -t index
```

This will **index** your project and store smartly sharded JSON database files at:

```bash
~/.context_pilot_db/<workspace>/
```

---

### Selectively Index your Workspace

```
contextpilot /path/to/workspace -t index "subdir1,subdir2"
```

Pass relative paths to the argument as above, and it will only index those
folders for you.

---

### Query for Top Context Files

```bash
contextpilot /path/to/workspace -t query path/to/file.rs -s <start-line> -e <end-line>
```

Fetch **top related files** for the selected line range.

---

### Get relevant commits

```bash
contextpilot /path/to/workspace -t desc path/to/file.rs -s <start-line> -e <end-line>
```

Gives you the relevant commits to the selected piece of code.

---

### Fetch PR review comments

```bash
contextpilot /path/to/workspace -t prreviewcomments <commit-hash>
```

Fetches GitHub PR review comments for a commit that was merged via a PR. This helps you see the code review discussions that happened during the PR review process.

---

### Get commit descriptions with PR review comments

```bash
contextpilot /path/to/workspace -t descwithprcomments path/to/file.rs -s <start-line> -e <end-line>
```

This combines the functionality of `desc` and `prreviewcomments` modes. It first gets the relevant commit hashes for the selected line range, then fetches both commit descriptions and PR review comments for each commit. This gives you a complete picture of the code changes and the discussions that happened during code review.

---

## 🖥️ Editor Integrations

### Neovim

- Plugin available: https://github.com/krshrimali/context-pilot.nvim (details available on the link).

### VSCode

- Just search available on VSCode Marketplace with name `contextpilot` under the name of Kushashwa Ravi Shrimali as the publisher :)
- Extension available here: https://github.com/krshrimali/context-pilot-vscode.
