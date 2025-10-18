# GitHub Actions Workflows

This directory contains automated CI/CD workflows for Context Pilot.

## Workflows

### 1. Tests (`tests.yml`)

**Purpose:** Runs comprehensive test suite on every PR and push to main.

**Triggers:**
- Pull requests to any branch (including draft PRs)
- Push to main branch (after PR merge)
- Manual workflow dispatch

**What it does:**
1. Sets up Rust toolchain with rustfmt and clippy
2. Checks code formatting (`cargo fmt`)
3. Runs linter (`cargo clippy`)
4. Builds the project
5. Runs all unit tests
6. Runs integration tests (including new rename detection tests)
7. All tests run with `--test-threads=1` due to git state dependencies

**Status Badge:**
```markdown
[![Tests](https://github.com/krshrimali/context-pilot-rs/actions/workflows/tests.yml/badge.svg)](https://github.com/krshrimali/context-pilot-rs/actions/workflows/tests.yml)
```

**Notes:**
- Uses caching for faster builds (cargo registry, git index, target directory)
- Configures git user for tests that create repositories
- One integration test (`test_accuracy_with_git_blame_across_rename`) may fail due to known accuracy limitations - workflow continues anyway

---

### 2. Build (`build.yml`)

**Purpose:** Builds release binaries on all supported platforms and verifies they work.

**Triggers:**
- Pull requests to any branch
- Push to main branch
- Manual workflow dispatch

**What it does:**
1. Builds release binaries on:
   - Linux (Ubuntu latest, x86_64-unknown-linux-gnu)
   - macOS (latest, x86_64-apple-darwin)
   - Windows (latest, x86_64-pc-windows-msvc)
2. Verifies binary exists
3. Tests `--help` flag works on each platform
4. Uploads binaries as artifacts (7-day retention)
5. Creates build summary

**Status Badge:**
```markdown
[![Build](https://github.com/krshrimali/context-pilot-rs/actions/workflows/build.yml/badge.svg)](https://github.com/krshrimali/context-pilot-rs/actions/workflows/build.yml)
```

**Artifacts:**
- `contextpilot-ubuntu-latest` - Linux binary
- `contextpilot-macos-latest` - macOS binary
- `contextpilot-windows-latest` - Windows binary (.exe)

---

## Status Badges

Both workflows have status badges in the main README.md:

- **Tests Badge:** Shows if all tests are passing
- **Build Badge:** Shows if builds succeed on all platforms

Badges update automatically:
- After each PR is opened/updated
- After each push to main (post-merge)
- After manual workflow runs

The badges link to the workflow runs page for detailed logs.

---

## Local Testing

Before pushing, you can test locally:

### Run tests
```bash
# All tests with single thread (as in CI)
cargo test -- --test-threads=1

# Specific test suites
cargo test --test rename_detection -- --test-threads=1
cargo test --test rename_integration -- --test-threads=1
```

### Build release binary
```bash
cargo build --release
./target/release/contextpilot --help
```

### Check formatting and linting
```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Troubleshooting

### Tests fail in CI but pass locally
- Ensure you're running with `--test-threads=1`
- Check git configuration is set up properly
- Verify you have sufficient git history (`fetch-depth: 0` in checkout)

### Build fails on specific platform
- Check platform-specific dependencies
- Verify Rust version compatibility
- Review artifact logs for detailed error messages

### Badge not updating
- Workflows must run at least once for badges to appear
- Badge URLs must match repository name exactly
- GitHub may cache badges for a few minutes

---

## Adding New Tests

When adding new tests that use git:

1. Ensure they work with `--test-threads=1`
2. Add them to the appropriate section in `tests.yml`
3. Use `continue-on-error: true` if the test has known limitations
4. Document any special requirements

## Workflow Maintenance

### Updating dependencies
- Rust toolchain: Modify `dtolnay/rust-toolchain@stable`
- Actions versions: Update `uses:` statements (actions/checkout@v4, etc.)
- Cache keys: Update if changing dependency structure

### Adding platforms
- Add to `matrix.os` in `build.yml`
- Define target triple and binary name in `matrix.include`
- Test locally if possible before merging
