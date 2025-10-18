# Git Hooks for Context Pilot

This directory contains git hooks to maintain code quality and consistency.

## Pre-Commit Hook

The pre-commit hook ensures that all Rust code is properly formatted before committing.

### What It Does

1. **Checks for rustfmt** - Ensures you have rustfmt installed
2. **Formats checking** - Runs `cargo fmt --check` on staged Rust files
3. **Clippy linting** - Runs `cargo clippy` to catch common issues (warning only)
4. **Prevents bad commits** - Stops the commit if formatting issues are found
5. **Provides helpful instructions** - Shows how to fix issues

### Installation

Run the installation script from the repository root:

```bash
./git-hooks/install-hooks.sh
```

Or manually:

```bash
cp .git-hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Usage

Once installed, the hook runs automatically on every commit:

```bash
git add .
git commit -m "Your commit message"
```

**If formatting issues are found:**

```
❌ Formatting check failed!

The following files need formatting:
src/main.rs
src/lib.rs

To fix formatting issues, run:
  cargo fmt

Then stage the changes and commit again:
  git add .
  git commit
```

**Fix and retry:**

```bash
cargo fmt
git add .
git commit -m "Your commit message"
```

### Bypassing the Hook

**Not recommended**, but you can bypass the hook if necessary:

```bash
git commit --no-verify -m "Your commit message"
```

Only use `--no-verify` when absolutely necessary (e.g., emergency fixes, work-in-progress commits that you'll clean up later).

### Troubleshooting

#### Hook not running

1. Make sure the hook is executable:
   ```bash
   chmod +x .git/hooks/pre-commit
   ```

2. Verify the hook exists:
   ```bash
   ls -la .git/hooks/pre-commit
   ```

#### rustfmt not found

Install rustfmt:
```bash
rustup component add rustfmt
```

#### Clippy not found

Install clippy:
```bash
rustup component add clippy
```

#### Hook running on non-Rust files

The hook only checks `.rs` files. If it's checking other files, there may be a configuration issue.

### Customization

You can customize the hook behavior by editing `.git-hooks/pre-commit`:

- **Disable clippy checks**: Comment out the clippy section
- **Make clippy errors fatal**: Change the clippy warning to `exit 1`
- **Adjust output colors**: Modify the color variables
- **Add additional checks**: Add more validation steps

### CI/CD Integration

This pre-commit hook complements the GitHub Actions workflows:

- **Local**: Pre-commit hook catches issues before they're pushed
- **CI**: GitHub Actions workflows verify all PRs and merges
- **Result**: Fewer failed CI builds, faster development cycle

### Best Practices

1. **Always run the hook** - Don't use `--no-verify` unless necessary
2. **Format regularly** - Run `cargo fmt` periodically during development
3. **Check clippy** - Run `cargo clippy` before committing to catch issues early
4. **Keep hooks updated** - Pull latest changes to get hook improvements

### Uninstalling

To remove the pre-commit hook:

```bash
rm .git/hooks/pre-commit
```

To reinstall:

```bash
./git-hooks/install-hooks.sh
```

## Future Hooks

We may add additional hooks in the future:

- **pre-push** - Run full test suite before pushing
- **commit-msg** - Validate commit message format
- **post-merge** - Update dependencies after pulling changes

Stay tuned for updates!
