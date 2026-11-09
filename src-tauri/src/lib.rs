use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Output, Stdio};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

/// Prevents a console window from flashing for each spawned process.
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Default config embedded at compile time, copied to the app data folder on first launch.
const DEFAULT_CONFIG: &str = include_str!("../../config.json");

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
	#[serde(default = "default_always_on_top")]
	always_on_top: bool,
}

fn default_opacity() -> f64 {
	0.92
}

fn default_always_on_top() -> bool {
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
fn hidden_cmd(command_line: &str) -> Command {
	let mut cmd = Command::new("cmd");
	cmd.raw_arg(format!("/S /C \"{}\"", command_line))
		.creation_flags(CREATE_NO_WINDOW);
	cmd
}

/// Calls docker directly (not through cmd, which would eat the `^` of the name filter).
fn docker(args: &[&str]) -> Result<Output, String> {
	Command::new("docker")
		.args(args)
		.creation_flags(CREATE_NO_WINDOW)
		.output()
		.map_err(|e| format!("Cannot run docker: {}", e))
}

fn docker_status(container: &str) -> String {
	match docker(&["ps", "-q", "--filter", &format!("name=^{}$", container)]) {
		Ok(out) if out.status.success() && !out.stdout.trim_ascii().is_empty() => "running",
		_ => "stopped",
	}
	.to_string()
}

/// (pid, parent pid) of every running process.
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
					emit_log(&app, &service_id, line.trim_end_matches(['\r', '\n']));
				}
			}
		}
	});
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
		let mut cmd = hidden_cmd(&format!("start \"\" {}", service.cmd.trim()));
		if let Some(cwd) = service.cwd() {
			cmd.current_dir(cwd);
		}
		cmd.spawn().map_err(|e| format!("Failed to launch '{}': {}", service.name, e))?;
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

	let child = state.processes.lock().unwrap().remove(&service_id);
	if let Some(child) = child {
		kill_all_async(vec![child]).await;
	}
	Ok(())
}

#[tauri::command]
async fn get_service_status(app: AppHandle, state: State<'_, AppState>, service_id: String) -> Result<String, String> {
	if state.shell_services.lock().unwrap().contains(&service_id) {
		return Ok("launched".to_string());
	}
	{
		let mut procs = state.processes.lock().unwrap();
		if let Some(child) = procs.get_mut(&service_id) {
			let status = match child.try_wait() {
				Ok(None) => "running",
				Ok(Some(exit)) if exit.success() => "stopped",
				_ => "error",
			};
			return Ok(status.to_string());
		}
	}
	// Docker containers are checked even if started outside the app
	let container = find_service(&app, &state, &service_id).ok().and_then(|s| s.docker_container());
	match container {
		Some(container) => tauri::async_runtime::spawn_blocking(move || docker_status(&container))
			.await
			.map_err(|e| e.to_string()),
		None => Ok("stopped".to_string()),
	}
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
	tauri::Builder::default()
		.plugin(tauri_plugin_dialog::init())
		.manage(AppState::default())
		.invoke_handler(tauri::generate_handler![
			read_config,
			save_config,
			start_service,
			stop_service,
			get_service_status,
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

	fn alive(pids: &[u32]) -> Vec<u32> {
		let table = process_table();
		pids.iter().copied().filter(|pid| table.iter().any(|(p, _)| p == pid)).collect()
	}

	/// cmd.exe with a nested cmd and a `start /B` child: every process of the tree must die.
	#[test]
	fn kill_all_kills_the_whole_tree() {
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
}
