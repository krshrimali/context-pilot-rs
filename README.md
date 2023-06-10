# ContextPilot: All the context, right at your ~door~ code step.

## What is ContextPilot?

Use this binary to find top N (all for now) relevant files and authors for the given:

- Line of Code
- Range of lines
- Current File

Plugins available for both NeoVim and Visual Studio Code.

## Why do you need it?

We all ask the questions below, once in every 2 days if not daily!

- Hi, where to look for the usage of this class?
- Hey, do you know who could help me with this?
- I changed this file, is there any other file I should look at as well?
- I’m not sure if I understand where we are using this.

As developers, code means us more than a paragraph. This does exactly that.

## Algorithm

INPUT: Your whole file/section of code/current line

For each line:

    - If line >= Threshold:
        - let commit_hash = get_commit_sha(line_number.start, line_number.end)
        - file_author_details = get_files_changed(commit_hash)
        - recurse(commit_hash) # 5 times:
            - commit_hash = get_previous_commit_sha(commit_hash)
            - file_author_details = get_files_changed(commit_hash)
        - Generate BTreeMap for frequency mapping
        - return most N common files and authors

## Future Steps

1. Store results for each commit hash in an in-house DB
2. Indexing when the editor starts (optional)
3. Configurable for both VSCode and NeoVim
4. Ship PyCharm Extension
5. Integrate LLM for deciding top 5 files and authors for relevance

## Building

(TODO: Improve this section of code)

```bash
cargo build --release
cargo run <args>
```

## Want to contribute?

Create an issue and let me know!!
