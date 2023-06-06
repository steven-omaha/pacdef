Release Checklist
-----------------
* Ensure local `main` is up to date with respect to `origin/main`.
* Run `cargo update` and review dependency updates.
  Commit updated `Cargo.lock`.
* Run `cargo outdated` and review semver incompatible updates. 
  Unless there is a strong motivation otherwise, review and update every dependency.
* Update the date and version in all man pages.
* Run `cargo publish -p pacdef_core -p pacdef <level>`.
  `pacdef_core` and `pacdef` shall have the same version at all times.
  Verify everything works as expected.
* Rerun `cargo publish` with `--execute.`
* Bump the AUR package.
