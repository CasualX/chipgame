import { createWasmAPI } from "./shade.js";

const INPUT_UP = 1 << 0;
const INPUT_LEFT = 1 << 1;
const INPUT_DOWN = 1 << 2;
const INPUT_RIGHT = 1 << 3;
const INPUT_A = 1 << 4;
const INPUT_B = 1 << 5;
const INPUT_START = 1 << 6;
const INPUT_SELECT = 1 << 7;

window.chipGame = function chipGame() {
	return {
		loadingStatus: "Starting...",
		loadingProgress: 0,
		loadingReadyVisible: false,
		loadingReadyText: "Press any input to begin",
		hudText: "",
		showDisplayModeCta: false,
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
		activationSeen: false,
		activationPointerActive: false,
		keyboardSeen: false,
		gamepadSeen: false,
		lastNonTouchInput: 0,
		pressedKeys: new Set(),
		padTouchId: null,
		audioCtx: null,
		audioLoaded: false,
		soundBank: new Map(),
		musicBank: new Map(),
		musicSource: null,
		currentMusicKey: null,
		wasmExports: null,
		wasmGamePtr: 0,
		frameHandle: 0,
		initialized: false,
		cleanupRegistered: false,

		init() {
			if (this.initialized) return;
			this.initialized = true;
			this.lastNonTouchInput = performance.now();
			this.boundWindowError = (event) => this.handleWindowError(event);
			this.boundUnhandledRejection = (event) => this.handleUnhandledRejection(event);
			this.boundKeyDown = (event) => this.handleKeyDown(event);
			this.boundKeyUp = (event) => this.handleKeyUp(event);
			this.boundPointerDown = (event) => this.handlePointerDown(event);
			this.boundGamepadConnected = () => this.handleGamepadConnected();
			this.boundGamepadDisconnected = () => this.handleGamepadDisconnected();
			this.boundResize = () => this.updateDisplayModeCta();
			this.boundFullscreenChange = () => this.updateDisplayModeCta();
			this.boundOrientationChange = () => this.updateDisplayModeCta();

			window.addEventListener("error", this.boundWindowError);
			window.addEventListener("unhandledrejection", this.boundUnhandledRejection);
			window.addEventListener("keydown", this.boundKeyDown);
			window.addEventListener("keyup", this.boundKeyUp);
			window.addEventListener("pointerdown", this.boundPointerDown);
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

		setLoadingStatus(text, progress) {
			this.loadingStatus = String(text);
			if (typeof progress === "number") {
				this.loadingProgress = Math.max(0, Math.min(1, progress));
			}
		},

		showLoadingReady(show) {
			this.loadingReadyVisible = !!show;
		},

		setReadyMessage(text) {
			this.loadingReadyText = text;
		},

		isLandscapeViewport() {
			if (screen && screen.orientation && typeof screen.orientation.type === "string") {
				return screen.orientation.type.startsWith("landscape");
			}
			return window.innerWidth >= window.innerHeight;
		},

		isFullscreenActive() {
			return document.fullscreenElement === document.documentElement;
		},

		updateDisplayModeCta() {
			this.showDisplayModeCta = !(this.isFullscreenActive() && this.isLandscapeViewport());
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
			const dpr = Math.max(1, Math.min(3, window.devicePixelRatio || 1));
			const rect = canvas.getBoundingClientRect();
			const width = Math.max(1, Math.floor(rect.width * dpr));
			const height = Math.max(1, Math.floor(rect.height * dpr));
			if (canvas.width !== width || canvas.height !== height) {
				canvas.width = width;
				canvas.height = height;
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

		tryLoadAudio() {
			if (this.audioLoaded) return;
			const ctx = this.ensureAudioContext();
			if (ctx && ctx.state === "suspended") {
				ctx.resume().catch(() => {});
			}
			if (!this.wasmExports || !this.wasmExports.audio_init) return;
			try {
				this.wasmExports.audio_init();
				this.audioLoaded = true;
			}
			catch (err) {
				console.warn("audio_init failed", err);
			}
		},

		handleKeyDown(event) {
			this.pressedKeys.add(event.code);
			this.keyboardSeen = true;
			this.lastNonTouchInput = performance.now();
			this.tryLoadAudio();
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
			this.keyboardSeen = true;
			this.lastNonTouchInput = performance.now();
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

		handlePointerDown(event) {
			if (event.pointerType !== "touch") {
				this.lastNonTouchInput = performance.now();
			}
			this.tryLoadAudio();
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
				this.gamepadSeen = true;
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

		updateTouchVisibility() {
			const coarse = window.matchMedia && window.matchMedia("(pointer: coarse)").matches;
			const recentlyNonTouch = performance.now() - this.lastNonTouchInput < 3000;
			const shouldEnable = coarse && !(this.keyboardSeen || this.gamepadSeen) && !recentlyNonTouch;
			if (shouldEnable === this.touchEnabled) return;
			this.touchEnabled = shouldEnable;
			if (!this.touchEnabled) {
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
			this.tryLoadAudio();
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
			this.tryLoadAudio();
			this.touchPressed[id] = true;
		},

		releaseTouchButton(id) {
			this.touchPressed[id] = false;
		},

		onActivationPointerDown() {
			this.activationPointerActive = true;
			this.tryLoadAudio();
		},

		onActivationTouchStart() {
			this.activationPointerActive = true;
			this.tryLoadAudio();
		},

		onActivationTouchEnd() {
			this.activationPointerActive = false;
		},

		onActivationPointerEnd() {
			this.activationPointerActive = false;
		},

		onPadTouchStart(event) {
			if (!this.touchEnabled || this.padTouchId !== null) return;
			const touch = event.changedTouches[0];
			if (!touch) return;
			this.padTouchId = touch.identifier;
			this.tryLoadAudio();
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

		handleGamepadConnected() {
			this.gamepadSeen = true;
			this.lastNonTouchInput = performance.now();
			this.updateTouchVisibility();
		},

		handleGamepadDisconnected() {
			this.gamepadSeen = false;
			this.updateTouchVisibility();
		},

		async load() {
			this.updateDisplayModeCta();
			this.setLoadingStatus("Initializing WebGL...", 0.1);
			const shade = createWasmAPI(this.$refs.canvas, {
				alpha: false,
				desynchronized: true,
				antialias: false,
				premultipliedAlpha: false,
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

			const play_sound = (sound_id) => {
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

			const play_music = (music_id) => {
				const ctx = this.ensureAudioContext();
				if (!ctx) return;
				const id = music_id | 0;
				if (id < 0) {
					stopMusic();
					return;
				}
				if (this.currentMusicKey === id && this.musicSource) return;
				const entry = this.musicBank.get(id);
				if (!entry || !entry.buffer || ctx.state !== "running") return;
				stopMusic();
				const source = ctx.createBufferSource();
				source.buffer = entry.buffer;
				source.loop = true;
				source.connect(entry.gain);
				source.start(0);
				this.musicSource = source;
				this.currentMusicKey = id;
			};

			const register_sound = (sound_id, data_ptr, data_len) => {
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

			const register_music = (music_id, data_ptr, data_len) => {
				const ctx = this.ensureAudioContext();
				if (!ctx || !wasmMemory) return;
				const bytes = new Uint8Array(wasmMemory.buffer, data_ptr, data_len);
				const copy = new Uint8Array(bytes);
				ctx.decodeAudioData(copy.buffer).then((buffer) => {
					const gain = ctx.createGain();
					gain.gain.value = 0.375;
					gain.connect(ctx.destination);
					this.musicBank.set(music_id | 0, { buffer, gain });
				}).catch((err) => {
					console.warn("decodeAudioData failed", music_id, err);
				});
			};

			const set_title = (title_ptr, title_len) => {
				document.title = readUtf8(title_ptr, title_len) || "Chip DX";
			};

			const quit_game = () => {
				console.log("quit_game requested");
			};

			const read_file = (path_ptr, path_len, content_ptr, content_len_ptr) => {
				if (!wasmMemory) return -1;
				const path = readUtf8(path_ptr, path_len);
				if (!path || !content_len_ptr) return -1;
				let content = null;
				try {
					content = localStorage.getItem(path);
				}
				catch (err) {
					console.warn("read_file localStorage failed", err);
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

			const write_file = (path_ptr, path_len, content_ptr, content_len) => {
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
					console.warn("write_file localStorage failed", err);
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
					play_sound,
					play_music,
					register_sound,
					register_music,
					set_title,
					quit_game,
					read_file,
					write_file,
				},
			};

			this.setLoadingStatus("Compiling WASM...", 0.45);
			const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), imports);
			shade.bindInstance(instance);
			wasmMemory = instance.exports.memory;

			const exports = instance.exports;
			if (!exports.create || !exports.think || !exports.draw) {
				throw new Error("WASM exports missing one of: create/think/draw");
			}
			this.wasmExports = exports;

			this.setLoadingStatus("Starting...", 0.8);
			let gamePtr = 0;
			try {
				gamePtr = exports.create();
			}
			catch (err) {
				throw new Error(`create() trapped: ${err && err.message ? err.message : String(err)}`);
			}
			if (!gamePtr) {
				throw new Error("create() returned null (0) instance pointer");
			}
			this.wasmGamePtr = gamePtr;

			this.setLoadingStatus("Ready", 1);
			this.setReadyMessage("Press any input to begin");
			this.showLoadingReady(true);

			const stepMs = 1000 / 60;
			let acc = 0;
			let last = performance.now();
			let drawCount = 0;
			let thinkCount = 0;
			let lastHud = performance.now();

			const frame = (now) => {
				const dt = Math.min(250, now - last);
				last = now;
				acc += dt;

				this.updateTouchVisibility();

				const buttons = this.getButtonsBitmask();
				const activationInput = buttons !== 0 || this.activationPointerActive;
				if (!this.gameActive) {
					this.resizeCanvasToDisplaySize();
					if (!this.activationSeen && activationInput) {
						this.activationSeen = true;
						this.setReadyMessage("Release all input to begin");
						this.showLoadingReady(true);
					}
					if (this.activationSeen && !activationInput) {
						this.gameActive = true;
					}
					this.frameHandle = requestAnimationFrame(frame);
					return;
				}

				while (acc >= stepMs) {
					try {
						exports.think(gamePtr, buttons);
					}
					catch (err) {
						this.setLoadingStatus(`think() trapped: ${err && err.message ? err.message : String(err)}`);
						return;
					}
					thinkCount++;
					acc -= stepMs;
				}

				const { width, height } = this.resizeCanvasToDisplaySize();
				try {
					exports.draw(gamePtr, now / 1000.0, width, height);
				}
				catch (err) {
					this.setLoadingStatus(`draw() trapped: ${err && err.message ? err.message : String(err)}`);
					return;
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
						if (exports.destroy) exports.destroy(gamePtr);
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
