use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::process::{Child, Command, Output, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

/// Prevents a console window from flashing for each spawned process.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Default config embedded at compile time, copied to the app data folder on first launch.
const DEFAULT_CONFIG: &str = include_str!("../../config.json");

/// Page of the latest release, opened when the widget shows that an update is available
const RELEASES_URL: &str = "https://github.com/benjaminpavone/dev-launcher/releases/latest";

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
struct ServiceConfig {
	id: String,
	name: String,
	icon: String,
	cmd: String,
	cwd: String,
	conda_env: Option<String>,
	/// None or "" = normal service, "shell" = fire & forget
	#[serde(default)]
	service_type: Option<String>,
}

impl ServiceConfig {
	fn is_shell(&self) -> bool {
		self.service_type.as_deref() == Some("shell")
	}

	/// Container name when the command is `docker start <name>`.
	fn docker_container(&self) -> Option<String> {
		let parts: Vec<&str> = self.cmd.split_whitespace().collect();
		match parts.as_slice() {
			["docker", "start", name, ..] => Some(name.to_string()),
			_ => None,
		}
	}

	fn conda_env(&self) -> Option<&str> {
		self.conda_env.as_deref().map(str::trim).filter(|env| !env.is_empty())
	}

	fn cwd(&self) -> Option<&str> {
		Some(self.cwd.trim()).filter(|cwd| !cwd.is_empty())
	}
}

#[derive(Serialize, Deserialize, Clone)]
struct ProjectConfig {
	id: String,
	name: String,
	color: String,
	services: Vec<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
	projects: Vec<ProjectConfig>,
	#[serde(default = "default_opacity")]
	opacity: f64,
	#[serde(default = "default_true")]
	always_on_top: bool,
	/// Ask GitHub at startup whether a newer release exists
	#[serde(default = "default_true")]
	check_updates: bool,
}

/// Higher on macOS, where the background behind the widget is not blurred (see App.vue).
fn default_opacity() -> f64 {
	if cfg!(target_os = "macos") {
		0.97
	} else {
		0.92
	}
}

fn default_true() -> bool {
	true
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct AppState {
	/// Services of the last loaded config, by id
	services: Mutex<HashMap<String, ServiceConfig>>,
	/// Running processes of normal services, by service id
	processes: Mutex<HashMap<String, Child>>,
	/// Shell services launched this session
	shell_services: Mutex<HashSet<String>>,
	/// First local URL printed by each running normal service, by service id
	urls: Mutex<HashMap<String, LocalUrl>>,
}

fn config_path(app: &AppHandle) -> std::path::PathBuf {
	app.path()
		.app_data_dir()
		.unwrap_or_else(|_| std::path::PathBuf::from("."))
		.join("config.json")
}

fn ensure_config(app: &AppHandle) {
	let path = config_path(app);
	if path.exists() {
		return;
	}
	if let Some(parent) = path.parent() {
		let _ = std::fs::create_dir_all(parent);
	}
	let _ = std::fs::write(&path, DEFAULT_CONFIG);
}

fn cache_services(state: &AppState, config: &Config) {
	*state.services.lock().unwrap() = config
		.projects
		.iter()
		.flat_map(|p| p.services.iter())
		.map(|s| (s.id.clone(), s.clone()))
		.collect();
}

fn load_config(app: &AppHandle, state: &AppState) -> Result<Config, String> {
	ensure_config(app);
	let path = config_path(app);
	let content = std::fs::read_to_string(&path)
		.map_err(|e| format!("Cannot read config at {:?}: {}", path, e))?;
	let config: Config = serde_json::from_str(&content)
		.map_err(|e| format!("Invalid config.json: {}", e))?;
	cache_services(state, &config);
	Ok(config)
}

fn find_service(app: &AppHandle, state: &AppState, service_id: &str) -> Result<ServiceConfig, String> {
	if let Some(service) = state.services.lock().unwrap().get(service_id) {
		return Ok(service.clone());
	}
	load_config(app, state)?;
	state.services.lock().unwrap().get(service_id).cloned()
		.ok_or_else(|| format!("Service '{}' not found", service_id))
}

// ── Processes ─────────────────────────────────────────────────────────────────

/// Runs a command line through cmd.exe without a console window.
/// `/S /C "<line>"` makes cmd strip only the outer quotes, so quoted arguments survive.
#[cfg(windows)]
fn hidden_cmd(command_line: &str) -> Command {
	let mut cmd = Command::new("cmd");
	cmd.raw_arg(format!("/S /C \"{}\"", command_line))
		.creation_flags(CREATE_NO_WINDOW);
	cmd
}

/// Runs a command line through sh, in its own process group so that stopping it
/// can kill the whole group, children included.
#[cfg(unix)]
fn hidden_cmd(command_line: &str) -> Command {
	let mut cmd = Command::new("sh");
	cmd.arg("-c").arg(command_line).process_group(0);
	use_login_path(&mut cmd);
	cmd
}

/// Calls docker directly (not through cmd, which would eat the `^` of the name filter).
fn docker(args: &[&str]) -> Result<Output, String> {
	let mut cmd = Command::new("docker");
	cmd.args(args);
	#[cfg(windows)]
	cmd.creation_flags(CREATE_NO_WINDOW);
	#[cfg(unix)]
	use_login_path(&mut cmd);
	cmd.output().map_err(|e| format!("Cannot run docker: {}", e))
}

// ── macOS PATH ────────────────────────────────────────────────────────────────
// An app opened from the Finder or the Dock gets a minimal PATH (/usr/bin:/bin:/usr/sbin:/sbin):
// no nvm, Homebrew, conda or Docker. Commands get the PATH of the user's shell instead, as in
// a terminal. It is read once in the background at startup: an interactive shell can take
// several seconds to load, and a command started meanwhile waits for it.

#[cfg(unix)]
fn use_login_path(cmd: &mut Command) {
	#[cfg(target_os = "macos")]
	if let Some(path) = login_path() {
		cmd.env("PATH", path);
	}
	#[cfg(not(target_os = "macos"))]
	let _ = cmd;
}

#[cfg(target_os = "macos")]
fn login_path() -> Option<&'static str> {
	static LOGIN_PATH: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
	LOGIN_PATH.get_or_init(read_login_path).as_deref()
}

#[cfg(target_os = "macos")]
const PATH_MARKER: &str = "__DEV_LAUNCHER_PATH__";

/// Asks the user's interactive login shell for its PATH. Started from a terminal (`tauri dev`,
/// tests), the app already has it. Gives up after 15 seconds, keeping the current PATH.
#[cfg(target_os = "macos")]
fn read_login_path() -> Option<String> {
	if std::env::var_os("TERM").is_some() {
		return None;
	}
	let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
	let child = Command::new(shell)
		.arg("-ilc")
		.arg(format!("printf '{}%s' \"$PATH\"", PATH_MARKER))
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.process_group(0)
		.spawn()
		.ok()?;
	let pid = child.id();
	let (sender, receiver) = std::sync::mpsc::channel();
	std::thread::spawn(move || {
		let _ = sender.send(child.wait_with_output());
	});
	match receiver.recv_timeout(Duration::from_secs(15)) {
		Ok(Ok(output)) => parse_login_path(&String::from_utf8_lossy(&output.stdout)),
		_ => {
			kill_process_trees(&[pid]);
			None
		}
	}
}

/// The PATH printed after the marker; whatever the shell config printed before is ignored.
#[cfg(target_os = "macos")]
fn parse_login_path(output: &str) -> Option<String> {
	let (_, path) = output.rsplit_once(PATH_MARKER)?;
	let path = path.trim();
	(!path.is_empty()).then(|| path.to_string())
}

fn docker_status(container: &str) -> String {
	match docker(&["ps", "-q", "--filter", &format!("name=^{}$", container)]) {
		Ok(out) if out.status.success() && !out.stdout.trim_ascii().is_empty() => "running",
		_ => "stopped",
	}
	.to_string()
}

/// (pid, parent pid) of every running process.
#[cfg(windows)]
fn process_table() -> Vec<(u32, u32)> {
	use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
	use windows_sys::Win32::System::Diagnostics::ToolHelp::{
		CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
	};

	let mut table = Vec::new();
	unsafe {
		let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
		if snapshot == INVALID_HANDLE_VALUE {
			return table;
		}
		let mut entry: PROCESSENTRY32W = std::mem::zeroed();
		entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
		let mut found = Process32FirstW(snapshot, &mut entry);
		while found != 0 {
			table.push((entry.th32ProcessID, entry.th32ParentProcessID));
			found = Process32NextW(snapshot, &mut entry);
		}
		CloseHandle(snapshot);
	}
	table
}

/// Creation time of a process, None if it cannot be opened (gone, or not ours).
#[cfg(windows)]
fn creation_time(pid: u32) -> Option<u64> {
	use windows_sys::Win32::Foundation::{CloseHandle, FILETIME};
	use windows_sys::Win32::System::Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

	unsafe {
		let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
		if handle.is_null() {
			return None;
		}
		let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
		let (mut created, mut exited, mut kernel, mut user) = (zero, zero, zero, zero);
		let ok = GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user);
		CloseHandle(handle);
		(ok != 0).then_some(((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64)
	}
}

#[cfg(windows)]
fn terminate(pid: u32) {
	use windows_sys::Win32::Foundation::CloseHandle;
	use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

	unsafe {
		let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
		if !handle.is_null() {
			TerminateProcess(handle, 1);
			CloseHandle(handle);
		}
	}
}

/// Kills processes and all their descendants. npm can spawn node children detached from the
/// cmd.exe parent chain, which `taskkill /T` misses; walking the parent pid catches them.
/// A parent pid can be reused by Windows once its process is gone: a "child" created before
/// its supposed parent is an unrelated process and is left alone.
#[cfg(windows)]
fn kill_process_trees(roots: &[u32]) {
	let table = process_table();
	let mut tree = Vec::new();
	let mut seen = HashSet::new();
	let mut stack: Vec<u32> = roots.to_vec();
	while let Some(pid) = stack.pop() {
		if !seen.insert(pid) {
			continue;
		}
		tree.push(pid);
		let parent_created = creation_time(pid);
		for &(child, parent) in &table {
			if parent != pid || child == pid {
				continue;
			}
			let created_before_parent = matches!(
				(creation_time(child), parent_created),
				(Some(child_created), Some(parent_created)) if child_created < parent_created
			);
			if !created_before_parent {
				stack.push(child);
			}
		}
	}
	// Deepest processes first, so that no parent respawns a child already killed
	for pid in tree.into_iter().rev() {
		terminate(pid);
	}
}

/// Kills processes and all their descendants: each root was started as the leader of its
/// own process group (see `hidden_cmd`), so killing the group catches its children.
#[cfg(unix)]
fn kill_process_trees(roots: &[u32]) {
	for &pid in roots {
		// SAFETY: plain syscall, no memory involved; a group already gone only returns an error
		unsafe {
			libc::killpg(pid as libc::pid_t, libc::SIGKILL);
		}
	}
}

/// Blocks until every process tree is gone, so their ports are free when it returns.
fn kill_all(children: Vec<Child>) {
	let pids: Vec<u32> = children.iter().map(Child::id).collect();
	kill_process_trees(&pids);
	for mut child in children {
		let _ = child.wait();
	}
}

async fn kill_all_async(children: Vec<Child>) {
	if !children.is_empty() {
		let _ = tauri::async_runtime::spawn_blocking(move || kill_all(children)).await;
	}
}

fn take_all_processes(state: &AppState) -> Vec<Child> {
	state.processes.lock().unwrap().drain().map(|(_, child)| child).collect()
}

// ── Logs ──────────────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
struct LogLine {
	time: String,
	text: String,
	level: String,
}

fn strip_ansi(text: &str) -> String {
	let mut result = String::with_capacity(text.len());
	let mut chars = text.chars().peekable();
	while let Some(c) = chars.next() {
		if c == '\x1b' {
			if chars.peek() == Some(&'[') {
				chars.next();
				// consume until a letter (the command byte)
				for ch in chars.by_ref() {
					if ch.is_ascii_alphabetic() {
						break;
					}
				}
			}
		} else {
			result.push(c);
		}
	}
	result
}

fn detect_level(text: &str) -> &'static str {
	let upper = text.to_uppercase();
	if upper.contains("ERROR") || upper.contains("CRITICAL") {
		"error"
	} else if upper.contains("WARN") {
		"warn"
	} else {
		""
	}
}

fn emit_log(app: &AppHandle, service_id: &str, raw: &str) {
	let text = strip_ansi(raw);
	let level = detect_level(&text).to_string();
	let time = chrono::Local::now().format("%H:%M:%S").to_string();
	let _ = app.emit(&format!("log:{}", service_id), LogLine { time, text, level });
}

/// Forwards each line of a process pipe as a log event. Lines are decoded lossily:
/// Windows tools often print non-UTF-8 text, which must not stop the reading.
fn pipe_logs<R: Read + Send + 'static>(app: AppHandle, service_id: String, pipe: R) {
	std::thread::spawn(move || {
		let mut reader = BufReader::new(pipe);
		let mut buf = Vec::new();
		loop {
			buf.clear();
			match reader.read_until(b'\n', &mut buf) {
				Ok(0) | Err(_) => break,
				Ok(_) => {
					let line = String::from_utf8_lossy(&buf);
					let line = line.trim_end_matches(['\r', '\n']);
					remember_url(&app, &service_id, line);
					emit_log(&app, &service_id, line);
				}
			}
		}
	});
}

// ── Local URL ─────────────────────────────────────────────────────────────────

/// A dev server address on this machine, read from its logs
#[derive(Clone, Copy, Debug, PartialEq)]
struct LocalUrl {
	https: bool,
	port: u16,
}

impl LocalUrl {
	fn to_url(self) -> String {
		let scheme = if self.https { "https" } else { "http" };
		format!("{}://localhost:{}", scheme, self.port)
	}
}

/// Hosts that mean "this machine" in the address a dev server prints
const LOCAL_HOSTS: [&str; 5] = ["localhost", "127.0.0.1", "0.0.0.0", "::1", "::"];

/// First local URL with a port in a log line (ANSI codes already stripped), such as
/// Vite's `Local: http://localhost:5173/` or `Uvicorn running on http://127.0.0.1:8000`.
fn find_local_url(text: &str) -> Option<LocalUrl> {
	let mut rest = text;
	while let Some(i) = rest.find("://") {
		// The scheme is the word right before "://" (`[http://…` is fine, `xhttp://…` is not)
		let scheme = rest[..i].rsplit(|c: char| !c.is_ascii_alphanumeric()).next().unwrap_or("");
		let https = scheme.eq_ignore_ascii_case("https");
		let after = &rest[i + 3..];
		rest = after;
		if !https && !scheme.eq_ignore_ascii_case("http") {
			continue;
		}
		let (host, tail) = match after.strip_prefix('[') {
			Some(inner) => match inner.find(']') {
				Some(end) => (&inner[..end], &inner[end + 1..]),
				None => continue,
			},
			None => {
				let end = after.find([':', '/', ' ', ']']).unwrap_or(after.len());
				(&after[..end], &after[end..])
			}
		};
		if !LOCAL_HOSTS.contains(&host.to_ascii_lowercase().as_str()) {
			continue;
		}
		let Some(tail) = tail.strip_prefix(':') else { continue };
		let digits = tail.find(|c: char| !c.is_ascii_digit()).unwrap_or(tail.len());
		match tail[..digits].parse::<u16>() {
			Ok(port) if port > 0 => return Some(LocalUrl { https, port }),
			_ => continue,
		}
	}
	None
}

/// Keeps the first local URL a service prints, until it is stopped or restarted
fn remember_url(app: &AppHandle, service_id: &str, raw: &str) {
	let state = app.state::<AppState>();
	if state.urls.lock().unwrap().contains_key(service_id) {
		return;
	}
	if let Some(url) = find_local_url(&strip_ansi(raw)) {
		state.urls.lock().unwrap().entry(service_id.to_string()).or_insert(url);
	}
}

/// Whether something accepts connections on this port, over IPv4 or IPv6
fn port_is_open(port: u16) -> bool {
	let timeout = Duration::from_millis(200);
	[IpAddr::V4(Ipv4Addr::LOCALHOST), IpAddr::V6(Ipv6Addr::LOCALHOST)]
		.into_iter()
		.any(|ip| TcpStream::connect_timeout(&SocketAddr::new(ip, port), timeout).is_ok())
}

// ── Commands ──────────────────────────────────────────────────────────────────
// Commands that wait on external processes are async: sync commands run on the main
// thread and would freeze the window.

#[tauri::command]
fn read_config(app: AppHandle, state: State<'_, AppState>) -> Result<Config, String> {
	load_config(&app, &state)
}

#[tauri::command]
async fn save_config(app: AppHandle, state: State<'_, AppState>, config: Config) -> Result<(), String> {
	ensure_config(&app);
	let json = serde_json::to_string_pretty(&config)
		.map_err(|e| format!("Serialization error: {}", e))?;
	std::fs::write(config_path(&app), json).map_err(|e| format!("Cannot write config: {}", e))?;
	cache_services(&state, &config);

	// Services removed from the config can no longer be stopped from the UI: stop them now.
	// Docker containers are left alone, as on exit.
	let kept: HashSet<&str> = config.projects.iter()
		.flat_map(|p| p.services.iter())
		.map(|s| s.id.as_str())
		.collect();
	state.shell_services.lock().unwrap().retain(|id| kept.contains(id.as_str()));
	state.urls.lock().unwrap().retain(|id, _| kept.contains(id.as_str()));
	let orphans: Vec<Child> = {
		let mut procs = state.processes.lock().unwrap();
		let removed: Vec<String> = procs.keys().filter(|id| !kept.contains(id.as_str())).cloned().collect();
		removed.iter().filter_map(|id| procs.remove(id)).collect()
	};
	kill_all_async(orphans).await;
	Ok(())
}

#[tauri::command]
async fn start_service(app: AppHandle, state: State<'_, AppState>, service_id: String) -> Result<(), String> {
	let service = find_service(&app, &state, &service_id)?;

	// Shell services: fire & forget, nothing to track
	if service.is_shell() {
		#[cfg(windows)]
		let mut cmd = hidden_cmd(&format!("start \"\" {}", service.cmd.trim()));
		#[cfg(unix)]
		let mut cmd = hidden_cmd(service.cmd.trim());
		if let Some(cwd) = service.cwd() {
			cmd.current_dir(cwd);
		}
		let child = cmd.spawn().map_err(|e| format!("Failed to launch '{}': {}", service.name, e))?;
		// Reaps the shell once it exits, so it does not stay as a zombie process
		#[cfg(unix)]
		std::thread::spawn(move || {
			let mut child = child;
			let _ = child.wait();
		});
		#[cfg(windows)]
		drop(child);
		state.shell_services.lock().unwrap().insert(service_id);
		return Ok(());
	}

	// Docker services: the container runs on its own, its status comes from `docker ps`
	if let Some(container) = service.docker_container() {
		let output = tauri::async_runtime::spawn_blocking(move || docker(&["start", &container]))
			.await
			.map_err(|e| e.to_string())??;
		for bytes in [&output.stdout, &output.stderr] {
			for line in String::from_utf8_lossy(bytes).lines() {
				emit_log(&app, &service_id, line);
			}
		}
		return if output.status.success() {
			Ok(())
		} else {
			Err(format!("docker start failed ({})", output.status))
		};
	}

	{
		let mut procs = state.processes.lock().unwrap();
		if let Some(child) = procs.get_mut(&service_id) {
			if child.try_wait().map_err(|e| e.to_string())?.is_none() {
				return Ok(()); // already running
			}
			procs.remove(&service_id);
		}
	}

	let command_line = match service.conda_env() {
		Some(env) => format!("conda run --no-capture-output -n {} {}", env, service.cmd.trim()),
		None => service.cmd.trim().to_string(),
	};
	state.urls.lock().unwrap().remove(&service_id);
	let mut cmd = hidden_cmd(&command_line);
	cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
	if let Some(cwd) = service.cwd() {
		cmd.current_dir(cwd);
	}
	let mut child = cmd.spawn().map_err(|e| format!("Failed to start '{}': {}", service.name, e))?;
	if let Some(stdout) = child.stdout.take() {
		pipe_logs(app.clone(), service_id.clone(), stdout);
	}
	if let Some(stderr) = child.stderr.take() {
		pipe_logs(app.clone(), service_id.clone(), stderr);
	}
	state.processes.lock().unwrap().insert(service_id, child);
	Ok(())
}

#[tauri::command]
async fn stop_service(app: AppHandle, state: State<'_, AppState>, service_id: String) -> Result<(), String> {
	// Shell services: just forget them, external apps manage their own lifecycle
	if state.shell_services.lock().unwrap().remove(&service_id) {
		return Ok(());
	}

	let container = find_service(&app, &state, &service_id).ok().and_then(|s| s.docker_container());
	if let Some(container) = container {
		let output = tauri::async_runtime::spawn_blocking(move || docker(&["stop", &container]))
			.await
			.map_err(|e| e.to_string())??;
		return if output.status.success() {
			Ok(())
		} else {
			Err(format!("docker stop failed: {}", String::from_utf8_lossy(&output.stderr).trim()))
		};
	}

	state.urls.lock().unwrap().remove(&service_id);
	let child = state.processes.lock().unwrap().remove(&service_id);
	if let Some(child) = child {
		kill_all_async(vec![child]).await;
	}
	Ok(())
}

#[derive(Serialize)]
struct ServiceStatus {
	status: String,
	/// Port of the service's local URL, once it accepts connections
	port: Option<u16>,
}

impl ServiceStatus {
	fn new(status: &str) -> Self {
		ServiceStatus { status: status.to_string(), port: None }
	}
}

#[tauri::command]
async fn get_service_status(app: AppHandle, state: State<'_, AppState>, service_id: String) -> Result<ServiceStatus, String> {
	if state.shell_services.lock().unwrap().contains(&service_id) {
		return Ok(ServiceStatus::new("launched"));
	}
	let status = state.processes.lock().unwrap().get_mut(&service_id).map(|child| match child.try_wait() {
		Ok(None) => "running",
		Ok(Some(exit)) if exit.success() => "stopped",
		_ => "error",
	});
	if let Some(status) = status {
		let mut result = ServiceStatus::new(status);
		let url = state.urls.lock().unwrap().get(&service_id).copied();
		if let (Some(url), "running") = (url, status) {
			let open = tauri::async_runtime::spawn_blocking(move || port_is_open(url.port))
				.await
				.map_err(|e| e.to_string())?;
			result.port = open.then_some(url.port);
		}
		return Ok(result);
	}
	// Docker containers are checked even if started outside the app
	let container = find_service(&app, &state, &service_id).ok().and_then(|s| s.docker_container());
	match container {
		Some(container) => tauri::async_runtime::spawn_blocking(move || docker_status(&container))
			.await
			.map(|status| ServiceStatus::new(&status))
			.map_err(|e| e.to_string()),
		None => Ok(ServiceStatus::new("stopped")),
	}
}

/// Opens a URL built by the app itself (never one sent by the webview) in the default browser.
fn open_in_browser(url: &str) -> Result<(), String> {
	#[cfg(windows)]
	let mut cmd = hidden_cmd(&format!("start \"\" {}", url));
	#[cfg(target_os = "macos")]
	let mut cmd = Command::new("open");
	#[cfg(all(unix, not(target_os = "macos")))]
	let mut cmd = Command::new("xdg-open");
	#[cfg(unix)]
	cmd.arg(url);
	let mut child = cmd.spawn().map_err(|e| format!("Cannot open the browser: {}", e))?;
	// Reaps the launcher once it exits, so it does not stay as a zombie process
	std::thread::spawn(move || child.wait());
	Ok(())
}

/// Opens the service's local URL. The URL is built from the port the service printed,
/// so the webview can only open localhost addresses.
#[tauri::command]
fn open_service_url(state: State<'_, AppState>, service_id: String) -> Result<(), String> {
	let url = state.urls.lock().unwrap().get(&service_id).copied().ok_or("No local URL for this service")?;
	open_in_browser(&url.to_url())
}

#[tauri::command]
fn open_releases_page() -> Result<(), String> {
	open_in_browser(RELEASES_URL)
}

/// Stops every process (Docker containers are left running), then exits.
#[tauri::command]
fn shutdown(app: AppHandle, state: State<'_, AppState>) {
	let children = take_all_processes(&state);
	std::thread::spawn(move || {
		kill_all(children);
		app.exit(0);
	});
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	#[cfg(target_os = "macos")]
	std::thread::spawn(login_path);
	tauri::Builder::default()
		.plugin(tauri_plugin_dialog::init())
		.manage(AppState::default())
		.invoke_handler(tauri::generate_handler![
			read_config,
			save_config,
			start_service,
			stop_service,
			get_service_status,
			open_service_url,
			open_releases_page,
			shutdown,
		])
		.build(tauri::generate_context!())
		.expect("error while building tauri application")
		.run(|app, event| {
			// Any other way out than the close button: still stop the processes
			if let tauri::RunEvent::Exit = event {
				kill_all(take_all_processes(&app.state::<AppState>()));
			}
		});
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Serializes the tests that spawn processes and the ones that open sockets: a process
	/// spawned while a socket is being created can inherit it and keep its port open.
	fn serial() -> std::sync::MutexGuard<'static, ()> {
		static LOCK: Mutex<()> = Mutex::new(());
		LOCK.lock().unwrap_or_else(|e| e.into_inner())
	}

	#[cfg(windows)]
	fn alive(pids: &[u32]) -> Vec<u32> {
		let table = process_table();
		pids.iter().copied().filter(|pid| table.iter().any(|(p, _)| p == pid)).collect()
	}

	/// cmd.exe with a nested cmd and a `start /B` child: every process of the tree must die.
	#[cfg(windows)]
	#[test]
	fn kill_all_kills_the_whole_tree() {
		let _guard = serial();
		let child = hidden_cmd("start /B ping -n 60 127.0.0.1 >NUL & cmd /C ping -n 60 127.0.0.1 >NUL")
			.spawn()
			.expect("spawn test tree");
		let root = child.id();
		std::thread::sleep(std::time::Duration::from_millis(1500));

		let table = process_table();
		let mut tree = vec![root];
		let mut i = 0;
		while i < tree.len() {
			let pid = tree[i];
			tree.extend(table.iter().filter(|(c, p)| *p == pid && *c != pid).map(|(c, _)| *c));
			i += 1;
		}
		assert!(tree.len() >= 4, "expected cmd, ping, cmd, ping; got {:?}", tree);

		let started = std::time::Instant::now();
		kill_all(vec![child]);
		let elapsed = started.elapsed();
		std::thread::sleep(std::time::Duration::from_millis(300));

		assert!(alive(&tree).is_empty(), "still alive: {:?}", alive(&tree));
		println!("killed {} processes in {:?}", tree.len(), elapsed);
	}

	/// Pids of the processes in a process group.
	#[cfg(unix)]
	fn group_members(pgid: u32) -> Vec<u32> {
		let out = Command::new("pgrep").arg("-g").arg(pgid.to_string()).output().expect("run pgrep");
		String::from_utf8_lossy(&out.stdout).lines().filter_map(|line| line.trim().parse().ok()).collect()
	}

	/// sh with two background children: every process of the group must die.
	#[cfg(unix)]
	#[test]
	fn kill_all_kills_the_whole_tree() {
		let _guard = serial();
		let child = hidden_cmd("sleep 60 & sleep 60 & wait").spawn().expect("spawn test tree");
		let pgid = child.id();
		let started = wait_for(|| group_members(pgid).len() >= 3);
		assert!(started, "expected sh, sleep, sleep; got {:?}", group_members(pgid));

		kill_all(vec![child]);
		let killed = wait_for(|| group_members(pgid).is_empty());
		assert!(killed, "still alive: {:?}", group_members(pgid));
	}

	/// Polls for up to 2 seconds: processes start and disappear with a delay on a busy machine.
	#[cfg(unix)]
	fn wait_for(condition: impl Fn() -> bool) -> bool {
		(0..20).any(|_| {
			let done = condition();
			if !done {
				std::thread::sleep(std::time::Duration::from_millis(100));
			}
			done
		})
	}

	// ── Logs ──

	#[test]
	fn strip_ansi_removes_color_codes() {
		assert_eq!(strip_ansi("\x1b[32mOK\x1b[0m"), "OK");
		assert_eq!(strip_ansi("\x1b[1;31mERROR\x1b[0m: failed"), "ERROR: failed");
	}

	#[test]
	fn strip_ansi_keeps_plain_text() {
		assert_eq!(strip_ansi("Server started on port 8000"), "Server started on port 8000");
		assert_eq!(strip_ansi("déjà vu ✓"), "déjà vu ✓");
		assert_eq!(strip_ansi(""), "");
	}

	#[test]
	fn strip_ansi_drops_a_lone_escape() {
		assert_eq!(strip_ansi("abc\x1b"), "abc");
	}

	#[test]
	fn detect_level_finds_errors_and_warnings() {
		assert_eq!(detect_level("ERROR: connection refused"), "error");
		assert_eq!(detect_level("Critical failure"), "error");
		assert_eq!(detect_level("Warning: deprecated option"), "warn");
		assert_eq!(detect_level("[WARN] slow query"), "warn");
		assert_eq!(detect_level("Server started"), "");
	}

	#[test]
	fn detect_level_prefers_error_over_warning() {
		assert_eq!(detect_level("warning: build failed with an error"), "error");
	}

	// ── Config ──

	fn service(cmd: &str, cwd: &str, conda_env: Option<&str>, service_type: Option<&str>) -> ServiceConfig {
		ServiceConfig {
			id: "id".to_string(),
			name: "name".to_string(),
			icon: "icon".to_string(),
			cmd: cmd.to_string(),
			cwd: cwd.to_string(),
			conda_env: conda_env.map(str::to_string),
			service_type: service_type.map(str::to_string),
		}
	}

	#[test]
	fn docker_container_is_read_from_docker_start() {
		assert_eq!(service("docker start redis", "", None, None).docker_container().as_deref(), Some("redis"));
		assert_eq!(service("  docker   start   redis  ", "", None, None).docker_container().as_deref(), Some("redis"));
	}

	#[test]
	fn docker_container_is_none_for_other_commands() {
		assert_eq!(service("docker stop redis", "", None, None).docker_container(), None);
		assert_eq!(service("docker start", "", None, None).docker_container(), None);
		assert_eq!(service("npm run dev", "", None, None).docker_container(), None);
		assert_eq!(service("", "", None, None).docker_container(), None);
	}

	#[test]
	fn conda_env_ignores_blank_values() {
		assert_eq!(service("cmd", "", None, None).conda_env(), None);
		assert_eq!(service("cmd", "", Some(""), None).conda_env(), None);
		assert_eq!(service("cmd", "", Some("   "), None).conda_env(), None);
		assert_eq!(service("cmd", "", Some(" my-env "), None).conda_env(), Some("my-env"));
	}

	#[test]
	fn cwd_ignores_blank_values() {
		assert_eq!(service("cmd", "", None, None).cwd(), None);
		assert_eq!(service("cmd", "   ", None, None).cwd(), None);
		assert_eq!(service("cmd", " C:/my-app ", None, None).cwd(), Some("C:/my-app"));
	}

	#[test]
	fn is_shell_only_for_shell_type() {
		assert!(service("code .", "", None, Some("shell")).is_shell());
		assert!(!service("npm run dev", "", None, None).is_shell());
		assert!(!service("npm run dev", "", None, Some("")).is_shell());
	}

	#[test]
	fn config_uses_defaults_for_missing_fields() {
		let json = r##"{
			"projects": [{
				"id": "p",
				"name": "Project",
				"color": "#000000",
				"services": [{
					"id": "s",
					"name": "Service",
					"icon": "node",
					"cmd": "npm run dev",
					"cwd": ""
				}]
			}]
		}"##;
		let config: Config = serde_json::from_str(json).expect("minimal config should parse");
		assert_eq!(config.opacity, default_opacity());
		assert!(config.always_on_top);
		assert!(config.check_updates);
		let svc = &config.projects[0].services[0];
		assert_eq!(svc.service_type, None);
		assert_eq!(svc.conda_env, None);
		assert!(!svc.is_shell());
	}

	#[test]
	fn config_keeps_explicit_values() {
		let json = r#"{ "projects": [], "opacity": 0.5, "always_on_top": false, "check_updates": false }"#;
		let config: Config = serde_json::from_str(json).expect("config should parse");
		assert_eq!(config.opacity, 0.5);
		assert!(!config.always_on_top);
		assert!(!config.check_updates);
	}

	#[test]
	fn default_config_parses() {
		let config: Config = serde_json::from_str(DEFAULT_CONFIG).expect("embedded config.json should parse");
		assert!(!config.projects.is_empty());
		let services: Vec<&ServiceConfig> = config.projects.iter().flat_map(|p| p.services.iter()).collect();
		assert!(services.iter().any(|s| s.docker_container().is_some()));
		assert!(services.iter().any(|s| s.is_shell()));
		assert!(services.iter().any(|s| s.conda_env().is_some()));
	}

	fn http(port: u16) -> Option<LocalUrl> {
		Some(LocalUrl { https: false, port })
	}

	#[test]
	fn find_local_url_reads_dev_server_banners() {
		let banners = [
			("  ➜  Local:   http://localhost:5173/", 5173),                                   // Vite
			("INFO:     Uvicorn running on http://127.0.0.1:8000 (Press CTRL+C to quit)", 8000), // uvicorn
			("INFO:     Uvicorn running on http://0.0.0.0:8080 (Press CTRL+C to quit)", 8080),
			(" * Running on http://127.0.0.1:5000", 5000),                                       // Flask
			("Starting development server at http://127.0.0.1:8000/", 8000),                    // Django
			("   - Local:        http://localhost:3000", 3000),                                  // Next.js
			("   INFO  Server running on [http://127.0.0.1:8000].", 8000),                       // Laravel
			("[2026-11-14 10:00:00 +0100] [123] [INFO] Running on http://[::]:8000 (CTRL + C to quit)", 8000), // Hypercorn
			("Listening on http://[::1]:4321/", 4321),
			("Server at HTTP://LOCALHOST:9000", 9000),
		];
		for (line, port) in banners {
			assert_eq!(find_local_url(line), http(port), "{line}");
		}
	}

	#[test]
	fn find_local_url_keeps_https() {
		let line = "      Now listening on: https://localhost:7001"; // ASP.NET
		assert_eq!(find_local_url(line), Some(LocalUrl { https: true, port: 7001 }));
		assert_eq!(LocalUrl { https: true, port: 7001 }.to_url(), "https://localhost:7001");
	}

	#[test]
	fn find_local_url_reads_colored_vite_output() {
		let raw = "  \x1b[32m➜\x1b[39m  \x1b[1mLocal\x1b[22m:   \x1b[36mhttp://localhost:\x1b[1m5173\x1b[22m/\x1b[39m";
		assert_eq!(find_local_url(&strip_ansi(raw)), http(5173));
	}

	#[test]
	fn find_local_url_keeps_the_first_local_one() {
		let line = "see https://vitejs.dev/config or http://192.168.1.20:5173, local http://localhost:5174 and http://localhost:5175";
		assert_eq!(find_local_url(line), http(5174));
	}

	#[test]
	fn find_local_url_ignores_lines_without_a_local_port() {
		for line in [
			"",
			"VITE v8.0.0  ready in 312 ms",
			"  ➜  Network: http://192.168.1.20:5173/",
			"Docs at https://fastapi.tiangolo.com",
			"http://localhost/ has no port",
			"ftp://localhost:21",
			"xhttp://localhost:3000",
			"foohttps://localhost:3000",
			"http://localhost:0",
			"http://localhost:99999",
			"http://[::1",
			"localhost:3000 without a scheme",
		] {
			assert_eq!(find_local_url(line), None, "{line}");
		}
	}

	#[test]
	fn port_is_open_follows_a_real_listener() {
		let _guard = serial();
		let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind a free port");
		let port = listener.local_addr().unwrap().port();
		assert!(port_is_open(port));
		drop(listener);
		assert!(!port_is_open(port));
	}

	#[cfg(target_os = "macos")]
	#[test]
	fn parse_login_path_ignores_what_the_shell_config_prints() {
		let output = format!("Welcome!\n\x1b[32mnvm loaded\x1b[0m\n{}/usr/local/bin:/usr/bin:/bin\n", PATH_MARKER);
		assert_eq!(parse_login_path(&output).as_deref(), Some("/usr/local/bin:/usr/bin:/bin"));
		assert_eq!(parse_login_path("no marker at all"), None);
		assert_eq!(parse_login_path(PATH_MARKER), None);
	}
}
