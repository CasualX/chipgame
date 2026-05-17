import { createWasmAPI } from "./shade.js";

const INPUT_UP = 1 << 0;
const INPUT_LEFT = 1 << 1;
const INPUT_DOWN = 1 << 2;
const INPUT_RIGHT = 1 << 3;
const INPUT_A = 1 << 4;
const INPUT_B = 1 << 5;
const INPUT_START = 1 << 6;
const INPUT_SELECT = 1 << 7;

function average(values) {
	if (!values.length) return 0;
	let sum = 0;
	for (const value of values) sum += value;
	return sum / values.length;
}

function percentile(values, ratio) {
	if (!values.length) return 0;
	const sorted = values.slice().sort((a, b) => a - b);
	const index = Math.min(sorted.length - 1, Math.max(0, Math.floor((sorted.length - 1) * ratio)));
	return sorted[index];
}

window.chipGame = function chipGame() {
	return {
		loadingStatus: "Starting...",
		loadingProgress: 0,
		hudText: "",
		showDisplayModeCta: false,
		detectedControlScheme: "keyboard",
		selectedControlScheme: "keyboard",
		userSelectedControlScheme: false,
		isLandscapeLayout: false,
		loadingPanelScale: 1,
		loadingPanelFrameWidth: 620,
		loadingPanelFrameHeight: 0,
		hasTouchSupport: false,
		audioEnabled: true,
		touchEnabled: false,
		touchPressed: {
			up: false,
			left: false,
			down: false,
			right: false,
			a: false,
			b: false,
			start: false,
			select: false,
		},
		padDir: null,
		gameActive: false,
		pressedKeys: new Set(),
		padTouchId: null,
		audioCtx: null,
		audioLoaded: false,
		soundBank: new Map(),
		musicBank: new Map(),
		musicSource: null,
		currentMusicKey: null,
		requestedMusicKey: null,
		wasmExports: null,
		wasmGamePtr: 0,
		frameHandle: 0,
		initialized: false,
		cleanupRegistered: false,
		perfEnabled: false,
		perfPanel: null,
		perfStats: null,
		perfObservers: [],

		init() {
			if (this.initialized) return;
			this.initialized = true;
			this.perfEnabled = new URLSearchParams(window.location.search).has("perf");
			this.setupPerfDebug();
			this.syncViewportLayout();
			this.hasTouchSupport = this.detectTouchSupport();
			this.syncControlSchemeFromEnvironment();
			this.boundWindowError = (event) => this.handleWindowError(event);
			this.boundUnhandledRejection = (event) => this.handleUnhandledRejection(event);
			this.boundKeyDown = (event) => this.handleKeyDown(event);
			this.boundKeyUp = (event) => this.handleKeyUp(event);
			this.boundGamepadConnected = () => this.handleGamepadChange();
			this.boundGamepadDisconnected = () => this.handleGamepadChange();
			this.boundResize = () => this.syncViewportLayout();
			this.boundFullscreenChange = () => this.updateDisplayModeCta();
			this.boundOrientationChange = () => this.syncViewportLayout();

			window.addEventListener("error", this.boundWindowError);
			window.addEventListener("unhandledrejection", this.boundUnhandledRejection);
			window.addEventListener("keydown", this.boundKeyDown);
			window.addEventListener("keyup", this.boundKeyUp);
			window.addEventListener("gamepadconnected", this.boundGamepadConnected);
			window.addEventListener("gamepaddisconnected", this.boundGamepadDisconnected);
			window.addEventListener("resize", this.boundResize);
			document.addEventListener("fullscreenchange", this.boundFullscreenChange);
			if (screen && screen.orientation && screen.orientation.addEventListener) {
				screen.orientation.addEventListener("change", this.boundOrientationChange);
			}

			this.updateDisplayModeCta();
			this.load().catch((err) => {
				console.error(err);
				this.setLoadingStatus(String(err && err.message ? err.message : err));
			});
		},

		setupPerfDebug() {
			if (!this.perfEnabled || this.perfStats) return;
			this.perfStats = {
				frameDt: [],
				frameWorkMs: 0,
				frameWorkMaxMs: 0,
				inputMs: 0,
				inputMaxMs: 0,
				resizeMs: 0,
				resizeMaxMs: 0,
				thinkMs: 0,
				thinkMaxMs: 0,
				drawMs: 0,
				drawMaxMs: 0,
				frames: 0,
				steps: 0,
				resizeChanges: 0,
				longTasks: [],
				gcEvents: [],
				lastCanvasSize: "",
				lastFlushAt: performance.now(),
			};

			const observe = (type) => {
				if (typeof PerformanceObserver !== "function") return;
				try {
					const observer = new PerformanceObserver((list) => {
						if (!this.perfStats) return;
						for (const entry of list.getEntries()) {
							if (type === "longtask") {
								this.perfStats.longTasks.push(entry.duration);
							}
							else if (type === "gc") {
								this.perfStats.gcEvents.push(entry.duration);
							}
						}
					});
					observer.observe({ type, buffered: true });
					this.perfObservers.push(observer);
				}
				catch {
					// unsupported entry type
				}
			};

			observe("longtask");
			observe("gc");

			const panel = document.createElement("pre");
			panel.id = "perf-debug";
			panel.setAttribute("aria-live", "polite");
			Object.assign(panel.style, {
				position: "fixed",
				right: "8px",
				top: "8px",
				zIndex: "9999",
				margin: "0",
				padding: "8px 10px",
				maxWidth: "min(92vw, 520px)",
				maxHeight: "45vh",
				overflow: "auto",
				background: "rgba(9, 13, 20, 0.82)",
				color: "#d7f3ff",
				font: "12px/1.4 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
				border: "1px solid rgba(215, 243, 255, 0.22)",
				borderRadius: "8px",
				pointerEvents: "none",
				whiteSpace: "pre-wrap",
			});
			panel.textContent = "perf=1 enabled\ncollecting frame data...";
			document.body.appendChild(panel);
			this.perfPanel = panel;
			window.__chipPerf = {
				read: () => this.perfPanel?.textContent || "",
				stats: () => typeof structuredClone === "function"
					? structuredClone(this.perfStats)
					: JSON.parse(JSON.stringify(this.perfStats)),
			};
		},

		updatePerfPanel(lines) {
			if (!this.perfEnabled || !this.perfPanel) return;
			this.perfPanel.textContent = lines.join("\n");
		},

		flushPerfStats(now, shadeStats) {
			if (!this.perfEnabled || !this.perfStats) return;
			const elapsedMs = now - this.perfStats.lastFlushAt;
			if (elapsedMs < 1000) return;

			const frameDt = this.perfStats.frameDt;
			const fps = frameDt.length ? (1000 / average(frameDt)) : 0;
			const p95Dt = percentile(frameDt, 0.95);
			const p99Dt = percentile(frameDt, 0.99);
			const budget60 = frameDt.filter((value) => value > 16.7).length;
			const budget90 = frameDt.filter((value) => value > 11.2).length;
			const budget120 = frameDt.filter((value) => value > 8.4).length;
			const budget144 = frameDt.filter((value) => value > 7.0).length;
			const heapMb = performance.memory?.usedJSHeapSize ? (performance.memory.usedJSHeapSize / (1024 * 1024)) : 0;
			const longTaskTotal = this.perfStats.longTasks.reduce((sum, value) => sum + value, 0);
			const gcTotal = this.perfStats.gcEvents.reduce((sum, value) => sum + value, 0);
			const glCounts = shadeStats?.counts || {};
			const glMs = shadeStats?.ms || {};
			const glSummary = [
				`texImage ${glCounts.texImage2D || 0}/${(glMs.texImage2D || 0).toFixed(1)}ms`,
				`texSub ${glCounts.texSubImage2D || 0}/${(glMs.texSubImage2D || 0).toFixed(1)}ms`,
				`bufData ${glCounts.bufferData || 0}/${(glMs.bufferData || 0).toFixed(1)}ms`,
				`bufSub ${glCounts.bufferSubData || 0}/${(glMs.bufferSubData || 0).toFixed(1)}ms`,
				`newTex ${glCounts.createTexture || 0}`,
				`delTex ${glCounts.deleteTexture || 0}`,
				`newFb ${glCounts.createFramebuffer || 0}`,
				`ctxLost ${shadeStats?.contextLost || 0}`,
			];

			const lines = [
				`perf=1 window ${(elapsedMs / 1000).toFixed(1)}s`,
				`raf avg ${fps.toFixed(1)}fps | p95 ${(p95Dt || 0).toFixed(2)}ms | p99 ${(p99Dt || 0).toFixed(2)}ms`,
				`missed budgets 60:${budget60} 90:${budget90} 120:${budget120} 144:${budget144}`,
				`js avg ms input ${(this.perfStats.inputMs / Math.max(1, this.perfStats.frames)).toFixed(2)} resize ${(this.perfStats.resizeMs / Math.max(1, this.perfStats.frames)).toFixed(2)} draw ${(this.perfStats.drawMs / Math.max(1, this.perfStats.frames)).toFixed(2)} think ${(this.perfStats.thinkMs / Math.max(1, this.perfStats.steps)).toFixed(2)}`,
				`js max ms input ${this.perfStats.inputMaxMs.toFixed(2)} resize ${this.perfStats.resizeMaxMs.toFixed(2)} draw ${this.perfStats.drawMaxMs.toFixed(2)} think ${this.perfStats.thinkMaxMs.toFixed(2)} frame ${this.perfStats.frameWorkMaxMs.toFixed(2)}`,
				`canvas ${this.perfStats.lastCanvasSize || "?"} | size changes ${this.perfStats.resizeChanges}`,
				`longtasks ${this.perfStats.longTasks.length}/${longTaskTotal.toFixed(1)}ms | gc ${this.perfStats.gcEvents.length}/${gcTotal.toFixed(1)}ms | heap ${heapMb ? heapMb.toFixed(1) + "MB" : "n/a"}`,
				`webgl ${glSummary.join(" | ")}`,
			];

			console.table({
				fps: Number(fps.toFixed(1)),
				raf_p95_ms: Number((p95Dt || 0).toFixed(2)),
				raf_p99_ms: Number((p99Dt || 0).toFixed(2)),
				input_avg_ms: Number((this.perfStats.inputMs / Math.max(1, this.perfStats.frames)).toFixed(2)),
				resize_avg_ms: Number((this.perfStats.resizeMs / Math.max(1, this.perfStats.frames)).toFixed(2)),
				draw_avg_ms: Number((this.perfStats.drawMs / Math.max(1, this.perfStats.frames)).toFixed(2)),
				think_avg_ms: Number((this.perfStats.thinkMs / Math.max(1, this.perfStats.steps)).toFixed(2)),
				longtasks: this.perfStats.longTasks.length,
				gc_events: this.perfStats.gcEvents.length,
				resize_changes: this.perfStats.resizeChanges,
				gl_texImage2D: glCounts.texImage2D || 0,
				gl_bufferData: glCounts.bufferData || 0,
				gl_createTexture: glCounts.createTexture || 0,
			});
			this.updatePerfPanel(lines);

			this.perfStats.frameDt = [];
			this.perfStats.frameWorkMs = 0;
			this.perfStats.frameWorkMaxMs = 0;
			this.perfStats.inputMs = 0;
			this.perfStats.inputMaxMs = 0;
			this.perfStats.resizeMs = 0;
			this.perfStats.resizeMaxMs = 0;
			this.perfStats.thinkMs = 0;
			this.perfStats.thinkMaxMs = 0;
			this.perfStats.drawMs = 0;
			this.perfStats.drawMaxMs = 0;
			this.perfStats.frames = 0;
			this.perfStats.steps = 0;
			this.perfStats.resizeChanges = 0;
			this.perfStats.longTasks = [];
			this.perfStats.gcEvents = [];
			this.perfStats.lastFlushAt = now;
		},

		get overlayStatusText() {
			if (this.loadingProgress >= 1 && this.wasmGamePtr) {
				return "Loading game session: Ready to play";
			}
			return `Loading game session: ${this.loadingStatus}`;
		},

		get isReadyToPlay() {
			return this.loadingProgress >= 1 && !!this.wasmGamePtr;
		},

		get showAllControlHints() {
			return this.isLandscapeLayout;
		},

		setLoadingStatus(text, progress) {
			this.loadingStatus = String(text);
			if (typeof progress === "number") {
				this.loadingProgress = Math.max(0, Math.min(1, progress));
			}
			this.queueLoadingPanelScaleSync();
		},

		detectTouchSupport() {
			if (navigator.maxTouchPoints > 0) return true;
			if (window.matchMedia && window.matchMedia("(pointer: coarse)").matches) return true;
			return "ontouchstart" in window;
		},

		hasConnectedGamepad() {
			const pads = navigator.getGamepads ? navigator.getGamepads() : [];
			for (const gamepad of pads) {
				if (gamepad && gamepad.connected) {
					return true;
				}
			}
			return false;
		},

		getPreferredControlScheme() {
			if (this.hasConnectedGamepad()) return "gamepad";
			if (this.hasTouchSupport) return "touch";
			return "keyboard";
		},

		syncControlSchemeFromEnvironment() {
			this.detectedControlScheme = this.getPreferredControlScheme();
			if (!this.userSelectedControlScheme) {
				this.selectedControlScheme = this.detectedControlScheme;
			}
			this.syncTouchControls();
		},

		selectControlScheme(scheme) {
			this.userSelectedControlScheme = true;
			this.selectedControlScheme = scheme;
			this.syncTouchControls();
			this.queueLoadingPanelScaleSync();
		},

		getCustomLevelPayload() {
			const params = new URLSearchParams(window.location.search);
			const compressed = params.get("levelc");
			if (compressed !== null) {
				return { value: compressed, compressed: true };
			}
			const plain = params.get("level");
			if (plain !== null) {
				return { value: plain, compressed: false };
			}
			return null;
		},

		isLandscapeViewport() {
			return window.innerWidth >= window.innerHeight;
		},

		syncViewportLayout() {
			this.isLandscapeLayout = this.isLandscapeViewport();
			this.updateDisplayModeCta();
			this.queueLoadingPanelScaleSync();
		},

		queueLoadingPanelScaleSync() {
			if (this.gameActive) return;
			const sync = () => this.syncLoadingPanelScale();
			if (typeof this.$nextTick === "function") {
				this.$nextTick(() => requestAnimationFrame(sync));
				return;
			}
			requestAnimationFrame(sync);
		},

		syncLoadingPanelScale() {
			const panel = this.$refs.loadingPanel;
			if (!panel) return;
			const viewportPadding = 24;
			const availableWidth = Math.max(1, window.innerWidth - viewportPadding);
			const availableHeight = Math.max(1, window.innerHeight - viewportPadding);
			const panelWidth = panel.offsetWidth;
			const panelHeight = panel.offsetHeight;
			if (panelWidth <= 0 || panelHeight <= 0) {
				this.loadingPanelScale = 1;
				this.loadingPanelFrameWidth = 620;
				this.loadingPanelFrameHeight = 0;
				return;
			}
			this.loadingPanelScale = Math.min(1, availableWidth / panelWidth, availableHeight / panelHeight);
			this.loadingPanelFrameWidth = panelWidth * this.loadingPanelScale;
			this.loadingPanelFrameHeight = panelHeight * this.loadingPanelScale;
		},

		isFullscreenActive() {
			return document.fullscreenElement === document.documentElement;
		},

		updateDisplayModeCta() {
			this.showDisplayModeCta = !(this.isFullscreenActive() && this.isLandscapeLayout);
		},

		handleWindowError(event) {
			console.error("window.error", event.error || event.message || event);
			this.setLoadingStatus(String(event.error?.message || event.message || event));
		},

		handleUnhandledRejection(event) {
			console.error("unhandledrejection", event.reason);
			this.setLoadingStatus(String(event.reason?.message || event.reason || event));
		},

		resizeCanvasToDisplaySize() {
			const canvas = this.$refs.canvas;
			const startedAt = this.perfEnabled ? performance.now() : 0;
			const dpr = Math.max(1, Math.min(3, window.devicePixelRatio || 1));
			const rect = canvas.getBoundingClientRect();
			const width = Math.max(1, Math.floor(rect.width * dpr));
			const height = Math.max(1, Math.floor(rect.height * dpr));
			const changed = canvas.width !== width || canvas.height !== height;
			if (canvas.width !== width || canvas.height !== height) {
				canvas.width = width;
				canvas.height = height;
			}
			if (this.perfEnabled && this.perfStats) {
				const elapsed = performance.now() - startedAt;
				this.perfStats.resizeMs += elapsed;
				this.perfStats.resizeMaxMs = Math.max(this.perfStats.resizeMaxMs, elapsed);
				if (changed) {
					this.perfStats.resizeChanges++;
				}
				this.perfStats.lastCanvasSize = `${width}x${height} @${dpr.toFixed(2)}x`;
			}
			return { width, height, dpr };
		},

		ensureAudioContext() {
			if (this.audioCtx) return this.audioCtx;
			const AudioContextCtor = window.AudioContext || window.webkitAudioContext;
			if (!AudioContextCtor) return null;
			this.audioCtx = new AudioContextCtor();
			return this.audioCtx;
		},

		async enableAudio() {
			if (this.audioLoaded) return;
			const ctx = this.ensureAudioContext();
			if (!ctx) return;
			if (ctx.state === "suspended") {
				try {
					await ctx.resume();
				}
				catch {
					return;
				}
			}
			if (!this.wasmExports) return;
			try {
				this.wasmExports.audioInit();
				this.audioLoaded = true;
			}
			catch (err) {
				console.warn("audio init failed", err);
			}
		},

		handleKeyDown(event) {
			this.pressedKeys.add(event.code);
			if (
				event.code.startsWith("Arrow") ||
				event.code === "Space" ||
				event.code === "Backspace" ||
				event.code === "Enter" ||
				event.code === "ShiftLeft" ||
				event.code === "ShiftRight"
			) {
				event.preventDefault();
			}
		},

		handleKeyUp(event) {
			this.pressedKeys.delete(event.code);
			if (
				event.code.startsWith("Arrow") ||
				event.code === "Space" ||
				event.code === "Backspace" ||
				event.code === "Enter" ||
				event.code === "ShiftLeft" ||
				event.code === "ShiftRight"
			) {
				event.preventDefault();
			}
		},

		getButtonsBitmask() {
			let buttons = 0;
			if (this.pressedKeys.has("ArrowUp") || this.pressedKeys.has("KeyW")) buttons |= INPUT_UP;
			if (this.pressedKeys.has("ArrowLeft") || this.pressedKeys.has("KeyA")) buttons |= INPUT_LEFT;
			if (this.pressedKeys.has("ArrowDown") || this.pressedKeys.has("KeyS")) buttons |= INPUT_DOWN;
			if (this.pressedKeys.has("ArrowRight") || this.pressedKeys.has("KeyD")) buttons |= INPUT_RIGHT;
			if (this.pressedKeys.has("Space")) buttons |= INPUT_A;
			if (this.pressedKeys.has("Backspace")) buttons |= INPUT_B;
			if (this.pressedKeys.has("Enter")) buttons |= INPUT_START;
			if (this.pressedKeys.has("ShiftLeft") || this.pressedKeys.has("ShiftRight")) buttons |= INPUT_SELECT;

			if (this.touchPressed.up) buttons |= INPUT_UP;
			if (this.touchPressed.left) buttons |= INPUT_LEFT;
			if (this.touchPressed.down) buttons |= INPUT_DOWN;
			if (this.touchPressed.right) buttons |= INPUT_RIGHT;
			if (this.touchPressed.a) buttons |= INPUT_A;
			if (this.touchPressed.b) buttons |= INPUT_B;
			if (this.touchPressed.start) buttons |= INPUT_START;
			if (this.touchPressed.select) buttons |= INPUT_SELECT;

			buttons |= this.getGamepadButtonsBitmask();
			return buttons;
		},

		getGamepadButtonsBitmask() {
			const pads = navigator.getGamepads ? navigator.getGamepads() : [];
			let buttons = 0;
			const deadzone = 0.5;

			for (const gamepad of pads) {
				if (!gamepad || !gamepad.connected) continue;
				const gamepadButtons = gamepad.buttons || [];
				const axes = gamepad.axes || [];

				if (gamepadButtons[12]?.pressed) buttons |= INPUT_UP;
				if (gamepadButtons[13]?.pressed) buttons |= INPUT_DOWN;
				if (gamepadButtons[14]?.pressed) buttons |= INPUT_LEFT;
				if (gamepadButtons[15]?.pressed) buttons |= INPUT_RIGHT;
				if (gamepadButtons[0]?.pressed) buttons |= INPUT_A;
				if (gamepadButtons[1]?.pressed) buttons |= INPUT_B;
				if (gamepadButtons[9]?.pressed) buttons |= INPUT_START;
				if (gamepadButtons[8]?.pressed || gamepadButtons[16]?.pressed) buttons |= INPUT_SELECT;

				const lx = axes[0] ?? 0;
				const ly = axes[1] ?? 0;
				if (lx <= -deadzone) buttons |= INPUT_LEFT;
				if (lx >= deadzone) buttons |= INPUT_RIGHT;
				if (ly <= -deadzone) buttons |= INPUT_UP;
				if (ly >= deadzone) buttons |= INPUT_DOWN;
			}

			return buttons;
		},

		syncTouchControls() {
			const shouldEnable = this.gameActive && this.selectedControlScheme === "touch";
			if (shouldEnable === this.touchEnabled) return;
			this.touchEnabled = shouldEnable;
			if (!shouldEnable) {
				this.clearTouchInputs();
			}
		},

		clearTouchInputs() {
			for (const key of Object.keys(this.touchPressed)) {
				this.touchPressed[key] = false;
			}
			this.padTouchId = null;
			this.padDir = null;
		},

		requestFullscreenOnTouch() {
			if (document.fullscreenElement) return;
			const target = document.documentElement;
			if (!target || !target.requestFullscreen) return;
			try {
				const result = target.requestFullscreen();
				if (result && typeof result.catch === "function") {
					result.catch(() => {});
				}
			}
			catch {
				// ignore
			}
		},

		async requestLandscapeLock() {
			const orientation = screen && screen.orientation;
			if (!orientation || !orientation.lock) return;
			try {
				await orientation.lock("landscape");
			}
			catch {
				// ignore
			}
		},

		async requestPreferredDisplayMode() {
			this.requestFullscreenOnTouch();
			await this.requestLandscapeLock();
			this.updateDisplayModeCta();
		},

		setPadDirection(dir) {
			if (this.padDir === dir) return;
			if (this.padDir) {
				this.touchPressed[this.padDir] = false;
			}
			this.padDir = dir;
			if (this.padDir) {
				this.touchPressed[this.padDir] = true;
			}
		},

		getPadDirectionFromTouch(touch) {
			const rect = this.$refs.touchPad.getBoundingClientRect();
			const x = touch.clientX - rect.left;
			const y = touch.clientY - rect.top;
			const dx = x - rect.width / 2;
			const dy = y - rect.height / 2;
			const minDim = Math.min(rect.width, rect.height);
			const dist = Math.hypot(dx, dy);
			if (dist < minDim * 0.09) return null;
			if (Math.abs(dx) > Math.abs(dy)) {
				return dx > 0 ? "right" : "left";
			}
			return dy > 0 ? "down" : "up";
		},

		pressTouchButton(id) {
			if (!this.touchEnabled) return;
			this.touchPressed[id] = true;
		},

		releaseTouchButton(id) {
			this.touchPressed[id] = false;
		},

		handleGamepadChange() {
			this.syncControlSchemeFromEnvironment();
		},

		async startGame() {
			if (!this.isReadyToPlay) return;
			if (this.audioEnabled) {
				await this.enableAudio();
			}
			this.gameActive = true;
			this.syncTouchControls();
		},

		onPadTouchStart(event) {
			if (!this.touchEnabled || this.padTouchId !== null) return;
			const touch = event.changedTouches[0];
			if (!touch) return;
			this.padTouchId = touch.identifier;
			this.setPadDirection(this.getPadDirectionFromTouch(touch));
		},

		onPadTouchMove(event) {
			if (!this.touchEnabled || this.padTouchId === null) return;
			const touch = Array.from(event.touches).find((item) => item.identifier === this.padTouchId);
			if (!touch) return;
			this.setPadDirection(this.getPadDirectionFromTouch(touch));
		},

		onPadTouchEnd(event) {
			if (this.padTouchId === null) return;
			for (const touch of event.changedTouches) {
				if (touch.identifier === this.padTouchId) {
					this.padTouchId = null;
					this.setPadDirection(null);
					break;
				}
			}
		},

		async load() {
			this.updateDisplayModeCta();
			const customLevelPayload = this.getCustomLevelPayload();
			this.setLoadingStatus("Initializing WebGL...", 0.1);
			const shade = createWasmAPI(this.$refs.canvas, {
				alpha: false,
				desynchronized: true,
				antialias: false,
				premultipliedAlpha: false,
				perfEnabled: this.perfEnabled,
			});

			let wasmMemory = null;
			const decoder = new TextDecoder();
			const encoder = new TextEncoder();

			const readUtf8 = (ptr, len) => {
				if (!wasmMemory || ptr === 0 || len === 0) return "";
				return decoder.decode(new Uint8Array(wasmMemory.buffer, ptr, len));
			};

			const randomBytes = (ptr, len) => {
				if (!wasmMemory) return;
				const out = new Uint8Array(wasmMemory.buffer, ptr, len);
				if (globalThis.crypto && globalThis.crypto.getRandomValues) {
					globalThis.crypto.getRandomValues(out);
					return;
				}
				for (let index = 0; index < out.length; index++) {
					out[index] = (Math.random() * 256) | 0;
				}
			};

			let resultValue = null;

			const playSound = (sound_id) => {
				if (!this.audioEnabled) return;
				const ctx = this.ensureAudioContext();
				if (!ctx) return;
				const entry = this.soundBank.get(sound_id | 0);
				if (!entry || !entry.buffer || ctx.state !== "running") return;
				const source = ctx.createBufferSource();
				source.buffer = entry.buffer;
				source.connect(entry.gain);
				source.start(0);
			};

			const stopMusic = () => {
				if (!this.musicSource) return;
				try {
					this.musicSource.stop();
				}
				catch {
					// ignore
				}
				this.musicSource.disconnect();
				this.musicSource = null;
				this.currentMusicKey = null;
			};

			const playMusic = (music_id) => {
				if (!this.audioEnabled) {
					this.requestedMusicKey = null;
					stopMusic();
					return;
				}
				const ctx = this.ensureAudioContext();
				if (!ctx) return;
				const id = music_id | 0;
				if (id < 0) {
					this.requestedMusicKey = null;
					stopMusic();
					return;
				}
				this.requestedMusicKey = id;
				if (this.currentMusicKey === id && this.musicSource) return;
				const entry = this.musicBank.get(id);
				if (!entry || !entry.buffer || ctx.state !== "running") {
					if (this.currentMusicKey !== id) {
						stopMusic();
					}
					return;
				}
				stopMusic();
				const source = ctx.createBufferSource();
				source.buffer = entry.buffer;
				source.loop = true;
				source.connect(entry.gain);
				source.start(0);
				this.musicSource = source;
				this.currentMusicKey = id;
			};

			const registerSound = (sound_id, data_ptr, data_len) => {
				const ctx = this.ensureAudioContext();
				if (!ctx || !wasmMemory) return;
				const bytes = new Uint8Array(wasmMemory.buffer, data_ptr, data_len);
				const copy = new Uint8Array(bytes);
				ctx.decodeAudioData(copy.buffer).then((buffer) => {
					const gain = ctx.createGain();
					gain.gain.value = 1.0;
					gain.connect(ctx.destination);
					this.soundBank.set(sound_id | 0, { buffer, gain });
				}).catch((err) => {
					console.warn("decodeAudioData failed", sound_id, err);
				});
			};

			const registerMusic = (music_id, data_ptr, data_len) => {
				const ctx = this.ensureAudioContext();
				if (!ctx || !wasmMemory) return;
				const bytes = new Uint8Array(wasmMemory.buffer, data_ptr, data_len);
				const copy = new Uint8Array(bytes);
				ctx.decodeAudioData(copy.buffer).then((buffer) => {
					const id = music_id | 0;
					const gain = ctx.createGain();
					gain.gain.value = 0.375;
					gain.connect(ctx.destination);
					this.musicBank.set(id, { buffer, gain });
					if (this.requestedMusicKey === id && (!this.musicSource || this.currentMusicKey !== id)) {
						playMusic(id);
					}
				}).catch((err) => {
					console.warn("decodeAudioData failed", music_id, err);
				});
			};

			const setTitle = (title_ptr, title_len) => {
				document.title = readUtf8(title_ptr, title_len) || "Chip DX";
			};

			const quitGame = () => {
				console.log("quitGame requested");
			};

			const resultError = (message_ptr, message_len) => {
				resultValue = new Error(readUtf8(message_ptr, message_len) || "Unknown error");
			};

			const readFile = (path_ptr, path_len, content_ptr, content_len_ptr) => {
				if (!wasmMemory) return -1;
				const path = readUtf8(path_ptr, path_len);
				if (!path || !content_len_ptr) return -1;
				let content = null;
				try {
					content = localStorage.getItem(path);
				}
				catch (err) {
					console.warn("readFile localStorage failed", err);
					return -1;
				}
				if (content === null) return -1;
				const bytes = encoder.encode(content);
				const lenView = new Uint32Array(wasmMemory.buffer, content_len_ptr, 1);
				if (!content_ptr) {
					lenView[0] = bytes.length;
					return 0;
				}
				const cap = lenView[0] >>> 0;
				const out = new Uint8Array(wasmMemory.buffer, content_ptr, cap);
				const toCopy = Math.min(cap, bytes.length);
				out.set(bytes.subarray(0, toCopy));
				lenView[0] = toCopy;
				return 0;
			};

			const writeFile = (path_ptr, path_len, content_ptr, content_len) => {
				if (!wasmMemory) return -1;
				const path = readUtf8(path_ptr, path_len);
				if (!path || !content_ptr) return -1;
				const bytes = new Uint8Array(wasmMemory.buffer, content_ptr, content_len);
				const content = decoder.decode(bytes);
				try {
					localStorage.setItem(path, content);
					return 0;
				}
				catch (err) {
					console.warn("writeFile localStorage failed", err);
					return -1;
				}
			};

			this.setLoadingStatus("Loading WASM...", 0.25);
			const response = await fetch("./chipwasm.wasm");
			if (!response.ok) {
				throw new Error(`Failed to fetch chipwasm.wasm: ${response.status} ${response.statusText}`);
			}

			const imports = {
				webgl: shade,
				env: {
					randomBytes,
					playSound,
					playMusic,
					registerSound,
					registerMusic,
					setTitle,
					quitGame,
					resultError,
					readFile,
					writeFile,
				},
			};

			this.setLoadingStatus("Compiling WASM...", 0.45);
			const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), imports);
			shade.bindInstance(instance);
			wasmMemory = instance.exports.memory;

			const exports = instance.exports;
			this.wasmExports = exports;

			const allocWasmBytes = (bytes) => {
				if (!exports.allocBytes || !exports.freeBytes) {
					throw new Error("WASM exports missing allocBytes/freeBytes bridge");
				}
				const capacity = Math.max(1, bytes.length);
				const ptr = exports.allocBytes(capacity);
				if (!ptr) {
					throw new Error("WASM allocBytes() returned null");
				}
				new Uint8Array(wasmMemory.buffer, ptr, bytes.length).set(bytes);
				return {
					ptr,
					len: bytes.length,
					capacity,
				};
			};

			this.setLoadingStatus("Starting...", 0.8);
			let gamePtr = 0;
			try {
				if (customLevelPayload !== null) {
					this.setLoadingStatus("Loading custom level...", 0.8);
					resultValue = null;
					const payloadBytes = encoder.encode(customLevelPayload.value);
					const payload = allocWasmBytes(payloadBytes);
					try {
						gamePtr = exports.createCustomLevel(payload.ptr, payload.len, customLevelPayload.compressed);
					}
					finally {
						exports.freeBytes(payload.ptr, payload.capacity);
					}
					if (resultValue instanceof Error) {
						throw resultValue;
					}
				}
				else {
					gamePtr = exports.createInstance();
				}
			}
			catch (err) {
				const message = err && err.message ? err.message : String(err);
				throw new Error(`${customLevelPayload !== null ? "custom level boot" : "createInstance()"} trapped: ${message}`);
			}
			if (!gamePtr) {
				const label = customLevelPayload === null ? "createInstance()" : "createCustomLevel()";
				throw new Error(`${label} returned null (0) instance pointer`);
			}
			this.wasmGamePtr = gamePtr;

			this.setLoadingStatus("Ready", 1);

			const stepMs = 1000 / 60;
			let acc = 0;
			let last = performance.now();
			let drawCount = 0;
			let thinkCount = 0;
			let lastHud = performance.now();

			const frame = (now) => {
				const frameStartedAt = this.perfEnabled ? performance.now() : 0;
				const dt = Math.min(250, now - last);
				last = now;
				acc += dt;
				if (this.perfEnabled && this.perfStats) {
					this.perfStats.frameDt.push(dt);
				}

				if (!this.userSelectedControlScheme) {
					this.syncControlSchemeFromEnvironment();
				}

				const inputStartedAt = this.perfEnabled ? performance.now() : 0;
				const buttons = this.getButtonsBitmask();
				if (this.perfEnabled && this.perfStats) {
					const elapsed = performance.now() - inputStartedAt;
					this.perfStats.inputMs += elapsed;
					this.perfStats.inputMaxMs = Math.max(this.perfStats.inputMaxMs, elapsed);
				}
				if (!this.gameActive) {
					this.resizeCanvasToDisplaySize();
					if (this.perfEnabled && this.perfStats) {
						const shadeStats = shade.getPerfStats ? shade.getPerfStats() : null;
						this.flushPerfStats(now, shadeStats);
						shade.resetPerfStats?.();
						const elapsed = performance.now() - frameStartedAt;
						this.perfStats.frameWorkMs += elapsed;
						this.perfStats.frameWorkMaxMs = Math.max(this.perfStats.frameWorkMaxMs, elapsed);
						this.perfStats.frames++;
					}
					this.frameHandle = requestAnimationFrame(frame);
					return;
				}

				while (acc >= stepMs) {
					const thinkStartedAt = this.perfEnabled ? performance.now() : 0;
					try {
						exports.thinkInstance(gamePtr, buttons);
					}
					catch (err) {
						this.setLoadingStatus(`thinkInstance() trapped: ${err && err.message ? err.message : String(err)}`);
						return;
					}
					if (this.perfEnabled && this.perfStats) {
						const elapsed = performance.now() - thinkStartedAt;
						this.perfStats.thinkMs += elapsed;
						this.perfStats.thinkMaxMs = Math.max(this.perfStats.thinkMaxMs, elapsed);
						this.perfStats.steps++;
					}
					thinkCount++;
					acc -= stepMs;
				}

				const { width, height } = this.resizeCanvasToDisplaySize();
				const drawStartedAt = this.perfEnabled ? performance.now() : 0;
				try {
					exports.drawInstance(gamePtr, now / 1000.0, width, height);
				}
				catch (err) {
					this.setLoadingStatus(`drawInstance() trapped: ${err && err.message ? err.message : String(err)}`);
					return;
				}
				if (this.perfEnabled && this.perfStats) {
					const elapsed = performance.now() - drawStartedAt;
					this.perfStats.drawMs += elapsed;
					this.perfStats.drawMaxMs = Math.max(this.perfStats.drawMaxMs, elapsed);
				}
				drawCount++;

				if (now - lastHud >= 500) {
					const secs = (now - lastHud) / 1000;
					const fps = Math.round(drawCount / secs);
					const tps = Math.round(thinkCount / secs);
					this.hudText = `draw ${fps}fps · think ${tps}hz`;
					drawCount = 0;
					thinkCount = 0;
					lastHud = now;
				}

				if (this.perfEnabled && this.perfStats) {
					const shadeStats = shade.getPerfStats ? shade.getPerfStats() : null;
					const elapsed = performance.now() - frameStartedAt;
					this.perfStats.frameWorkMs += elapsed;
					this.perfStats.frameWorkMaxMs = Math.max(this.perfStats.frameWorkMaxMs, elapsed);
					this.perfStats.frames++;
					this.flushPerfStats(now, shadeStats);
					shade.resetPerfStats?.();
				}

				this.frameHandle = requestAnimationFrame(frame);
			};

			this.resizeCanvasToDisplaySize();
			this.frameHandle = requestAnimationFrame(frame);

			if (!this.cleanupRegistered) {
				const cleanup = () => {
					if (this.frameHandle) {
						cancelAnimationFrame(this.frameHandle);
						this.frameHandle = 0;
					}
					try {
						exports.destroyInstance(gamePtr);
					}
					catch {
						// ignore
					}
				};
				window.addEventListener("pagehide", cleanup, { once: true });
				window.addEventListener("beforeunload", cleanup, { once: true });
				this.cleanupRegistered = true;
			}
		},
	};
};
