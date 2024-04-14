# Contributing

## General Steps

Thank you for considering to contribute to `pacdef`. The recommended workflow is
this:

1. Open a github issue, mention that you would like to fix the issue in a PR.
2. Wait for approval.
3. Fork the repository and implement your fix / feature.
4. Make sure your code generates no warnings, and passes `rustfmt` and `clippy`.
5. Open the pull request.

## Rust-Analyzer Issues

Rust Analyzer may not work unless both the `pacutils` and `apt` packages are
installed. On Arch that is, this may vary on other distros.
