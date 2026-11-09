<template>
	<div class="settings-overlay" @click.self="$emit('close')">
		<div class="settings-panel">
			<div class="settings-header">
				<span class="settings-title">Settings</span>
				<button class="icon-btn icon-btn--close" @click="$emit('close')">
					<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
						<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
					</svg>
				</button>
			</div>

			<div class="settings-body">
				<!-- Opacity setting -->
				<div class="setting-block">
					<div class="setting-row">
						<label class="setting-label">Widget opacity</label>
						<span class="setting-value">{{ Math.round(localConfig.opacity * 100) }}%</span>
					</div>
					<input
						type="range" min="0.2" max="1" step="0.01"
						v-model.number="localConfig.opacity"
						class="input--range"
					/>
				</div>

				<!-- Always on top setting -->
				<label class="setting-block setting-row setting-toggle">
					<span class="setting-label">Always on top</span>
					<input type="checkbox" v-model="localConfig.always_on_top" class="input--switch" />
				</label>

				<div v-for="(project, pi) in localConfig.projects" :key="project.id" class="project-block">
					<!-- Project header -->
					<div class="project-row">
						<input
							v-model="project.name"
							class="input input--name"
							placeholder="Project name"
						/>
						<input
							v-model="project.color"
							type="color"
							class="input input--color"
							title="Color"
						/>
						<button class="btn-danger" @click="removeProject(pi)" title="Delete project">
							<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
								<polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6M14 11v6"/><path d="M9 6V4h6v2"/>
							</svg>
						</button>
					</div>

					<!-- Services -->
					<div v-for="(svc, si) in project.services" :key="svc.id" class="service-block">
						<div class="service-row">
							<input v-model="svc.name" class="input input--grow" placeholder="Service name" />
							<button class="btn-danger btn-sm" @click="removeService(pi, si)" title="Delete">
								<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
									<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
								</svg>
							</button>
						</div>
						<div class="service-row">
							<select v-model="svc.icon" class="input input--select input--grow" title="Icon">
								<option value="python">🐍 Python</option>
								<option value="node">🟢 Node</option>
								<option value="docker">🐳 Docker</option>
								<option value="shell">🚀 Shell</option>
							</select>
							<select
								v-model="svc.service_type"
								class="input input--select input--type input--grow"
								title="Shell: launch an external app without tracking or killing it"
							>
								<option value="">Normal</option>
								<option value="shell">Shell (fire &amp; forget)</option>
							</select>
						</div>
						<div class="service-row">
							<input v-model="svc.cwd" class="input input--cwd" placeholder="Working directory (cwd)" />
							<button class="btn-browse" @click="browseCwd(pi, si)" title="Browse">
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
								</svg>
								Browse
							</button>
						</div>
						<input v-model="svc.cmd" class="input input--cmd" placeholder="Command (e.g. uvicorn app:app --port 8000 --reload)" />
						<input v-model="svc.conda_env" class="input" placeholder="Conda env (optional)" />
					</div>

					<button class="btn-add" @click="addService(pi)">+ Add service</button>
				</div>

				<button class="btn-add btn-add--project" @click="addProject">+ Add project</button>
			</div>

			<div class="settings-footer">
				<span v-if="saveMsg" class="save-msg" :class="{ error: saveError }">{{ saveMsg }}</span>
				<div class="footer-btns">
					<button class="btn-cancel" @click="$emit('close')">Cancel</button>
					<button class="btn-save" @click="save" :disabled="saving">
						{{ saving ? 'Saving…' : 'Save' }}
					</button>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const props = defineProps({ config: { type: Object, required: true } });
const emit = defineEmits(["close", "saved"]);

// Deep clone to avoid mutating parent before save
const localConfig = ref({ opacity: 0.92, always_on_top: true, ...JSON.parse(JSON.stringify(props.config)) });
// A normal service has no service_type in the file: show it as "Normal" in the select
localConfig.value.projects.forEach(p => p.services.forEach(s => { s.service_type ??= ""; }));

const saving = ref(false);
const saveMsg = ref("");
const saveError = ref(false);

// ── Browse folder ─────────────────────────────────────────────────────────────
async function browseCwd(pi, si) {
	const selected = await open({ directory: true, multiple: false, title: "Choose the working directory" });
	if (selected) {
		localConfig.value.projects[pi].services[si].cwd = selected;
	}
}

// ── Add / remove ──────────────────────────────────────────────────────────────
function addProject() {
	const id = "project-" + Date.now();
	localConfig.value.projects.push({
		id, name: "New project", color: "#7c3aed", services: []
	});
}
function removeProject(pi) {
	localConfig.value.projects.splice(pi, 1);
}
function addService(pi) {
	const id = "svc-" + Date.now();
	localConfig.value.projects[pi].services.push({
		id, name: "New service", icon: "node", service_type: "", cmd: "", cwd: "", conda_env: ""
	});
}
function removeService(pi, si) {
	localConfig.value.projects[pi].services.splice(si, 1);
}

// ── Save ──────────────────────────────────────────────────────────────────────
async function save() {
	saving.value = true;
	saveMsg.value = "";
	try {
		// Clean up empty conda_env before saving
		const cleanConfig = JSON.parse(JSON.stringify(localConfig.value));
		cleanConfig.projects.forEach(p => p.services.forEach(s => {
			if (!s.conda_env) delete s.conda_env;
			if (!s.service_type) delete s.service_type;
		}));
		await invoke("save_config", { config: cleanConfig });
		saveMsg.value = "✓ Saved";
		saveError.value = false;
		emit("saved", cleanConfig);
		setTimeout(() => { saveMsg.value = ""; }, 2000);
	} catch (e) {
		saveMsg.value = "Error: " + e;
		saveError.value = true;
	} finally {
		saving.value = false;
	}
}
</script>

<style scoped>
/* ── Overlay ── */
.settings-overlay {
	position: fixed; inset: 0; z-index: 100;
	background: rgba(0,0,0,0.55);
	backdrop-filter: blur(4px);
	display: flex; align-items: stretch;
}

/* ── Panel ── */
.settings-panel {
	width: 100%; display: flex; flex-direction: column;
	background: rgba(12, 12, 24, 0.97);
	border: 1px solid rgba(255,255,255,0.08);
	overflow: hidden;
}

.settings-header {
	display: flex; align-items: center; justify-content: space-between;
	padding: 11px 14px;
	border-bottom: 1px solid rgba(255,255,255,0.07);
	flex-shrink: 0;
	-webkit-app-region: drag;
}
.settings-title { font-size: 12px; font-weight: 650; color: rgba(255,255,255,0.75); }

/* ── Body ── */
.settings-body {
	flex: 1; overflow-y: auto; overflow-x: hidden; padding: 10px;
	display: flex; flex-direction: column; gap: 10px;
	scrollbar-width: thin; scrollbar-color: rgba(124,58,237,0.2) transparent;
}
.settings-body::-webkit-scrollbar { width: 3px; }
.settings-body::-webkit-scrollbar-thumb { background: rgba(124,58,237,0.25); border-radius: 2px; }

/* ── Opacity setting ── */
.setting-block {
	background: rgba(255,255,255,0.03);
	border: 1px solid rgba(255,255,255,0.07);
	border-radius: 10px; padding: 9px 10px;
	display: flex; flex-direction: column; gap: 6px;
}
.setting-row { display: flex; justify-content: space-between; align-items: center; }
.setting-label { font-size: 11px; color: rgba(255,255,255,0.55); font-weight: 600; }
.setting-value { font-size: 11px; color: #a78bfa; font-weight: 700; }
.input--range {
	width: 100%; height: 4px; cursor: pointer;
	accent-color: #7c3aed;
}
.setting-toggle { flex-direction: row; cursor: pointer; }

/* Toggle switch */
.input--switch {
	appearance: none; -webkit-appearance: none;
	width: 28px; height: 16px; flex-shrink: 0;
	border-radius: 8px; position: relative; cursor: pointer;
	background: rgba(255,255,255,0.12);
	border: 1px solid rgba(255,255,255,0.15);
	transition: background 0.18s, border-color 0.18s;
}
.input--switch::after {
	content: ""; position: absolute; top: 2px; left: 2px;
	width: 10px; height: 10px; border-radius: 50%;
	background: rgba(255,255,255,0.6);
	transition: transform 0.18s, background 0.18s;
}
.input--switch:checked { background: rgba(124,58,237,0.6); border-color: rgba(124,58,237,0.8); }
.input--switch:checked::after { transform: translateX(12px); background: #fff; }

/* ── Project block ── */
.project-block {
	background: rgba(255,255,255,0.03);
	border: 1px solid rgba(255,255,255,0.07);
	border-radius: 10px; padding: 9px; display: flex; flex-direction: column; gap: 7px;
}
.project-row {
	display: flex; gap: 6px; align-items: center;
}

/* ── Service block ── */
.service-block {
	background: rgba(0,0,0,0.2);
	border-radius: 8px; padding: 8px;
	display: flex; flex-direction: column; gap: 5px;
	border: 1px solid rgba(255,255,255,0.05);
}
.service-row { display: flex; gap: 5px; align-items: center; }

/* ── Inputs ── */
.input {
	background: rgba(255,255,255,0.05);
	border: 1px solid rgba(255,255,255,0.1);
	border-radius: 6px; padding: 5px 8px;
	color: rgba(255,255,255,0.8); font-size: 11px;
	font-family: inherit; outline: none; width: 100%; min-width: 0;
	transition: border-color 0.15s;
}
.input:focus { border-color: rgba(124,58,237,0.5); }
.input--name  { flex: 1; font-weight: 600; }
.input--color { width: 32px; height: 28px; padding: 2px; cursor: pointer; flex-shrink: 0; border-radius: 6px; }
.input--select { width: auto; flex-shrink: 0; cursor: pointer; background: rgba(20, 20, 35, 0.95); color: rgba(255,255,255,0.8); }
.input--select option { background: #14141f; color: rgba(255,255,255,0.85); }
.input--type { font-size: 10px; color: rgba(167,139,250,0.85); }
.input--grow { flex: 1 1 0; width: auto; }
.input--cwd { flex: 1; font-size: 10px; color: rgba(255,255,255,0.5); }
.input--cmd { font-family: "Cascadia Code", "Consolas", monospace; font-size: 10px; }

/* ── Buttons ── */
.btn-browse {
	display: flex; align-items: center; gap: 4px;
	background: rgba(124,58,237,0.12); border: 1px solid rgba(124,58,237,0.25);
	border-radius: 6px; padding: 4px 8px; color: #a78bfa;
	font-size: 10px; font-family: inherit; cursor: pointer; flex-shrink: 0;
	transition: all 0.15s; white-space: nowrap;
}
.btn-browse:hover { background: rgba(124,58,237,0.22); border-color: rgba(124,58,237,0.45); }

.btn-add {
	background: rgba(255,255,255,0.04); border: 1px dashed rgba(255,255,255,0.15);
	border-radius: 7px; padding: 6px; color: rgba(255,255,255,0.35);
	font-size: 10px; font-family: inherit; cursor: pointer; width: 100%;
	transition: all 0.15s;
}
.btn-add:hover { background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.6); }
.btn-add--project {
	border-color: rgba(124,58,237,0.25); color: rgba(167,139,250,0.6);
}
.btn-add--project:hover { border-color: rgba(124,58,237,0.5); color: #a78bfa; }

.btn-danger {
	width: 26px; height: 26px; flex-shrink: 0;
	background: rgba(220,38,38,0.08); border: 1px solid rgba(220,38,38,0.15);
	border-radius: 6px; color: rgba(248,113,113,0.5);
	cursor: pointer; display: flex; align-items: center; justify-content: center;
	transition: all 0.15s;
}
.btn-danger:hover { background: rgba(220,38,38,0.2); color: #fca5a5; border-color: rgba(220,38,38,0.4); }
.btn-sm { width: 22px; height: 22px; }

/* ── Footer ── */
.settings-footer {
	display: flex; align-items: center; justify-content: space-between;
	padding: 9px 14px; border-top: 1px solid rgba(255,255,255,0.07);
	flex-shrink: 0;
}
.footer-btns { display: flex; gap: 6px; margin-left: auto; }
.save-msg { font-size: 10px; color: #4ade80; }
.save-msg.error { color: #f87171; }

.btn-cancel {
	background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);
	border-radius: 7px; padding: 5px 12px; color: rgba(255,255,255,0.4);
	font-size: 11px; font-family: inherit; cursor: pointer; transition: all 0.15s;
}
.btn-cancel:hover { background: rgba(255,255,255,0.1); color: rgba(255,255,255,0.7); }

.btn-save {
	background: linear-gradient(135deg, rgba(124,58,237,0.7), rgba(109,40,217,0.9));
	border: 1px solid rgba(124,58,237,0.4);
	border-radius: 7px; padding: 5px 14px; color: #fff;
	font-size: 11px; font-weight: 600; font-family: inherit; cursor: pointer;
	transition: all 0.15s;
}
.btn-save:hover:not(:disabled) { background: linear-gradient(135deg, rgba(124,58,237,0.9), rgba(109,40,217,1)); }
.btn-save:disabled { opacity: 0.5; cursor: default; }

/* icon-btn (same as App.vue) */
.icon-btn {
	width: 26px; height: 26px;
	display: flex; align-items: center; justify-content: center;
	background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);
	border-radius: 8px; color: rgba(255,255,255,0.35);
	cursor: pointer; transition: all 0.18s; -webkit-app-region: no-drag;
}
.icon-btn--close:hover {
	background: rgba(220,38,38,0.2); color: #fca5a5; border-color: rgba(220,38,38,0.35);
}
</style>
