Release Checklist
-----------------
* Ensure local `main` is up to date with respect to `origin/main`.
* Run `cargo update` and review dependency updates.
  Commit updated `Cargo.lock` with "chore(release): update lockfile".
* Run `cargo outdated` and review semver incompatible updates. 
  Unless there is a strong motivation otherwise, review and update every dependency.
* Update the date and version in all man pages: "chore(release): bump man pages".
* Run `cargo release -p pacdef <version>`.
  `pacdef_core` and `pacdef` shall have the same version at all times.
  Verify everything works as expected.
* Rerun `cargo publish` with `--execute.`
* Generate GitHub release with `git cliff`
* Bump the AUR package.
