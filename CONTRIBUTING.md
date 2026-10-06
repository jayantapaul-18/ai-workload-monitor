# Contributing

Thanks for helping improve AI Workload Monitor. The project is licensed under the [MIT License](LICENSE).

## Setup

```bash
./scripts/install-deps.sh
npm install
npm run tauri dev
```

`npm install` enables the git hooks.

## Checks

```bash
npm run typecheck
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

The pre-commit hook runs the TypeScript check and `cargo fmt --check`.

## Commits

Use [Commitizen](https://github.com/commitizen/cz-cli) so messages stay [conventional](https://www.conventionalcommits.org/):

```bash
npm run commit
```

Commitlint rejects messages that do not match `type: subject` or `type(scope): subject`. Common types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`.

## Pull requests

Open a pull request against `main`. Describe what changed and how you checked it. GitHub Actions runs the same typecheck, format check, and Rust tests.
