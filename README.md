# Dev Launcher

[![CI](https://github.com/benjaminpavone/dev-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/benjaminpavone/dev-launcher/actions/workflows/ci.yml)

A small, transparent, always-on-top Windows widget to start, stop and monitor your local dev servers — uvicorn, npm/Vite, Docker containers — without keeping a terminal open for each of them.

**Windows only** for now; a macOS version is planned.

<p align="center"><img src="docs/screenshot.png" alt="Dev Launcher widget with two projects and a Docker service running" width="300"></p>

## Download

From the [Releases](../../releases) page, built by GitHub Actions from the tagged source:

- **`dev-launcher_<version>_portable.exe`**: download and run, nothing to install (needs WebView2, built into Windows 10 and 11);
- **`dev-launcher_<version>_x64-setup.exe`**: installer with a Start menu shortcut and an uninstaller.

The app is not code-signed, so Windows SmartScreen may warn on first launch: click **More info**, then **Run anyway**.

## Getting started

1. Launch the app. The first time, it shows example projects with placeholder paths.
2. Open the **Settings** (gear icon) and replace them with your own projects and services. For each service:
   - **Command**: what you would type in a terminal, e.g. `npm run dev` or `uvicorn main:app --port 8000 --reload`;
   - **Working directory**: the folder to run it from (**Browse** to pick it);
   - **Conda env** (optional): a conda environment to run the command in;
   - **Type**: *Normal* for a server, *Shell* to simply open an app (see below);
   - **Icon**: Python, Node, Docker or Shell, for display only.
3. Click **Save**. Each service then has a Start/Stop button, and each project a button to start or stop all its services at once.

## Service types

- **Normal**: the app runs the command and tracks it: *Running*, *Stopped*, or *Error* if it exits with an error. **Stop** kills the process and all its child processes, so the port is free as soon as the service shows as stopped.
- **Docker**: a Normal service whose command is `docker start <container>` is handled as a container. Its status comes from `docker ps`, even when the container was started outside the app, and **Stop** runs `docker stop`.
- **Shell** (fire & forget): opens an app such as VS Code, Teams or a `.bat`. It shows as *Launched* but is not tracked: clicking its button again only marks it as stopped, nothing is killed.

Closing Dev Launcher stops every Normal service it started. Docker containers and Shell apps keep running.

## Logs

Each Normal or Docker service has a **Logs** button showing its live output, with errors and warnings in color, a filter that highlights matching text, and an expand button for a larger view. Logs are kept in memory while the app is open.

## Settings and configuration

The settings panel also sets the widget **opacity** and whether it stays **always on top** (on by default).

Everything is saved in `%APPDATA%\com.devlauncher.app\config.json`. You can back up this file, or edit it by hand while the app is closed.

## Troubleshooting: servers left running

If servers survive a crash, kill them by command line instead of killing every `node.exe` or `python.exe` (which would also kill unrelated tools):

```powershell
# Vite / npm run dev
Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -match 'vite' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }

# uvicorn
Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -match 'uvicorn' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
```

## Development

Built with **Tauri 2** (Rust) and **Vue 3**. Prerequisites: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) and the [Tauri prerequisites for Windows](https://tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev
```

The CI runs Clippy and the unit tests on Windows for each push. To build the installers and publish a release, see [docs/RELEASING.md](docs/RELEASING.md).

## License

[MIT](LICENSE)
