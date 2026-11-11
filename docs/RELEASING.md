# Releasing

## Local release build

```bash
npm run build:release
```

Do not run `npm run tauri build` directly: it does not bump the version. `build:release`:

1. bumps the **patch** version in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` (`scripts/bump-version.js`, which can also be run alone);
2. runs `npm run tauri build`;
3. renames the executable to `src-tauri\target\release\dev-launcher_<version>.exe` (`scripts/rename-exe.js`).

After each release, add a section at the top of [`CHANGELOG.md`](../CHANGELOG.md).

## GitHub release

To publish installers on GitHub, push a tag matching the version: `git tag v1.0.0 && git push origin v1.0.0`. The [release workflow](../.github/workflows/release.yml) builds on a Windows runner and attaches the installers to a draft release, which you then publish.
