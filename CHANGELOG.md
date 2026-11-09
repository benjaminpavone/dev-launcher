# Changelog

## [1.0.0] - 2026-11-09

First public release.

### Added

- **GitHub Actions**: CI on every push (frontend build, `cargo clippy`, tests on Windows) and a release workflow that builds the Windows installers from a version tag
- **"Always on top" setting** in the settings panel (on by default, as before): the widget can now behave like a normal window

### Fixed

- **UI no longer freezes** while a service stops or while Docker statuses are polled: the Tauri commands that wait on external processes are now async (sync commands run on the main thread). Stop still returns only once the port is free
- **Shell services with arguments** (`code C:/my-project`) are launched correctly: the command line is passed to `cmd /S /C` as is instead of being quoted as a single program name
- **"Stop all" on a project** containing a launched Shell service: launched counts as up, so the project can be stopped in one click. The project counter counts launched Shell services too
- **Saving the settings no longer clears the logs** of running services
- **Removing a running service** in the settings now stops its process instead of leaving it alive and unreachable
- **Docker container name filter**: `docker ps` is called directly instead of through `cmd`, which ate the `^` anchor (`my-redis` matched `redis`)
- **Maximize/Restore icon** stays in sync with the window after a settings save
- **Exit through any path** kills the process tree like the Stop button, instead of `taskkill /T`
- **Non-UTF-8 output** (common with Windows tools) no longer drops log lines

### Changed

- UI fully in English
- **Stop and close are instant**: the process tree is walked and killed with the Windows API (`windows-sys`) instead of a PowerShell script, which took about 3 seconds per process tree. A process created before its supposed parent (reused pid) is left alone
- Dependencies updated to their latest stable versions, pinned: Tauri 2.12, Vue 3.5.45, Vite 8, `@vitejs/plugin-vue` 6 (`npm audit`: 0 vulnerabilities)
- Service config cached in memory instead of read from disk at each status poll
- Docker `start` output shown in the service logs; a failed `docker start` or `docker stop` shows the service in error
- The working directory can be typed in the settings, not only picked with Browse
- Package renamed `dev-launcher` (executable `dev-launcher.exe`). The app identifier, and so the config location, are unchanged
- `bump-version.js` also bumps `Cargo.toml` and `Cargo.lock`, and writes JSON with tabs
- Unused opener plugin and template assets removed; example `config.json`

---

### Security

- **Content Security Policy** enabled: the webview only loads the app's own files and talks to the Tauri backend

## [0.1.6] - 2026-07-03

### Added

- **Maximize/Restore button** in the title bar, between Minimize and Close (Windows order). The icon follows the window state, including when maximized by double-clicking the title bar
- **Log search**: filter field in each service's log panel (case-insensitive, live). Matches are **highlighted**; a message shows when no line matches
- **Full-screen logs**: new button to open a service's logs in a large modal (same search and highlight, search field focused on open, filtered line counter, closes with Escape or a click outside). Text stays selectable

### Changed

- **Title bar reorganized**: global "Start all / Stop all" button removed. The **Settings** button moves to the left, right after the `running/total` counter

---

## [0.1.5] - 2026-04-20

### Added

- **"Shell" service type (fire & forget)**: launches external apps (Teams, Slack, .exe, .bat…) without capturing stdout or killing them on stop. The app is launched with `cmd /C start ""` and its status becomes "Launched" (purple): we cannot know whether it is still running
- **Shell icon** (rocket, indigo)
- **"Type" dropdown** in the settings: _Normal_ or _Shell (fire & forget)_
- **"Launched" status badge**, distinct from "Running", never counted in the footer's `running/total`
- **Launch button** replaces Start/Stop for Shell services; the log panel is hidden (nothing to capture)

### Fixed

- `runningCount` and `totalCount` exclude Shell services so the footer counter stays accurate

---

## [0.1.4] - 2026-04-20

### Fixed

- **Incomplete process kill (port still in use)**: `taskkill /T` relies on Windows parent-child relations, but `npm` can spawn `node`/`vite` detached, so they survived and kept their port. Replaced by a recursive PowerShell kill that walks every descendant through `ParentProcessId`, even detached ones
- **`rename-exe.js`**: the Tauri executable name was wrong

---

## [0.1.3] - 2026-04-17

### Fixed

- **Port still in use after Stop**: the kill ran in a separate thread, so the port was not free yet when Start was clicked again. The kill is now synchronous: the port is free before `Ok()` is returned to the frontend

---

## [0.1.2] - 2026-04-17

### Fixed

- **Incomplete process kill**: `child.kill()` killed `cmd.exe` before `taskkill /T`, leaving the child processes (vite, node, uvicorn) orphaned with their port. Only `taskkill /F /T` is used now
- **Saving the settings no longer stops the services**: their statuses are kept after a Save

---

## [0.1.1] - 2026-04-17

### Fixed

- **Stopping services froze the UI**: blocking calls (`docker stop`, `taskkill`, `child.wait()`) now run in separate threads
- **Docker stopped on settings save**: Docker services are excluded from the automatic stop on Save
- **ANSI logs**: color codes generated by Vite/npm are stripped on the Rust side before being emitted

### Added

- **Version number** at the bottom left of the widget
- **`build:release` script**: bumps the patch version, then runs the Tauri build
- **Immediate poll at startup and after Save**, without waiting for the 10-second cycle
- **Opacity setting**: slider from 20% to 100%

---

## [0.1.0] - Initial

- Transparent always-on-top widget (Tauri 2 + Vue 3)
- Start and stop services (uvicorn, npm, docker)
- Docker status detected at startup without a prior click
- Log panel per service (resizable, selectable text)
- Settings panel (projects, services, conda env, cwd)
- Config embedded in the binary (first launch without a file)
- Clean shutdown without visible CMD windows
