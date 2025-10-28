# Code Snippets for Review

This directory contains copies of key source files formatted as markdown for easy review by Web Claude.

## Purpose

Web Claude (browser-based) cannot directly access your local filesystem or clone git repos, but it can fetch files from GitHub. By copying key files here as markdown and pushing to GitHub, you enable Web Claude to review your code while maintaining the source of truth in the actual source directories.

## Workflow

1. **After making changes**, run `./review-prep.sh` to copy files here
2. **Commit and push** to GitHub
3. **Share the GitHub URL** with Web Claude for review
4. **Web Claude reviews** and provides feedback
5. **Apply feedback** to actual source files
6. **Repeat** as needed

## Files

Files are copied from their source locations and wrapped in markdown code blocks:
- `app.rs.md` - Main application (`onboarding-wasm/src/app.rs`)
- `manager.rs.md` - State manager (`onboarding-wasm/src/onboarding/manager.rs`)
- `state.rs.md` - State definitions (`onboarding-wasm/src/onboarding/state.rs`)
- `actions.rs.md` - Action definitions (`onboarding-wasm/src/onboarding/actions.rs`)

## Note

These are COPIES for review purposes only. The actual source of truth is in `src/` and `onboarding-wasm/src/`.
