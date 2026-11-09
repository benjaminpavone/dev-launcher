<template>
	<div class="widget" :style="{ '--bg-alpha': opacity }">
		<!-- Shimmer top border -->
		<div class="shimmer"></div>

		<!-- Settings panel (full overlay) -->
		<SettingsPanel
			v-if="showSettings"
			:config="rawConfig"
			@close="showSettings = false"
			@saved="onConfigSaved"
		/>

		<!-- Header -->
		<div class="widget-header">
			<div class="header-left">
				<div class="logo">
					<span class="logo-dot"></span>
					<span class="logo-text">dev<span class="logo-accent">launcher</span></span>
				</div>
				<div class="badge">{{ runningCount }}/{{ totalCount }}</div>
				<button class="icon-btn" @click="showSettings = true" title="Settings">
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
					</svg>
				</button>
			</div>
			<div class="header-actions">
				<button class="icon-btn" @click="minimize" title="Minimize">
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="5" y1="12" x2="19" y2="12"/></svg>
				</button>
				<button class="icon-btn" @click="toggleMaximize" :title="isMaximized ? 'Restore' : 'Maximize'">
					<svg v-if="!isMaximized" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1.5"/></svg>
					<svg v-else width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="7" y="7" width="13" height="13" rx="1.5"/><path d="M4 15V5.5A1.5 1.5 0 0 1 5.5 4H15"/></svg>
				</button>
				<button class="icon-btn icon-btn--close" @click="closeWindow" title="Close">
					<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
						<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
					</svg>
				</button>
			</div>
		</div>


		<!-- Projects list -->
		<div class="widget-body">
			<div v-for="project in projects" :key="project.id" class="project-group">
				<!-- Project header -->
				<div class="project-header">
					<div class="project-color" :style="{ background: project.color }"></div>
					<span class="project-name">{{ project.name }}</span>
					<div class="project-meta">
						<span class="project-count">
							{{ project.services.filter(isUp).length }}/{{ project.services.length }}
						</span>
						<button
							class="btn-group-toggle"
							:class="{ 'all-running': isProjectUp(project) }"
							@click="toggleProject(project)"
							:title="isProjectUp(project) ? 'Stop all' : 'Start all'"
						>
							<svg v-if="!isProjectUp(project)" width="9" height="9" viewBox="0 0 24 24" fill="currentColor"><polygon points="5,3 19,12 5,21"/></svg>
							<svg v-else width="8" height="8" viewBox="0 0 24 24" fill="currentColor"><rect x="4" y="4" width="16" height="16" rx="2"/></svg>
						</button>
					</div>
				</div>

				<!-- Services -->
				<div class="services-list">
					<ServiceRow
						v-for="svc in project.services"
						:key="svc.id"
						:service="svc"
						@toggle="toggleService"
						@clearLogs="clearLogs"
					/>
				</div>
			</div>
		</div>

		<!-- Footer -->
		<div class="widget-footer">
			<span class="footer-version">v{{ appVersion }}</span>
			<div class="footer-right">
				<div class="status-dot" :class="{ 'all-good': runningCount === totalCount, error: hasError }"></div>
				<span>{{ runningCount }} running</span>
			</div>
		</div>
	</div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import ServiceRow from "./components/ServiceRow.vue";
import SettingsPanel from "./components/SettingsPanel.vue";

const appVersion = __APP_VERSION__;

const projects = ref([]);
const rawConfig = ref({ projects: [], opacity: 0.92, always_on_top: true });
const opacity = ref(0.92);
const showSettings = ref(false);
const isMaximized = ref(false);
const MAX_LOG_LINES = 300;
let logUnlisteners = [];
let unlistenResize = null;

// ── Load config from Rust ──────────────────────────────────────────────────
// Also called after a settings save: services that still exist keep their status and logs.
async function loadConfig() {
	try {
		const config = await invoke("read_config");
		rawConfig.value = config;
		opacity.value = config.opacity ?? 0.92;
		getCurrentWindow().setAlwaysOnTop(config.always_on_top ?? true)
			.catch(e => console.error("Failed to set always on top:", e));
		const previous = new Map(allServices.value.map(s => [s.id, s]));
		projects.value = config.projects.map(p => ({
			...p,
			services: p.services.map(s => ({
				...s,
				status: previous.get(s.id)?.status ?? "stopped",
				logs: previous.get(s.id)?.logs ?? [],
			})),
		}));
		await setupLogListeners();
		startPolling();
		pollOnce();
	} catch (e) {
		console.error("Failed to load config:", e);
	}
}

async function setupLogListeners() {
	logUnlisteners.forEach(fn => fn());
	logUnlisteners = await Promise.all(allServices.value.map(svc =>
		listen(`log:${svc.id}`, (event) => {
			svc.logs.push(event.payload);
			if (svc.logs.length > MAX_LOG_LINES) svc.logs.shift();
		})
	));
}

async function onConfigSaved() {
	showSettings.value = false;
	await loadConfig();
}

// ── Status polling ─────────────────────────────────────────────────────────
// Services with a start or stop in progress are skipped: their status is set by toggleService.
let pollInterval = null;
async function pollOnce() {
	await Promise.all(allServices.value.map(async svc => {
		if (svc.pending) return;
		try {
			const status = await invoke("get_service_status", { serviceId: svc.id });
			if (!svc.pending) svc.status = status;
		} catch {}
	}));
}
function startPolling() {
	clearInterval(pollInterval);
	pollInterval = setInterval(pollOnce, 10000);
}

onMounted(async () => {
	await loadConfig();
	const win = getCurrentWindow();
	isMaximized.value = await win.isMaximized();
	// Keeps the icon in sync with the window (button AND double-click on the title bar)
	unlistenResize = await win.onResized(async () => {
		isMaximized.value = await win.isMaximized();
	});
});
onUnmounted(() => {
	clearInterval(pollInterval);
	logUnlisteners.forEach(fn => fn());
	unlistenResize?.();
});

// ── Computed ──────────────────────────────────────────────────────────────
const allServices = computed(() => projects.value.flatMap(p => p.services));
const normalServices = computed(() => allServices.value.filter(s => s.service_type !== "shell"));
const runningCount = computed(() => normalServices.value.filter(s => s.status === "running").length);
const totalCount   = computed(() => normalServices.value.length);
const hasError     = computed(() => allServices.value.some(s => s.status === "error"));

// ── Actions ────────────────────────────────────────────────────────────────
// A launched Shell service counts as up: "Stop all" forgets it like the other services.
function isUp(svc) {
	return svc.status === "running" || svc.status === "launched";
}
function isProjectUp(project) {
	return project.services.length > 0 && project.services.every(isUp);
}

async function toggleService(id) {
	const svc = allServices.value.find(s => s.id === id);
	if (!svc || svc.pending) return;
	svc.pending = true;
	try {
		if (isUp(svc)) {
			await invoke("stop_service", { serviceId: id });
			svc.status = "stopped";
		} else {
			svc.status = "starting";
			await invoke("start_service", { serviceId: id });
			svc.status = svc.service_type === "shell" ? "launched" : "running";
		}
	} catch (e) {
		svc.status = "error";
		svc.logs.push({ time: new Date().toTimeString().slice(0, 8), text: String(e), level: "error" });
	} finally {
		svc.pending = false;
	}
}

function toggleProject(project) {
	const allUp = isProjectUp(project);
	for (const svc of project.services) {
		if (allUp || !isUp(svc)) toggleService(svc.id);
	}
}

function clearLogs(id) {
	const svc = allServices.value.find(s => s.id === id);
	if (svc) svc.logs = [];
}

async function minimize() {
	await getCurrentWindow().minimize();
}
async function toggleMaximize() {
	await getCurrentWindow().toggleMaximize();
}
async function closeWindow() {
	await invoke("shutdown");
}
</script>

<style scoped>
/* ── Widget shell ── */
.widget {
	width: 100%; height: 100vh;
	display: flex; flex-direction: column;
	background: rgba(10, 10, 20, var(--bg-alpha, 0.92));
	backdrop-filter: blur(40px) saturate(180%);
	-webkit-backdrop-filter: blur(40px) saturate(180%);
	border-radius: 18px;
	border: 1px solid rgba(255,255,255,0.08);
	font-family: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
	font-size: 12px; color: rgba(255,255,255,0.82);
	overflow: hidden;
	box-shadow:
		0 0 0 1px rgba(124,58,237,0.12),
		0 24px 80px rgba(0,0,0,0.8),
		inset 0 1px 0 rgba(255,255,255,0.06);
	user-select: none;
	position: relative;
}

/* Top shimmer line */
.shimmer {
	position: absolute; top: 0; left: 0; right: 0; height: 1px;
	background: linear-gradient(90deg, transparent, rgba(124,58,237,0.7) 35%, rgba(220,38,38,0.5) 65%, transparent);
	z-index: 2; pointer-events: none;
}

/* ── Header ── */
.widget-header {
	display: flex; justify-content: space-between; align-items: center;
	padding: 12px 14px 11px;
	background: rgba(255,255,255,0.03);
	border-bottom: 1px solid rgba(255,255,255,0.06);
	-webkit-app-region: drag;
	flex-shrink: 0; position: relative; z-index: 1;
}
.header-left  { display: flex; align-items: center; gap: 9px; }
.header-left .icon-btn { -webkit-app-region: no-drag; }
.header-actions { display: flex; gap: 4px; -webkit-app-region: no-drag; }

.logo { display: flex; align-items: center; gap: 8px; }
.logo-dot {
	width: 8px; height: 8px;
	background: radial-gradient(circle, #a78bfa, #7c3aed);
	border-radius: 50%;
	box-shadow: 0 0 6px rgba(124,58,237,0.9), 0 0 14px rgba(124,58,237,0.4);
	animation: pulse 3s ease-in-out infinite;
}
@keyframes pulse {
	0%,100% { box-shadow: 0 0 6px rgba(124,58,237,0.9), 0 0 14px rgba(124,58,237,0.4); }
	50%      { box-shadow: 0 0 3px rgba(124,58,237,0.3), 0 0 6px rgba(124,58,237,0.1); }
}
.logo-text { font-size: 13px; font-weight: 650; color: rgba(255,255,255,0.9); letter-spacing: -0.1px; }
.logo-accent {
	background: linear-gradient(135deg, #a78bfa, #dc2626);
	-webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;
}

.badge {
	font-size: 9px; font-weight: 700; letter-spacing: 0.4px;
	background: rgba(124,58,237,0.18); border: 1px solid rgba(124,58,237,0.3);
	color: #a78bfa; padding: 2px 7px; border-radius: 20px;
}

/* ── Icon buttons ── */
.icon-btn {
	width: 26px; height: 26px;
	display: flex; align-items: center; justify-content: center;
	background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);
	border-radius: 8px; color: rgba(255,255,255,0.35);
	cursor: pointer; transition: all 0.18s;
}
.icon-btn:hover {
	background: rgba(255,255,255,0.1); color: rgba(255,255,255,0.85);
	border-color: rgba(255,255,255,0.15);
}
.icon-btn--close:hover {
	background: rgba(220,38,38,0.2) !important;
	color: #fca5a5 !important; border-color: rgba(220,38,38,0.35) !important;
	box-shadow: 0 0 12px rgba(220,38,38,0.15) !important;
}

/* ── Body ── */
.widget-body {
	flex: 1; overflow-y: auto; overflow-x: hidden;
	padding: 8px; min-height: 0;
	scrollbar-width: thin; scrollbar-color: rgba(124,58,237,0.2) transparent;
	display: flex; flex-direction: column; gap: 8px;
}
.widget-body::-webkit-scrollbar { width: 3px; }
.widget-body::-webkit-scrollbar-thumb { background: rgba(124,58,237,0.25); border-radius: 2px; }

/* ── Project group ── */
.project-group { display: flex; flex-direction: column; gap: 3px; }

.project-header {
	display: flex; align-items: center; gap: 7px;
	padding: 5px 8px 5px 10px;
	border-radius: 8px;
	background: rgba(255,255,255,0.03);
}
.project-color {
	width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
}
.project-name { flex: 1; font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.6px; color: rgba(255,255,255,0.4); }
.project-meta { display: flex; align-items: center; gap: 6px; }
.project-count { font-size: 9px; color: rgba(255,255,255,0.25); }

.btn-group-toggle {
	width: 20px; height: 20px;
	border-radius: 6px; border: 1px solid rgba(255,255,255,0.08);
	background: rgba(255,255,255,0.05); color: rgba(255,255,255,0.35);
	cursor: pointer; display: flex; align-items: center; justify-content: center;
	transition: all 0.18s;
}
.btn-group-toggle:hover {
	background: rgba(74,222,128,0.15); border-color: rgba(74,222,128,0.3);
	color: #4ade80; box-shadow: 0 0 10px rgba(74,222,128,0.1);
}
.btn-group-toggle.all-running:hover {
	background: rgba(248,113,113,0.15) !important; border-color: rgba(248,113,113,0.3) !important;
	color: #fca5a5 !important;
}

.services-list { display: flex; flex-direction: column; gap: 3px; padding-left: 4px; }

/* ── Footer ── */
.widget-footer {
	display: flex; justify-content: space-between; align-items: center;
	padding: 7px 14px; border-top: 1px solid rgba(255,255,255,0.05);
	font-size: 9px; color: rgba(255,255,255,0.2);
	flex-shrink: 0; background: rgba(0,0,0,0.15);
}
.footer-version { color: rgba(255,255,255,0.15); font-size: 9px; }
.footer-right { display: flex; align-items: center; gap: 5px; }
.status-dot {
	width: 5px; height: 5px; border-radius: 50%;
	background: rgba(255,255,255,0.15);
}
.status-dot.all-good { background: #4ade80; box-shadow: 0 0 5px rgba(74,222,128,0.7); }
.status-dot.error    { background: #f87171; box-shadow: 0 0 5px rgba(248,113,113,0.7); }
</style>

