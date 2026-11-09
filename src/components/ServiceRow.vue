<template>
	<div class="service-row" :class="[`status--${service.status}`, { expanded: showLogs }]">
		<!-- Main row -->
		<div class="row-main">
			<!-- Icon -->
			<div class="svc-icon" :class="`icon--${service.icon}`">
				<svg v-if="service.icon === 'python'" viewBox="0 0 24 24" fill="currentColor">
					<path d="M11.9 2C9.3 2 7.5 2.8 7.5 4.2v1.6h4.5v.5H5.8C4.2 6.3 3 7.7 3 9.8s1 3.4 2.8 3.7l.5.1h1V12c0-2 1.7-3.5 4.6-3.5h4.4c1.6 0 2.7-1.1 2.7-2.7V4.2C19 2.8 17.2 2 14.6 2h-2.7zm-1.5 1.5c.5 0 .9.4.9.9s-.4.9-.9.9-.9-.4-.9-.9.4-.9.9-.9z"/>
					<path d="M20.2 6.3h-1v1.6C19.2 10 17.5 11.5 14.6 11.5H10.2c-1.6 0-2.7 1.1-2.7 2.7v2.6c0 1.4 1.8 2.2 4.4 2.2h2.6c2.6 0 4.4-.8 4.4-2.2v-1.6h-4.5v-.5h6.2c1.6 0 2.8-1.4 2.8-3.5s-1-3.4-2.8-3.7l-.4-.2zm-4.8 8.7c.5 0 .9.4.9.9s-.4.9-.9.9-.9-.4-.9-.9.4-.9.9-.9z"/>
				</svg>
				<svg v-else-if="service.icon === 'node'" viewBox="0 0 24 24" fill="currentColor">
					<path d="M12 2L2 7v10l10 5 10-5V7L12 2zm0 2.3l7.4 3.7-7.4 3.7L4.6 8 12 4.3zM4 9.7l7 3.5v7L4 16.7V9.7zm9 10.5v-7l7-3.5v7l-7 3.5z"/>
				</svg>
				<svg v-else-if="service.icon === 'docker'" viewBox="0 0 24 24" fill="currentColor">
					<path d="M13.983 11.078h2.119a.186.186 0 00.186-.185V9.006a.186.186 0 00-.186-.186h-2.119a.185.185 0 00-.185.185v1.888c0 .102.083.185.185.185m-2.954-5.43h2.118a.186.186 0 00.186-.186V3.574a.186.186 0 00-.186-.185h-2.118a.185.185 0 00-.185.185v1.888c0 .102.082.185.185.185m0 2.716h2.118a.187.187 0 00.186-.186V6.29a.186.186 0 00-.186-.185h-2.118a.185.185 0 00-.185.185v1.887c0 .102.082.186.185.186m-2.93 0h2.12a.186.186 0 00.184-.186V6.29a.185.185 0 00-.185-.185H8.1a.185.185 0 00-.185.185v1.887c0 .102.083.186.185.186m-2.964 0h2.119a.186.186 0 00.185-.186V6.29a.185.185 0 00-.185-.185H5.136a.186.186 0 00-.186.185v1.887c0 .102.084.186.186.186m5.893 2.715h2.118a.186.186 0 00.186-.185V9.006a.186.186 0 00-.186-.186h-2.118a.185.185 0 00-.185.185v1.888c0 .102.082.185.185.185m-2.93 0h2.12a.185.185 0 00.184-.185V9.006a.185.185 0 00-.184-.186h-2.12a.185.185 0 00-.184.185v1.888c0 .102.083.185.185.185m-2.964 0h2.119a.185.185 0 00.185-.185V9.006a.185.185 0 00-.184-.186h-2.12a.186.186 0 00-.186.185v1.888c0 .102.084.185.186.185m-2.92 0h2.12a.186.186 0 00.184-.185V9.006a.185.185 0 00-.184-.186h-2.12a.185.185 0 00-.185.186v1.887c0 .102.083.185.185.185M23.763 9.89c-.065-.051-.672-.51-1.954-.51-.338.001-.676.03-1.01.087-.248-1.7-1.653-2.53-1.716-2.566l-.344-.199-.226.327c-.284.438-.49.922-.612 1.43-.23.97-.09 1.882.403 2.661-.595.332-1.55.413-1.744.42H.751a.751.751 0 00-.75.748 11.376 11.376 0 00.692 4.062c.545 1.428 1.355 2.48 2.41 3.124 1.18.723 3.1 1.137 5.275 1.137.983.003 1.963-.086 2.93-.266a12.248 12.248 0 003.823-1.389c.98-.567 1.86-1.288 2.61-2.136 1.252-1.418 1.998-2.997 2.553-4.4h.221c1.372 0 2.215-.549 2.68-1.009.309-.293.55-.65.707-1.046l.098-.288z"/>
				</svg>
				<!-- Shell icon (rocket) -->
				<svg v-else-if="service.icon === 'shell'" viewBox="0 0 24 24" fill="currentColor">
					<path d="M13.13 22.19L11.5 18.36C13.07 17.78 14.54 17 15.9 16.09L13.13 22.19M5.64 12.5L1.81 10.87L7.91 8.1C7 9.46 6.22 10.93 5.64 12.5M21.61 2.39C21.61 2.39 16.66 .269 11 5.93C8.81 8.12 7.5 10.53 6.65 12.64C6.37 13.39 6.56 14.21 7.11 14.77L9.24 16.89C9.79 17.45 10.61 17.63 11.36 17.36C13.5 16.5 15.88 15.19 18.07 13C23.73 7.34 21.61 2.39 21.61 2.39M14.54 9.46C13.76 8.68 13.76 7.41 14.54 6.63S16.59 5.85 17.37 6.63C18.14 7.41 18.15 8.68 17.37 9.46C16.59 10.24 15.32 10.24 14.54 9.46M8.88 16.53L7.47 15.12L8.88 16.53M6.24 22L9.88 18.36C9.54 18.27 9.21 18.12 8.91 17.91L4.83 22H6.24M2 22H3.41L8.18 17.24L6.76 15.83L2 20.59V22M2 19.17L6.09 15.09C5.88 14.79 5.73 14.46 5.64 14.12L2 17.76V19.17Z"/>
				</svg>
			</div>

			<!-- Name + status -->
			<div class="svc-info">
				<span class="svc-name">{{ service.name }}</span>
				<span class="svc-status">
					<span class="status-dot"></span>
					{{ statusLabel }}
				</span>
			</div>

			<!-- Actions -->
			<div class="svc-actions">
				<!-- Shell: single launch button -->
				<button
					v-if="isShell"
					class="btn-power btn-launch"
					:class="service.status === 'launched' ? 'power--launched' : 'power--stopped'"
					@click="$emit('toggle', service.id)"
					:title="service.status === 'launched' ? 'Mark as stopped' : 'Launch'"
				>
					<svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor">
						<path d="M13.13 22.19L11.5 18.36C13.07 17.78 14.54 17 15.9 16.09L13.13 22.19M5.64 12.5L1.81 10.87L7.91 8.1C7 9.46 6.22 10.93 5.64 12.5M21.61 2.39C21.61 2.39 16.66 .269 11 5.93C8.81 8.12 7.5 10.53 6.65 12.64C6.37 13.39 6.56 14.21 7.11 14.77L9.24 16.89C9.79 17.45 10.61 17.63 11.36 17.36C13.5 16.5 15.88 15.19 18.07 13C23.73 7.34 21.61 2.39 21.61 2.39M14.54 9.46C13.76 8.68 13.76 7.41 14.54 6.63S16.59 5.85 17.37 6.63C18.14 7.41 18.15 8.68 17.37 9.46C16.59 10.24 15.32 10.24 14.54 9.46M8.88 16.53L7.47 15.12L8.88 16.53M6.24 22L9.88 18.36C9.54 18.27 9.21 18.12 8.91 17.91L4.83 22H6.24M2 22H3.41L8.18 17.24L6.76 15.83L2 20.59V22M2 19.17L6.09 15.09C5.88 14.79 5.73 14.46 5.64 14.12L2 17.76V19.17Z"/>
					</svg>
				</button>

				<!-- Normal: Start / Stop button -->
				<template v-else>
					<button
						class="btn-power"
						:class="`power--${service.status}`"
						@click="$emit('toggle', service.id)"
						:title="service.status === 'running' ? 'Stop' : 'Start'"
					>
						<svg v-if="service.status !== 'running'" width="11" height="11" viewBox="0 0 24 24" fill="currentColor">
							<polygon points="5,3 19,12 5,21"/>
						</svg>
						<svg v-else width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
							<rect x="4" y="4" width="16" height="16" rx="2"/>
						</svg>
					</button>

					<!-- Logs button (only for non-shell) -->
					<button
						class="btn-logs"
						:class="{ active: showLogs }"
						@click="showLogs = !showLogs"
						title="Logs"
					>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<line x1="3" y1="6" x2="21" y2="6"/>
							<line x1="3" y1="12" x2="21" y2="12"/>
							<line x1="3" y1="18" x2="15" y2="18"/>
						</svg>
					</button>
				</template>
			</div>
		</div>

		<!-- Log panel (only for non-shell services) -->
		<transition name="logs">
			<div v-if="showLogs && !isShell" class="log-panel">
				<div class="log-header">
					<span class="log-title">stdout</span>
					<div class="log-search-wrap">
						<svg class="log-search-icon" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
							<circle cx="11" cy="11" r="7"/><line x1="21" y1="21" x2="16.5" y2="16.5"/>
						</svg>
						<input
							v-model="logSearch"
							class="log-search"
							type="text"
							placeholder="Filter…"
							spellcheck="false"
						/>
						<button v-if="logSearch" class="log-search-clear" @click="logSearch = ''" title="Clear">×</button>
					</div>
					<button class="log-expand" @click="showLogsModal = true" title="Expand">
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/>
						</svg>
					</button>
					<button class="log-clear" @click="$emit('clearLogs', service.id)">clear</button>
				</div>
				<div class="log-body" ref="logBodyRef">
					<div v-if="!service.logs?.length" class="log-empty">No logs yet.</div>
					<div v-else-if="!filteredLogs.length" class="log-empty">No line matches “{{ logSearch }}”.</div>
					<div v-for="(line, i) in filteredLogs" :key="i" class="log-line">
						<span class="log-time">{{ line.time }}</span>
						<span class="log-text" :class="line.level" v-html="highlight(line.text)"></span>
					</div>
				</div>
			</div>
		</transition>

		<!-- Fullscreen log modal -->
		<teleport to="body">
			<transition name="modal">
				<div v-if="showLogsModal" class="log-modal-overlay" @mousedown.self="showLogsModal = false">
					<div class="log-modal">
						<div class="log-modal-header">
							<div class="log-modal-title">
								<span class="svc-icon-mini" :class="`icon--${service.icon}`"></span>
								<span>{{ service.name }}</span>
								<span class="log-modal-sub">stdout</span>
							</div>
							<div class="log-search-wrap log-search-wrap--modal">
								<svg class="log-search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
									<circle cx="11" cy="11" r="7"/><line x1="21" y1="21" x2="16.5" y2="16.5"/>
								</svg>
								<input
									ref="modalSearchRef"
									v-model="logSearch"
									class="log-search"
									type="text"
									placeholder="Filter lines…"
									spellcheck="false"
								/>
								<button v-if="logSearch" class="log-search-clear" @click="logSearch = ''" title="Clear">×</button>
							</div>
							<button class="log-clear" @click="$emit('clearLogs', service.id)">clear</button>
							<button class="log-modal-close" @click="showLogsModal = false" title="Close">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
									<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
								</svg>
							</button>
						</div>
						<div class="log-modal-body" ref="modalLogBodyRef">
							<div v-if="!service.logs?.length" class="log-empty">No logs yet.</div>
							<div v-else-if="!filteredLogs.length" class="log-empty">No line matches “{{ logSearch }}”.</div>
							<div v-for="(line, i) in filteredLogs" :key="i" class="log-line">
								<span class="log-time">{{ line.time }}</span>
								<span class="log-text" :class="line.level" v-html="highlight(line.text)"></span>
							</div>
						</div>
						<div class="log-modal-footer">
							{{ filteredLogs.length }}<template v-if="logSearch"> / {{ service.logs?.length || 0 }}</template> line{{ filteredLogs.length === 1 ? '' : 's' }}
						</div>
					</div>
				</div>
			</transition>
		</teleport>
	</div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onUnmounted } from "vue";

const props = defineProps({
	service: { type: Object, required: true },
});
defineEmits(["toggle", "clearLogs"]);

const showLogs = ref(false);
const showLogsModal = ref(false);
const logSearch = ref("");
const logBodyRef = ref(null);
const modalLogBodyRef = ref(null);
const modalSearchRef = ref(null);

const isShell = computed(() => props.service.service_type === "shell");

// Filters log lines on the search term (case-insensitive, on the text)
const filteredLogs = computed(() => {
	const logs = props.service.logs ?? [];
	const q = logSearch.value.trim().toLowerCase();
	if (!q) return logs;
	return logs.filter(l => (l.text ?? "").toLowerCase().includes(q));
});

// Renders a log line with the search term highlighted.
// HTML is always escaped so that logs cannot inject markup.
function escapeHtml(s) {
	return String(s).replace(/[&<>"']/g, c => (
		{ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]
	));
}
function highlight(text) {
	const raw = text ?? "";
	const q = logSearch.value.trim();
	if (!q) return escapeHtml(raw);
	const re = new RegExp(q.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "gi");
	let out = "", last = 0, m;
	while ((m = re.exec(raw)) !== null) {
		out += escapeHtml(raw.slice(last, m.index));
		out += `<mark class="log-hl">${escapeHtml(m[0])}</mark>`;
		last = m.index + m[0].length;
		if (m.index === re.lastIndex) re.lastIndex++; // guard against empty matches
	}
	return out + escapeHtml(raw.slice(last));
}

const statusLabel = computed(() => {
	switch (props.service.status) {
		case "running":  return "Running";
		case "launched": return "Launched";
		case "stopped":  return "Stopped";
		case "error":    return "Error";
		case "starting": return "Starting…";
		default: return "Stopped";
	}
});

// Auto-scroll logs to bottom when new lines arrive (inline + modal)
watch(() => props.service.logs?.length, async () => {
	await nextTick();
	if (showLogs.value && logBodyRef.value) logBodyRef.value.scrollTop = logBodyRef.value.scrollHeight;
	if (showLogsModal.value && modalLogBodyRef.value) modalLogBodyRef.value.scrollTop = modalLogBodyRef.value.scrollHeight;
});

// Scroll to bottom when opening the modal + close on Escape
function onKeydown(e) {
	if (e.key === "Escape" && showLogsModal.value) showLogsModal.value = false;
}
watch(showLogsModal, async (open) => {
	if (open) {
		window.addEventListener("keydown", onKeydown);
		await nextTick();
		if (modalLogBodyRef.value) modalLogBodyRef.value.scrollTop = modalLogBodyRef.value.scrollHeight;
		modalSearchRef.value?.focus();
	} else {
		window.removeEventListener("keydown", onKeydown);
	}
});
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<style scoped>
.service-row {
	border-radius: 10px;
	overflow: hidden;
	background: rgba(255,255,255,0.03);
	border: 1px solid rgba(255,255,255,0.07);
	transition: all 0.2s ease;
	position: relative;
}
/* Left accent bar — color depends on status */
.service-row::before {
	content: "";
	position: absolute;
	left: 0; top: 0; bottom: 0;
	width: 2px;
	background: rgba(255,255,255,0.08);
	transition: background 0.3s;
}
.status--running::before  { background: #4ade80; box-shadow: 0 0 8px rgba(74,222,128,0.6); }
.status--launched::before { background: #818cf8; box-shadow: 0 0 8px rgba(129,140,248,0.6); }
.status--error::before    { background: #f87171; box-shadow: 0 0 8px rgba(248,113,113,0.6); }
.status--starting::before { background: #facc15; box-shadow: 0 0 8px rgba(250,204,21,0.5); }

.row-main {
	display: flex;
	align-items: center;
	gap: 10px;
	padding: 9px 10px 9px 14px;
}

/* Icon */
.svc-icon {
	width: 28px; height: 28px;
	border-radius: 8px;
	display: flex; align-items: center; justify-content: center;
	flex-shrink: 0;
}
.svc-icon svg { width: 15px; height: 15px; }
.icon--python { background: rgba(59,130,246,0.15); color: #93c5fd; }
.icon--node   { background: rgba(34,197,94,0.15);  color: #86efac; }
.icon--docker { background: rgba(14,165,233,0.15); color: #7dd3fc; }
.icon--shell  { background: rgba(129,140,248,0.15); color: #818cf8; }

/* Info */
.svc-info {
	flex: 1; min-width: 0;
	display: flex; flex-direction: column; gap: 2px;
}
.svc-name {
	font-size: 11px; font-weight: 550;
	color: rgba(255,255,255,0.78);
	white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.svc-status {
	display: flex; align-items: center; gap: 4px;
	font-size: 9px; color: rgba(255,255,255,0.28);
	text-transform: uppercase; letter-spacing: 0.4px;
}
.status--running .svc-status  { color: #4ade80; }
.status--launched .svc-status { color: #818cf8; }
.status--error   .svc-status  { color: #f87171; }
.status--starting .svc-status { color: #facc15; }

/* Status dot */
.status-dot {
	width: 5px; height: 5px;
	border-radius: 50%;
	background: rgba(255,255,255,0.2);
	flex-shrink: 0;
}
.status--running .status-dot {
	background: #4ade80;
	box-shadow: 0 0 5px rgba(74,222,128,0.8);
	animation: pulse-dot 2.5s ease-in-out infinite;
}
.status--launched .status-dot {
	background: #818cf8;
	box-shadow: 0 0 5px rgba(129,140,248,0.8);
}
.status--error .status-dot {
	background: #f87171;
	box-shadow: 0 0 5px rgba(248,113,113,0.8);
}
.status--starting .status-dot {
	background: #facc15;
	animation: pulse-dot 1s ease-in-out infinite;
}
@keyframes pulse-dot {
	0%,100% { opacity: 1; }
	50% { opacity: 0.4; }
}

/* Actions */
.svc-actions { display: flex; gap: 5px; flex-shrink: 0; }

.btn-power {
	width: 26px; height: 26px;
	border-radius: 8px;
	border: 1px solid rgba(255,255,255,0.08);
	background: rgba(255,255,255,0.05);
	color: rgba(255,255,255,0.4);
	cursor: pointer;
	display: flex; align-items: center; justify-content: center;
	transition: all 0.18s;
}
.btn-power:hover { background: rgba(255,255,255,0.12); color: rgba(255,255,255,0.9); }
.power--stopped:hover, .power--error:hover {
	background: rgba(74,222,128,0.15) !important;
	border-color: rgba(74,222,128,0.3) !important;
	color: #4ade80 !important;
	box-shadow: 0 0 12px rgba(74,222,128,0.15) !important;
}
.power--launched {
	background: rgba(129,140,248,0.15);
	border-color: rgba(129,140,248,0.3);
	color: #818cf8;
}
.power--launched:hover {
	background: rgba(129,140,248,0.25) !important;
	border-color: rgba(129,140,248,0.5) !important;
	box-shadow: 0 0 12px rgba(129,140,248,0.2) !important;
}
.power--running {
	background: rgba(248,113,113,0.1);
	border-color: rgba(248,113,113,0.2);
	color: #fca5a5;
}
.power--running:hover {
	background: rgba(248,113,113,0.22) !important;
	border-color: rgba(248,113,113,0.4) !important;
	box-shadow: 0 0 12px rgba(248,113,113,0.15) !important;
}
.power--starting {
	background: rgba(250,204,21,0.1);
	border-color: rgba(250,204,21,0.2);
	color: #fde68a;
	cursor: wait;
}

.btn-logs {
	width: 26px; height: 26px;
	border-radius: 8px;
	border: 1px solid rgba(255,255,255,0.07);
	background: rgba(255,255,255,0.04);
	color: rgba(255,255,255,0.25);
	cursor: pointer;
	display: flex; align-items: center; justify-content: center;
	transition: all 0.18s;
}
.btn-logs:hover, .btn-logs.active {
	background: rgba(124,58,237,0.18);
	border-color: rgba(124,58,237,0.3);
	color: #a78bfa;
}

/* Log panel */
.log-panel {
	border-top: 1px solid rgba(255,255,255,0.05);
	background: rgba(0,0,0,0.25);
	user-select: text;
}
.log-header {
	display: flex; align-items: center; gap: 8px;
	padding: 5px 10px 4px 14px;
	font-size: 9px; color: rgba(255,255,255,0.2);
	text-transform: uppercase; letter-spacing: 0.5px;
	border-bottom: 1px solid rgba(255,255,255,0.04);
}
.log-title { flex-shrink: 0; }
.log-clear {
	background: none; border: none;
	color: rgba(255,255,255,0.2); cursor: pointer;
	font-size: 9px; font-family: inherit;
	transition: color 0.15s; padding: 0;
	flex-shrink: 0;
}
.log-clear:hover { color: rgba(255,255,255,0.5); }

/* Search field */
.log-search-wrap {
	display: flex; align-items: center; gap: 5px;
	flex: 1; min-width: 0;
	background: rgba(0,0,0,0.25);
	border: 1px solid rgba(255,255,255,0.07);
	border-radius: 6px;
	padding: 2px 6px;
	transition: border-color 0.15s;
}
.log-search-wrap:focus-within { border-color: rgba(124,58,237,0.45); }
.log-search-icon { color: rgba(255,255,255,0.25); flex-shrink: 0; }
.log-search {
	flex: 1; min-width: 0;
	background: none; border: none; outline: none;
	color: rgba(255,255,255,0.75);
	font-family: inherit; font-size: 10px;
	text-transform: none; letter-spacing: 0;
}
.log-search::placeholder { color: rgba(255,255,255,0.25); }
.log-search-clear {
	background: none; border: none; padding: 0;
	color: rgba(255,255,255,0.35); cursor: pointer;
	font-size: 14px; line-height: 1; flex-shrink: 0;
	transition: color 0.15s;
}
.log-search-clear:hover { color: rgba(255,255,255,0.75); }

/* Expand-to-modal button */
.log-expand {
	flex-shrink: 0;
	width: 22px; height: 22px;
	border-radius: 6px;
	border: 1px solid rgba(255,255,255,0.07);
	background: rgba(255,255,255,0.04);
	color: rgba(255,255,255,0.3);
	cursor: pointer;
	display: flex; align-items: center; justify-content: center;
	transition: all 0.15s;
}
.log-expand:hover {
	background: rgba(124,58,237,0.18);
	border-color: rgba(124,58,237,0.3);
	color: #a78bfa;
}
.log-body {
	min-height: 60px;
	max-height: 400px;
	height: 120px;
	resize: vertical;
	overflow-y: auto;
	padding: 6px 10px 7px 14px;
	scrollbar-width: thin;
	scrollbar-color: rgba(124,58,237,0.2) transparent;
	user-select: text;
	cursor: text;
}
.log-body::-webkit-scrollbar { width: 3px; }
.log-body::-webkit-scrollbar-thumb { background: rgba(124,58,237,0.25); border-radius: 2px; }
.log-empty { font-size: 10px; color: rgba(255,255,255,0.15); font-style: italic; }
.log-line { display: flex; gap: 8px; font-size: 10px; line-height: 1.6; }
.log-time { color: rgba(255,255,255,0.18); flex-shrink: 0; font-variant-numeric: tabular-nums; user-select: text; }
.log-text { color: rgba(255,255,255,0.5); word-break: break-all; font-family: "Cascadia Code","Consolas",monospace; user-select: text; }
.log-text.error { color: #fca5a5; }
.log-text.warn  { color: #fde68a; }

/* Search match highlight */
.log-hl {
	background: rgba(124,58,237,0.45);
	color: #fff;
	border-radius: 2px;
	padding: 0 1px;
	box-shadow: 0 0 0 1px rgba(167,139,250,0.4);
}

/* Slide animation */
.logs-enter-active, .logs-leave-active { transition: all 0.22s cubic-bezier(.4,0,.2,1); }
.logs-enter-from, .logs-leave-to { opacity: 0; transform: translateY(-6px); max-height: 0; }

/* ── Fullscreen log modal ── */
.log-modal-overlay {
	position: fixed; inset: 0; z-index: 1000;
	display: flex; align-items: center; justify-content: center;
	padding: 32px;
	background: rgba(0,0,0,0.6);
	backdrop-filter: blur(6px);
	-webkit-backdrop-filter: blur(6px);
	-webkit-app-region: no-drag;
}
.log-modal {
	display: flex; flex-direction: column;
	width: 100%; max-width: 900px; height: 100%; max-height: 640px;
	background: rgba(14,14,24,0.98);
	border: 1px solid rgba(255,255,255,0.1);
	border-radius: 14px;
	box-shadow: 0 24px 80px rgba(0,0,0,0.7), inset 0 1px 0 rgba(255,255,255,0.06);
	overflow: hidden;
}
.log-modal-header {
	display: flex; align-items: center; gap: 10px;
	padding: 10px 12px;
	border-bottom: 1px solid rgba(255,255,255,0.07);
	background: rgba(255,255,255,0.03);
	flex-shrink: 0;
}
.log-modal-title {
	display: flex; align-items: center; gap: 7px;
	font-size: 12px; font-weight: 600;
	color: rgba(255,255,255,0.85);
	flex-shrink: 0;
}
.svc-icon-mini { width: 8px; height: 8px; border-radius: 50%; background: currentColor; }
.log-modal-sub {
	font-size: 9px; font-weight: 500;
	color: rgba(255,255,255,0.25);
	text-transform: uppercase; letter-spacing: 0.5px;
}
.log-search-wrap--modal { flex: 1; max-width: 340px; padding: 4px 8px; }
.log-modal-close {
	flex-shrink: 0;
	width: 26px; height: 26px;
	border-radius: 8px;
	border: 1px solid rgba(255,255,255,0.08);
	background: rgba(255,255,255,0.05);
	color: rgba(255,255,255,0.4);
	cursor: pointer;
	display: flex; align-items: center; justify-content: center;
	transition: all 0.15s;
	margin-left: auto;
}
.log-modal-close:hover {
	background: rgba(220,38,38,0.2);
	border-color: rgba(220,38,38,0.35);
	color: #fca5a5;
}
.log-modal-body {
	flex: 1; min-height: 0;
	overflow-y: auto;
	padding: 10px 14px;
	background: rgba(0,0,0,0.3);
	user-select: text; cursor: text;
	scrollbar-width: thin;
	scrollbar-color: rgba(124,58,237,0.25) transparent;
}
.log-modal-body::-webkit-scrollbar { width: 5px; }
.log-modal-body::-webkit-scrollbar-thumb { background: rgba(124,58,237,0.3); border-radius: 3px; }
.log-modal-body .log-line { font-size: 11px; line-height: 1.7; }
.log-modal-footer {
	flex-shrink: 0;
	padding: 6px 14px;
	font-size: 9px; color: rgba(255,255,255,0.3);
	text-transform: uppercase; letter-spacing: 0.5px;
	border-top: 1px solid rgba(255,255,255,0.05);
	background: rgba(0,0,0,0.15);
}

/* Modal animation */
.modal-enter-active, .modal-leave-active { transition: opacity 0.18s ease; }
.modal-enter-active .log-modal, .modal-leave-active .log-modal { transition: transform 0.18s cubic-bezier(.4,0,.2,1); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-from .log-modal, .modal-leave-to .log-modal { transform: scale(0.97); }
</style>
