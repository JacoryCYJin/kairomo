use crate::runtime::AppState;
use serde::Serialize;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use timer_core::{ShortcutBinding, ShortcutBindings, ShortcutKind, ShortcutModifier, Snapshot};

const REGISTER_FAILED: &str = "register_failed";
const UNREGISTER_FAILED: &str = "unregister_failed";
const DUPLICATE: &str = "shortcut_duplicate";
const INVALID: &str = "shortcut_invalid";
const REGISTER_FAILED_TOP: &str = "shortcut_register_failed";
const SAVE_FAILED_TOP: &str = "shortcut_save_failed";
const ROLLBACK_FAILED_TOP: &str = "shortcut_rollback_failed";
const UNREGISTER_FAILED_TOP: &str = "shortcut_unregister_failed";
const RECORDING_UNAVAILABLE: &str = "shortcut_recording_unavailable";

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct BindingStatus {
    pub enabled: bool,
    pub active: bool,
    pub active_binding: Option<ShortcutBinding>,
    pub stale_bindings: Vec<ShortcutBinding>,
    pub error: Option<String>,
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ShortcutStatus {
    pub enabled: bool,
    pub recording: bool,
    pub platform: &'static str,
    pub toggle: BindingStatus,
    pub pause: BindingStatus,
}

#[derive(Clone, Debug, Serialize)]
pub struct ShortcutMutation {
    pub snapshot: Snapshot,
    pub status: ShortcutStatus,
    pub error: Option<String>,
}

trait Backend {
    fn register(
        &self,
        kind: ShortcutKind,
        shortcut: &ShortcutBinding,
        generation: u64,
    ) -> Result<(), String>;
    fn unregister(&self, kind: ShortcutKind, shortcut: &ShortcutBinding) -> Result<(), String>;
    fn unregister_all(&self) -> Result<(), String>;
}

#[derive(Clone)]
struct Desired {
    enabled: bool,
    revision: u64,
    bindings: ShortcutBindings,
}

trait SettingsSource {
    fn current(&self) -> Result<Desired, String>;
}

trait Store {
    fn snapshot(&self) -> Result<Snapshot, String>;
    fn update_bindings(&self, bindings: &ShortcutBindings) -> Result<Snapshot, String>;
}

#[derive(Clone, Debug)]
struct Registered {
    kind: ShortcutKind,
    shortcut: ShortcutBinding,
    generation: u64,
}

#[derive(Default)]
struct BindingRuntime {
    active: bool,
    active_binding: Option<ShortcutBinding>,
    active_generation: Option<u64>,
    error: Option<String>,
    generation_counter: u64,
}

impl BindingRuntime {
    fn next_generation(&mut self) -> u64 {
        self.generation_counter = self.generation_counter.saturating_add(1);
        self.generation_counter
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.active_binding = None;
        self.active_generation = None;
    }
}

#[derive(Default)]
struct State {
    enabled: bool,
    recording: bool,
    suspended: bool,
    shutting_down: bool,
    bindings: [BindingRuntime; 2],
    registered: Vec<Registered>,
    latches: [Option<u64>; 2],
    configured: ShortcutBindings,
}

pub struct ShortcutCore {
    apply: Mutex<()>,
    gate: Mutex<()>,
    state: Mutex<State>,
}

impl Default for ShortcutCore {
    fn default() -> Self {
        Self::new()
    }
}

fn index(kind: ShortcutKind) -> usize {
    match kind {
        ShortcutKind::Toggle => 0,
        ShortcutKind::Pause => 1,
    }
}

fn kinds() -> [ShortcutKind; 2] {
    [ShortcutKind::Toggle, ShortcutKind::Pause]
}

impl ShortcutCore {
    pub fn new() -> Self {
        Self {
            apply: Mutex::new(()),
            gate: Mutex::new(()),
            state: Mutex::new(State::default()),
        }
    }

    fn deactivate_actions(&self) {
        let mut state = lock(&self.state);
        for binding in &mut state.bindings {
            binding.deactivate();
        }
        state.latches = [None, None];
    }

    fn begin_shutdown(&self) {
        let mut state = lock(&self.state);
        state.shutting_down = true;
        state.recording = false;
        state.suspended = false;
        for binding in &mut state.bindings {
            binding.deactivate();
        }
        state.latches = [None, None];
    }

    pub fn status(&self) -> ShortcutStatus {
        let state = lock(&self.state);
        let defaults = ShortcutBindings::default();
        ShortcutStatus {
            enabled: state.enabled,
            recording: state.recording,
            platform: platform(),
            toggle: binding_status(&state, ShortcutKind::Toggle, &defaults),
            pause: binding_status(&state, ShortcutKind::Pause, &defaults),
        }
    }

    fn allowed(&self, kind: ShortcutKind, generation: u64) -> bool {
        let state = lock(&self.state);
        allowed_locked(&state, kind, generation)
    }

    fn handle_event<F: FnOnce()>(
        &self,
        kind: ShortcutKind,
        generation: u64,
        event: ShortcutState,
        action: F,
    ) {
        match event {
            ShortcutState::Pressed => {
                let should_run = {
                    let mut state = lock(&self.state);
                    if !allowed_locked(&state, kind, generation)
                        || state.latches[index(kind)] == Some(generation)
                    {
                        false
                    } else {
                        state.latches[index(kind)] = Some(generation);
                        true
                    }
                };
                if should_run {
                    action();
                }
            }
            ShortcutState::Released => {
                let mut state = lock(&self.state);
                if state.latches[index(kind)] == Some(generation) {
                    state.latches[index(kind)] = None;
                }
            }
        }
    }

    fn run_gated<F: FnOnce()>(&self, kind: ShortcutKind, generation: u64, action: F) -> bool {
        let _gate = lock(&self.gate);
        if !self.allowed(kind, generation) {
            return false;
        }
        action();
        true
    }

    fn register_candidate(
        &self,
        backend: &dyn Backend,
        kind: ShortcutKind,
        shortcut: &ShortcutBinding,
    ) -> Result<u64, String> {
        let generation = {
            let mut state = lock(&self.state);
            state.bindings[index(kind)].next_generation()
        };
        backend.register(kind, shortcut, generation)?;
        let mut state = lock(&self.state);
        state.registered.push(Registered {
            kind,
            shortcut: shortcut.clone(),
            generation,
        });
        Ok(generation)
    }

    fn activate(&self, kind: ShortcutKind, shortcut: &ShortcutBinding, generation: u64) {
        let mut state = lock(&self.state);
        if !state.enabled
            || !state.configured.enabled(kind)
            || state.recording
            || state.suspended
            || state.shutting_down
        {
            return;
        }
        let binding = &mut state.bindings[index(kind)];
        binding.active = true;
        binding.active_binding = Some(shortcut.clone());
        binding.active_generation = Some(generation);
        binding.error = None;
    }

    fn set_error(&self, kind: ShortcutKind, error: &str) {
        lock(&self.state).bindings[index(kind)].error = Some(error.into());
    }

    fn remove_registered(&self, kind: ShortcutKind, generation: u64) {
        lock(&self.state)
            .registered
            .retain(|entry| entry.kind != kind || entry.generation != generation);
    }

    fn deactivate_binding(&self, kind: ShortcutKind) {
        let mut state = lock(&self.state);
        state.bindings[index(kind)].deactivate();
        state.latches[index(kind)] = None;
    }

    fn release_all(&self, backend: &dyn Backend) -> Vec<Registered> {
        self.deactivate_actions();
        let entries = lock(&self.state).registered.clone();
        let mut failures = Vec::new();
        for entry in entries {
            match backend.unregister(entry.kind, &entry.shortcut) {
                Ok(()) => self.remove_registered(entry.kind, entry.generation),
                Err(_) => {
                    self.set_error(entry.kind, UNREGISTER_FAILED);
                    failures.push(entry);
                }
            }
        }
        failures
    }

    fn ensure_enabled(&self, backend: &dyn Backend, desired: &ShortcutBindings) {
        let entries = lock(&self.state).registered.clone();
        for entry in entries {
            if !desired.enabled(entry.kind) || !entry.shortcut.same(desired.for_binding(entry.kind))
            {
                self.deactivate_binding(entry.kind);
                match backend.unregister(entry.kind, &entry.shortcut) {
                    Ok(()) => self.remove_registered(entry.kind, entry.generation),
                    Err(_) => self.set_error(entry.kind, UNREGISTER_FAILED),
                }
            }
        }

        for kind in kinds() {
            if !desired.enabled(kind) {
                self.deactivate_binding(kind);
                continue;
            }
            let shortcut = desired.for_binding(kind).clone();
            let existing = lock(&self.state)
                .registered
                .iter()
                .find(|entry| entry.kind == kind && entry.shortcut.same(&shortcut))
                .cloned();
            let generation = if let Some(entry) = existing {
                entry.generation
            } else {
                match self.register_candidate(backend, kind, &shortcut) {
                    Ok(generation) => generation,
                    Err(_) => {
                        self.set_error(kind, REGISTER_FAILED);
                        continue;
                    }
                }
            };
            self.activate(kind, &shortcut, generation);
        }
    }

    fn restore_after_failure(&self, backend: &dyn Backend, old: &Desired) -> bool {
        {
            let mut state = lock(&self.state);
            state.recording = false;
            state.suspended = false;
            state.enabled = old.enabled;
            state.configured = old.bindings.clone();
        }
        self.deactivate_actions();
        self.release_all(backend);
        if old.enabled {
            self.ensure_enabled(backend, &old.bindings);
            let status = self.status();
            let toggle_ok = !old.bindings.enabled(ShortcutKind::Toggle) || status.toggle.active;
            let pause_ok = !old.bindings.enabled(ShortcutKind::Pause) || status.pause.active;
            toggle_ok && pause_ok && status.toggle.error.is_none() && status.pause.error.is_none()
        } else {
            let status = self.status();
            status.toggle.error.is_none() && status.pause.error.is_none()
        }
    }

    fn reconcile(&self, backend: &dyn Backend, source: &dyn SettingsSource) -> ShortcutStatus {
        if let Ok(desired) = source.current() {
            if !desired.enabled {
                self.deactivate_actions();
            }
        }
        let _apply = lock(&self.apply);
        self.reconcile_locked(backend, source)
    }

    fn reconcile_locked(
        &self,
        backend: &dyn Backend,
        source: &dyn SettingsSource,
    ) -> ShortcutStatus {
        while let Ok(desired) = source.current() {
            let (shutting_down, recording, suspended) = {
                let mut state = lock(&self.state);
                state.configured = desired.bindings.clone();
                if state.shutting_down {
                    state.enabled = desired.enabled;
                }
                (state.shutting_down, state.recording, state.suspended)
            };
            if shutting_down {
                break;
            }
            if desired.enabled {
                lock(&self.state).enabled = true;
                if !recording && !suspended {
                    self.ensure_enabled(backend, &desired.bindings);
                }
            } else {
                self.deactivate_actions();
                self.release_all(backend);
                lock(&self.state).enabled = false;
            }
            match source.current() {
                Ok(latest) if latest.revision == desired.revision => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        self.status()
    }

    #[cfg(test)]
    fn replace_bindings(
        &self,
        backend: &dyn Backend,
        source: &dyn SettingsSource,
        store: &dyn Store,
        target: ShortcutBindings,
    ) -> Result<ShortcutMutation, String> {
        let _apply = lock(&self.apply);
        let old = source.current()?;
        self.replace_bindings_locked(backend, store, old, target)
    }

    fn restore_all(
        &self,
        backend: &dyn Backend,
        source: &dyn SettingsSource,
        store: &dyn Store,
    ) -> Result<ShortcutMutation, String> {
        let _apply = lock(&self.apply);
        let old = source.current()?;
        let target = ShortcutBindings {
            toggle_enabled: old.bindings.toggle_enabled,
            pause_enabled: old.bindings.pause_enabled,
            ..ShortcutBindings::default()
        };
        self.replace_bindings_locked(backend, store, old, target)
    }

    fn update_binding(
        &self,
        backend: &dyn Backend,
        source: &dyn SettingsSource,
        store: &dyn Store,
        kind: ShortcutKind,
        binding: ShortcutBinding,
    ) -> Result<ShortcutMutation, String> {
        let _apply = lock(&self.apply);
        let old = source.current()?;
        let mut target = old.bindings.clone();
        target.set_binding(kind, binding.canonical());
        self.replace_bindings_locked(backend, store, old, target)
    }

    fn replace_bindings_locked(
        &self,
        backend: &dyn Backend,
        store: &dyn Store,
        old: Desired,
        target: ShortcutBindings,
    ) -> Result<ShortcutMutation, String> {
        {
            let mut state = lock(&self.state);
            state.configured = old.bindings.clone();
            state.enabled = old.enabled;
        }
        let validation_error = if target.toggle.same(&target.pause) {
            Some(DUPLICATE)
        } else if target.validate().is_err() {
            Some(INVALID)
        } else {
            None
        };
        if let Some(code) = validation_error {
            let rollback_ok = if lock(&self.state).recording {
                self.restore_after_failure(backend, &old)
            } else {
                true
            };
            let snapshot = store.snapshot()?;
            return Ok(ShortcutMutation {
                snapshot,
                status: self.status(),
                error: Some(if rollback_ok {
                    code.into()
                } else {
                    ROLLBACK_FAILED_TOP.into()
                }),
            });
        }
        if target == old.bindings {
            let rollback_ok = if lock(&self.state).recording {
                self.restore_after_failure(backend, &old)
            } else {
                true
            };
            let snapshot = store.snapshot()?;
            return Ok(ShortcutMutation {
                snapshot,
                status: self.status(),
                error: if rollback_ok {
                    None
                } else {
                    Some(ROLLBACK_FAILED_TOP.into())
                },
            });
        }

        {
            let mut state = lock(&self.state);
            state.suspended = true;
        }
        self.deactivate_actions();
        let release_failures = self.release_all(backend);
        if !release_failures.is_empty() {
            let rollback_ok = self.restore_after_failure(backend, &old);
            let snapshot = store.snapshot()?;
            return Ok(ShortcutMutation {
                snapshot,
                status: self.status(),
                error: Some(if rollback_ok {
                    UNREGISTER_FAILED_TOP.into()
                } else {
                    ROLLBACK_FAILED_TOP.into()
                }),
            });
        }

        let mut registered_targets = Vec::new();
        if old.enabled {
            for kind in kinds() {
                if !target.enabled(kind) {
                    continue;
                }
                match self.register_candidate(backend, kind, target.for_binding(kind)) {
                    Ok(generation) => registered_targets.push((kind, generation)),
                    Err(_) => {
                        let rollback_ok = self.restore_after_failure(backend, &old);
                        let snapshot = store.snapshot()?;
                        return Ok(ShortcutMutation {
                            snapshot,
                            status: self.status(),
                            error: Some(if rollback_ok {
                                REGISTER_FAILED_TOP.into()
                            } else {
                                ROLLBACK_FAILED_TOP.into()
                            }),
                        });
                    }
                }
            }
        }

        let snapshot = match store.update_bindings(&target) {
            Ok(snapshot) => snapshot,
            Err(_) => {
                let rollback_ok = self.restore_after_failure(backend, &old);
                let snapshot = store.snapshot()?;
                return Ok(ShortcutMutation {
                    snapshot,
                    status: self.status(),
                    error: Some(if rollback_ok {
                        SAVE_FAILED_TOP.into()
                    } else {
                        ROLLBACK_FAILED_TOP.into()
                    }),
                });
            }
        };

        {
            let mut state = lock(&self.state);
            state.configured = target.clone();
            state.enabled = old.enabled;
            state.recording = false;
            state.suspended = false;
        }
        let mut cleanup_error = None;
        if old.enabled {
            for (kind, generation) in registered_targets {
                self.activate(kind, target.for_binding(kind), generation);
            }
        } else {
            let failures = self.release_all(backend);
            self.deactivate_actions();
            if !failures.is_empty() {
                cleanup_error = Some(UNREGISTER_FAILED_TOP.to_string());
            }
        }

        Ok(ShortcutMutation {
            snapshot,
            status: self.status(),
            error: cleanup_error,
        })
    }

    fn set_binding_enabled(
        &self,
        backend: &dyn Backend,
        source: &dyn SettingsSource,
        store: &dyn Store,
        kind: ShortcutKind,
        enabled: bool,
    ) -> Result<ShortcutMutation, String> {
        let _apply = lock(&self.apply);
        let old = source.current()?;
        let mut target = old.bindings.clone();
        target.set_enabled(kind, enabled);
        self.replace_bindings_locked(backend, store, old, target)
    }

    fn begin_recording(
        &self,
        backend: &dyn Backend,
        source: &dyn SettingsSource,
        store: &dyn Store,
    ) -> Result<ShortcutMutation, String> {
        let _apply = lock(&self.apply);
        let desired = source.current()?;
        {
            let mut state = lock(&self.state);
            state.configured = desired.bindings.clone();
            state.enabled = desired.enabled;
        }
        if lock(&self.state).recording {
            let snapshot = store.snapshot()?;
            return Ok(ShortcutMutation {
                snapshot,
                status: self.status(),
                error: None,
            });
        }
        {
            let mut state = lock(&self.state);
            state.recording = true;
            state.suspended = false;
        }
        self.deactivate_actions();
        let failures = self.release_all(backend);
        if !failures.is_empty() {
            let rollback_ok = self.restore_after_failure(backend, &desired);
            let snapshot = store.snapshot()?;
            return Ok(ShortcutMutation {
                snapshot,
                status: self.status(),
                error: Some(if rollback_ok {
                    RECORDING_UNAVAILABLE.into()
                } else {
                    ROLLBACK_FAILED_TOP.into()
                }),
            });
        }
        let snapshot = store.snapshot()?;
        Ok(ShortcutMutation {
            snapshot,
            status: self.status(),
            error: None,
        })
    }

    fn end_recording(
        &self,
        backend: &dyn Backend,
        source: &dyn SettingsSource,
        store: &dyn Store,
    ) -> Result<ShortcutMutation, String> {
        {
            let mut state = lock(&self.state);
            state.recording = false;
            state.suspended = false;
        }
        let status = self.reconcile(backend, source);
        let snapshot = store.snapshot()?;
        let error = if status.toggle.error.is_some() || status.pause.error.is_some() {
            Some(REGISTER_FAILED_TOP.into())
        } else {
            None
        };
        Ok(ShortcutMutation {
            snapshot,
            status,
            error,
        })
    }
}

fn allowed_locked(state: &State, kind: ShortcutKind, generation: u64) -> bool {
    let binding = &state.bindings[index(kind)];
    state.enabled
        && state.configured.enabled(kind)
        && !state.recording
        && !state.suspended
        && !state.shutting_down
        && binding.active
        && binding.active_generation == Some(generation)
}

fn binding_status(state: &State, kind: ShortcutKind, defaults: &ShortcutBindings) -> BindingStatus {
    let active_generation = state.bindings[index(kind)].active_generation;
    let mut stale_bindings = Vec::new();
    for entry in &state.registered {
        if entry.kind == kind
            && Some(entry.generation) != active_generation
            && !stale_bindings
                .iter()
                .any(|shortcut: &ShortcutBinding| shortcut.same(&entry.shortcut))
        {
            stale_bindings.push(entry.shortcut.clone());
        }
    }
    BindingStatus {
        enabled: state.configured.enabled(kind),
        active: state.bindings[index(kind)].active && active_generation.is_some(),
        active_binding: state.bindings[index(kind)].active_binding.clone(),
        stale_bindings,
        error: state.bindings[index(kind)].error.clone(),
        is_default: state
            .configured
            .for_binding(kind)
            .same(defaults.for_binding(kind)),
    }
}

fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else {
        "other"
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn accelerator(binding: &ShortcutBinding) -> String {
    let mut modifiers = binding.modifiers.clone();
    modifiers.sort();
    modifiers.dedup();
    let mut parts = Vec::with_capacity(modifiers.len() + 1);
    for modifier in modifiers {
        parts.push(
            match modifier {
                ShortcutModifier::Control => "Control",
                ShortcutModifier::Alt => "Alt",
                ShortcutModifier::Shift => "Shift",
                ShortcutModifier::Meta => "Meta",
            }
            .to_string(),
        );
    }
    parts.push(binding.key.clone());
    parts.join("+")
}

pub struct Shortcuts(pub Arc<ShortcutCore>);

struct TauriBackend {
    app: AppHandle,
    core: Arc<ShortcutCore>,
}

impl TauriBackend {
    fn new(app: AppHandle, core: Arc<ShortcutCore>) -> Self {
        Self { app, core }
    }
}

impl Backend for TauriBackend {
    fn register(
        &self,
        kind: ShortcutKind,
        shortcut: &ShortcutBinding,
        generation: u64,
    ) -> Result<(), String> {
        let accelerator = accelerator(shortcut);
        let core = self.core.clone();
        let result =
            self.app
                .global_shortcut()
                .on_shortcut(accelerator.as_str(), move |app, _, event| {
                    crate::platform::windows::trace(
                        app,
                        &format!(
                            "shortcut event {:?} {:?} gen={generation}",
                            kind, event.state
                        ),
                    );
                    let core = core.clone();
                    let action_core = core.clone();
                    let app = app.clone();
                    core.handle_event(kind, generation, event.state, move || {
                        dispatch_action(app, action_core, kind, generation);
                    });
                });
        match result {
            Ok(()) => {
                crate::platform::windows::trace(
                    &self.app,
                    &format!(
                        "shortcut registered {:?} {accelerator} gen={generation}",
                        kind
                    ),
                );
                Ok(())
            }
            Err(error) => {
                crate::platform::windows::trace(
                    &self.app,
                    &format!("shortcut register failed: {error}"),
                );
                Err(error.to_string())
            }
        }
    }

    fn unregister(&self, _kind: ShortcutKind, shortcut: &ShortcutBinding) -> Result<(), String> {
        let accelerator = accelerator(shortcut);
        self.app
            .global_shortcut()
            .unregister(accelerator.as_str())
            .map_err(|error| {
                crate::platform::windows::trace(
                    &self.app,
                    &format!("shortcut unregister failed: {error}"),
                );
                error.to_string()
            })
    }

    fn unregister_all(&self) -> Result<(), String> {
        self.app
            .global_shortcut()
            .unregister_all()
            .map_err(|error| error.to_string())
    }
}

fn dispatch_action(app: AppHandle, core: Arc<ShortcutCore>, kind: ShortcutKind, generation: u64) {
    match kind {
        ShortcutKind::Toggle => {
            let handle = app.clone();
            let core = core.clone();
            let _ = app.run_on_main_thread(move || {
                if core.run_gated(kind, generation, || {
                    crate::platform::windows::toggle_now(&handle);
                }) {
                    crate::platform::windows::trace(&handle, "shortcut action toggle");
                } else {
                    crate::platform::windows::trace(&handle, "shortcut action toggle skipped");
                }
            });
        }
        ShortcutKind::Pause => {
            if core.run_gated(kind, generation, || crate::platform::toggle_pause(&app)) {
                crate::platform::windows::trace(&app, "shortcut action pause/resume");
            } else {
                crate::platform::windows::trace(&app, "shortcut action pause/resume skipped");
            }
        }
    }
}

struct RuntimeSettings {
    app: AppHandle,
}

impl RuntimeSettings {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl SettingsSource for RuntimeSettings {
    fn current(&self) -> Result<Desired, String> {
        let state = self.app.state::<AppState>();
        let runtime = state
            .0
            .lock()
            .map_err(|_| "runtime_unavailable".to_string())?;
        Ok(Desired {
            enabled: runtime.engine.state.settings.global_shortcuts,
            revision: runtime.engine.state.revision,
            bindings: runtime.engine.state.settings.shortcut_bindings.clone(),
        })
    }
}

struct RuntimeStore {
    app: AppHandle,
}

impl RuntimeStore {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Store for RuntimeStore {
    fn snapshot(&self) -> Result<Snapshot, String> {
        let state = self.app.state::<AppState>();
        let runtime = state
            .0
            .lock()
            .map_err(|_| "runtime_unavailable".to_string())?;
        Ok(runtime.engine.state.clone())
    }

    fn update_bindings(&self, bindings: &ShortcutBindings) -> Result<Snapshot, String> {
        let state = self.app.state::<AppState>();
        let runtime = state
            .0
            .lock()
            .map_err(|_| "runtime_unavailable".to_string())?;
        let mut runtime = runtime;
        runtime.shortcut_bindings(bindings.clone())
    }
}

fn core(app: &AppHandle) -> Arc<ShortcutCore> {
    app.state::<Shortcuts>().0.clone()
}

fn backend(app: &AppHandle, core: Arc<ShortcutCore>) -> TauriBackend {
    TauriBackend::new(app.clone(), core)
}

fn source(app: &AppHandle) -> RuntimeSettings {
    RuntimeSettings::new(app.clone())
}

fn store(app: &AppHandle) -> RuntimeStore {
    RuntimeStore::new(app.clone())
}

pub fn setup(app: &AppHandle) {
    let core = Arc::new(ShortcutCore::new());
    let _ = app.manage(Shortcuts(core));
}

/// Register saved shortcuts only after Tauri's event loop is ready.
pub fn start(app: &AppHandle) {
    let handle = app.clone();
    let core = core(app);
    std::thread::spawn(move || {
        let backend = backend(&handle, core.clone());
        let source = source(&handle);
        let status = core.reconcile(&backend, &source);
        let _ = handle.emit("shortcuts:changed", status);
    });
}

pub fn apply_from_saved(app: &AppHandle) -> ShortcutStatus {
    let core = core(app);
    let backend = backend(app, core.clone());
    let source = source(app);
    let status = core.reconcile(&backend, &source);
    let _ = app.emit("shortcuts:changed", &status);
    status
}

pub fn begin_recording(app: &AppHandle) -> Result<ShortcutMutation, String> {
    let core = core(app);
    let backend = backend(app, core.clone());
    let source = source(app);
    let store = store(app);
    let mutation = core.begin_recording(&backend, &source, &store)?;
    let _ = app.emit("shortcuts:changed", &mutation.status);
    Ok(mutation)
}

pub fn end_recording(app: &AppHandle) -> Result<ShortcutMutation, String> {
    let core = core(app);
    let backend = backend(app, core.clone());
    let source = source(app);
    let store = store(app);
    let mutation = core.end_recording(&backend, &source, &store)?;
    let _ = app.emit("shortcuts:changed", &mutation.status);
    Ok(mutation)
}

pub fn update_shortcut(
    app: &AppHandle,
    kind: ShortcutKind,
    binding: ShortcutBinding,
) -> Result<ShortcutMutation, String> {
    let core = core(app);
    let backend = backend(app, core.clone());
    let source = source(app);
    let store = store(app);
    let mutation = core.update_binding(&backend, &source, &store, kind, binding)?;
    let _ = app.emit("shortcuts:changed", &mutation.status);
    Ok(mutation)
}

pub fn restore_shortcut(app: &AppHandle, kind: ShortcutKind) -> Result<ShortcutMutation, String> {
    let binding = match kind {
        ShortcutKind::Toggle => ShortcutBinding::toggle_default(),
        ShortcutKind::Pause => ShortcutBinding::pause_default(),
    };
    update_shortcut(app, kind, binding)
}

pub fn set_shortcut_enabled(
    app: &AppHandle,
    kind: ShortcutKind,
    enabled: bool,
) -> Result<ShortcutMutation, String> {
    let core = core(app);
    let backend = backend(app, core.clone());
    let source = source(app);
    let store = store(app);
    let mutation = core.set_binding_enabled(&backend, &source, &store, kind, enabled)?;
    let _ = app.emit("shortcuts:changed", &mutation.status);
    Ok(mutation)
}

pub fn restore_all_shortcuts(app: &AppHandle) -> Result<ShortcutMutation, String> {
    let core = core(app);
    let backend = backend(app, core.clone());
    let source = source(app);
    let store = store(app);
    let mutation = core.restore_all(&backend, &source, &store)?;
    let _ = app.emit("shortcuts:changed", &mutation.status);
    Ok(mutation)
}

pub fn retry_shortcuts(app: &AppHandle) -> Result<ShortcutMutation, String> {
    let core = core(app);
    let backend = backend(app, core.clone());
    let source = source(app);
    let store = store(app);
    let status = core.reconcile(&backend, &source);
    let snapshot = store.snapshot()?;
    let _ = app.emit("shortcuts:changed", &status);
    Ok(ShortcutMutation {
        snapshot,
        status,
        error: None,
    })
}

pub fn shutdown(app: &AppHandle) {
    let Some(shortcuts) = app.try_state::<Shortcuts>() else {
        return;
    };
    let core = shortcuts.0.clone();
    core.begin_shutdown();
    let backend = backend(app, core);
    if let Err(error) = backend.unregister_all() {
        crate::platform::windows::trace(app, &format!("shortcut unregister_all failed: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{channel, Receiver, Sender},
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct FakeRegistration {
        kind: ShortcutKind,
        shortcut: ShortcutBinding,
        generation: u64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Call {
        Register(ShortcutKind),
        Unregister(ShortcutKind),
        UnregisterAll,
    }

    struct Block {
        entered: Sender<()>,
        release: Receiver<()>,
    }

    #[derive(Default)]
    struct FakeBackend {
        registrations: Mutex<Vec<FakeRegistration>>,
        calls: Mutex<Vec<Call>>,
        fail_register: Mutex<HashSet<String>>,
        fail_unregister: Mutex<HashSet<String>>,
        block: Mutex<Option<Block>>,
    }

    impl FakeBackend {
        fn registrations(&self) -> Vec<FakeRegistration> {
            lock(&self.registrations).clone()
        }

        fn calls(&self) -> Vec<Call> {
            lock(&self.calls).clone()
        }

        fn clear_calls(&self) {
            lock(&self.calls).clear();
        }

        fn fail_register(&self, shortcut: &ShortcutBinding) {
            lock(&self.fail_register).insert(label(shortcut));
        }

        fn fail_unregister(&self, shortcut: &ShortcutBinding) {
            lock(&self.fail_unregister).insert(label(shortcut));
        }

        fn clear_failures(&self) {
            lock(&self.fail_register).clear();
            lock(&self.fail_unregister).clear();
        }

        fn block_next_register(&self) -> (Receiver<()>, Sender<()>) {
            let (entered_tx, entered_rx) = channel();
            let (release_tx, release_rx) = channel();
            *lock(&self.block) = Some(Block {
                entered: entered_tx,
                release: release_rx,
            });
            (entered_rx, release_tx)
        }
    }

    impl Backend for FakeBackend {
        fn register(
            &self,
            kind: ShortcutKind,
            shortcut: &ShortcutBinding,
            generation: u64,
        ) -> Result<(), String> {
            if let Some(block) = lock(&self.block).take() {
                let _ = block.entered.send(());
                let _ = block.release.recv();
            }
            if lock(&self.fail_register).contains(&label(shortcut)) {
                lock(&self.calls).push(Call::Register(kind));
                return Err("register_failed".into());
            }
            if lock(&self.registrations)
                .iter()
                .any(|entry| entry.shortcut.same(shortcut))
            {
                lock(&self.calls).push(Call::Register(kind));
                return Err("already_registered".into());
            }
            lock(&self.registrations).push(FakeRegistration {
                kind,
                shortcut: shortcut.clone(),
                generation,
            });
            lock(&self.calls).push(Call::Register(kind));
            Ok(())
        }

        fn unregister(&self, kind: ShortcutKind, shortcut: &ShortcutBinding) -> Result<(), String> {
            lock(&self.calls).push(Call::Unregister(kind));
            if lock(&self.fail_unregister).contains(&label(shortcut)) {
                return Err("unregister_failed".into());
            }
            let mut registrations = lock(&self.registrations);
            let Some(position) = registrations
                .iter()
                .position(|entry| entry.kind == kind && entry.shortcut.same(shortcut))
            else {
                return Err("not_registered".into());
            };
            registrations.remove(position);
            Ok(())
        }

        fn unregister_all(&self) -> Result<(), String> {
            lock(&self.calls).push(Call::UnregisterAll);
            lock(&self.registrations).clear();
            Ok(())
        }
    }

    #[derive(Clone)]
    struct SharedSettings {
        enabled: bool,
        bindings: ShortcutBindings,
        revision: u64,
    }

    struct FakeSettings {
        shared: Arc<Mutex<SharedSettings>>,
    }

    impl FakeSettings {
        fn set(&self, enabled: bool, bindings: ShortcutBindings) {
            let mut shared = lock(&self.shared);
            shared.enabled = enabled;
            shared.bindings = bindings;
            shared.revision = shared.revision.saturating_add(1);
        }
    }

    impl SettingsSource for FakeSettings {
        fn current(&self) -> Result<Desired, String> {
            let shared = lock(&self.shared);
            Ok(Desired {
                enabled: shared.enabled,
                revision: shared.revision,
                bindings: shared.bindings.clone(),
            })
        }
    }

    struct FakeStore {
        shared: Arc<Mutex<SharedSettings>>,
        fail: AtomicBool,
        block: Mutex<Option<Block>>,
    }

    impl FakeStore {
        fn set_fail(&self, fail: bool) {
            self.fail.store(fail, Ordering::SeqCst);
        }

        fn block_next_update(&self) -> (Receiver<()>, Sender<()>) {
            let (entered_tx, entered_rx) = channel();
            let (release_tx, release_rx) = channel();
            *lock(&self.block) = Some(Block {
                entered: entered_tx,
                release: release_rx,
            });
            (entered_rx, release_tx)
        }

        fn snapshot_value(&self) -> Snapshot {
            let shared = lock(&self.shared);
            snapshot_with(shared.enabled, &shared.bindings, shared.revision)
        }
    }

    impl Store for FakeStore {
        fn snapshot(&self) -> Result<Snapshot, String> {
            Ok(self.snapshot_value())
        }

        fn update_bindings(&self, bindings: &ShortcutBindings) -> Result<Snapshot, String> {
            if let Some(block) = lock(&self.block).take() {
                let _ = block.entered.send(());
                let _ = block.release.recv();
            }
            if self.fail.load(Ordering::SeqCst) {
                return Err("save_failed".into());
            }
            let mut shared = lock(&self.shared);
            shared.bindings = bindings.clone();
            shared.revision = shared.revision.saturating_add(1);
            Ok(snapshot_with(
                shared.enabled,
                &shared.bindings,
                shared.revision,
            ))
        }
    }

    fn snapshot_with(enabled: bool, bindings: &ShortcutBindings, revision: u64) -> Snapshot {
        let mut snapshot = Snapshot::default();
        snapshot.settings.global_shortcuts = enabled;
        snapshot.settings.shortcut_bindings = bindings.clone();
        snapshot.revision = revision;
        snapshot
    }

    fn label(shortcut: &ShortcutBinding) -> String {
        format!("{:?}:{}", shortcut.canonical().modifiers, shortcut.key)
    }

    fn key(modifiers: &[ShortcutModifier], key: &str) -> ShortcutBinding {
        ShortcutBinding {
            modifiers: modifiers.to_vec(),
            key: key.into(),
        }
    }

    fn core_with(
        bindings: ShortcutBindings,
        enabled: bool,
    ) -> (
        ShortcutCore,
        FakeBackend,
        FakeSettings,
        FakeStore,
        Arc<Mutex<SharedSettings>>,
    ) {
        let shared = Arc::new(Mutex::new(SharedSettings {
            enabled,
            bindings,
            revision: 0,
        }));
        (
            ShortcutCore::new(),
            FakeBackend::default(),
            FakeSettings {
                shared: shared.clone(),
            },
            FakeStore {
                shared: shared.clone(),
                fail: AtomicBool::new(false),
                block: Mutex::new(None),
            },
            shared,
        )
    }

    fn enabled_core(
        bindings: ShortcutBindings,
    ) -> (
        ShortcutCore,
        FakeBackend,
        FakeSettings,
        FakeStore,
        Arc<Mutex<SharedSettings>>,
    ) {
        let (core, backend, settings, store, shared) = core_with(bindings, true);
        let status = core.reconcile(&backend, &settings);
        assert!(status.toggle.active && status.pause.active);
        (core, backend, settings, store, shared)
    }

    fn active_generation(core: &ShortcutCore, kind: ShortcutKind) -> u64 {
        lock(&core.state).bindings[index(kind)]
            .active_generation
            .expect("active generation")
    }

    fn press(core: &ShortcutCore, kind: ShortcutKind, generation: u64, actions: &AtomicUsize) {
        core.handle_event(kind, generation, ShortcutState::Pressed, || {
            core.run_gated(kind, generation, || {
                actions.fetch_add(1, Ordering::SeqCst);
            });
        });
    }

    fn release(core: &ShortcutCore, kind: ShortcutKind, generation: u64) {
        core.handle_event(kind, generation, ShortcutState::Released, || {});
    }

    #[test]
    fn partial_registration_keeps_healthy_binding_and_retry_only_attempts_failed_one() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, _store, _shared) = core_with(defaults.clone(), true);
        backend.fail_register(&defaults.pause);
        let status = core.reconcile(&backend, &settings);
        assert!(status.toggle.active);
        assert!(!status.pause.active);
        assert_eq!(status.pause.error.as_deref(), Some(REGISTER_FAILED));

        backend.clear_failures();
        backend.clear_calls();
        let status = core.reconcile(&backend, &settings);
        assert!(status.pause.active);
        assert_eq!(backend.calls(), vec![Call::Register(ShortcutKind::Pause)]);
    }

    #[test]
    fn one_press_runs_once_until_release() {
        let (core, _backend, _settings, _store, _shared) =
            enabled_core(ShortcutBindings::default());
        let generation = active_generation(&core, ShortcutKind::Toggle);
        let actions = AtomicUsize::new(0);
        press(&core, ShortcutKind::Toggle, generation, &actions);
        press(&core, ShortcutKind::Toggle, generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 1);
        release(&core, ShortcutKind::Toggle, generation);
        press(&core, ShortcutKind::Toggle, generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn disabling_while_register_is_blocked_finishes_disabled_and_ignores_events() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, _store, shared) = core_with(defaults.clone(), true);
        let core = Arc::new(core);
        let backend = Arc::new(backend);
        let settings = Arc::new(settings);
        let (entered, release_register) = backend.block_next_register();

        let first_core = core.clone();
        let first_backend = backend.clone();
        let first_settings = settings.clone();
        let first =
            std::thread::spawn(move || first_core.reconcile(&*first_backend, &*first_settings));
        entered.recv().unwrap();

        lock(&shared).enabled = false;
        settings.set(false, defaults);
        let second_core = core.clone();
        let second_backend = backend.clone();
        let second_settings = settings.clone();
        let second =
            std::thread::spawn(move || second_core.reconcile(&*second_backend, &*second_settings));

        release_register.send(()).unwrap();
        first.join().unwrap();
        second.join().unwrap();

        let status = core.status();
        assert!(!status.enabled);
        assert!(!status.toggle.active && !status.pause.active);
        assert!(backend.registrations().is_empty());
    }

    #[test]
    fn gated_action_rechecks_generation_at_execution_time() {
        let (core, _backend, _settings, _store, _shared) =
            enabled_core(ShortcutBindings::default());
        let generation = active_generation(&core, ShortcutKind::Toggle);
        core.deactivate_actions();
        let actions = AtomicUsize::new(0);
        assert!(!core.run_gated(ShortcutKind::Toggle, generation, || {
            actions.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(actions.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn unregister_failure_retains_stale_and_retry_cleans_only_failed_binding() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, _store, _shared) = enabled_core(defaults.clone());
        backend.fail_unregister(&defaults.toggle);
        settings.set(false, defaults.clone());
        let status = core.reconcile(&backend, &settings);
        assert!(!status.enabled);
        assert!(!status.toggle.active);
        assert!(status
            .toggle
            .stale_bindings
            .iter()
            .any(|binding| binding.same(&defaults.toggle)));

        backend.clear_failures();
        backend.clear_calls();
        let status = core.reconcile(&backend, &settings);
        assert!(status.toggle.stale_bindings.is_empty());
        assert_eq!(
            backend.calls(),
            vec![Call::Unregister(ShortcutKind::Toggle)]
        );
    }

    #[test]
    fn old_residual_events_do_not_activate_after_new_binding() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, _store, _shared) = enabled_core(defaults.clone());
        let old_generation = active_generation(&core, ShortcutKind::Toggle);
        backend.fail_unregister(&defaults.toggle);
        let next = ShortcutBindings {
            toggle: key(
                &[ShortcutModifier::Control, ShortcutModifier::Shift],
                "KeyK",
            ),
            pause: defaults.pause.clone(),
            ..defaults.clone()
        };
        settings.set(true, next.clone());
        let status = core.reconcile(&backend, &settings);
        assert!(status.toggle.active);
        assert_eq!(status.toggle.active_binding.as_ref().unwrap().key, "KeyK");
        assert!(status
            .toggle
            .stale_bindings
            .iter()
            .any(|binding| binding.same(&defaults.toggle)));

        let new_generation = active_generation(&core, ShortcutKind::Toggle);
        let actions = AtomicUsize::new(0);
        press(&core, ShortcutKind::Toggle, old_generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 0);
        press(&core, ShortcutKind::Toggle, new_generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn candidate_is_inactive_until_persist_succeeds() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, _shared) = enabled_core(defaults);
        let core = Arc::new(core);
        let backend = Arc::new(backend);
        let settings = Arc::new(settings);
        let store = Arc::new(store);
        let next = key(
            &[ShortcutModifier::Control, ShortcutModifier::Shift],
            "KeyK",
        );
        let (entered, release_update) = store.block_next_update();

        let worker_core = core.clone();
        let worker_backend = backend.clone();
        let worker_settings = settings.clone();
        let worker_store = store.clone();
        let worker_binding = next.clone();
        let worker = std::thread::spawn(move || {
            worker_core.update_binding(
                &*worker_backend,
                &*worker_settings,
                &*worker_store,
                ShortcutKind::Toggle,
                worker_binding,
            )
        });
        entered.recv().unwrap();

        let candidate = backend
            .registrations()
            .into_iter()
            .find(|entry| entry.kind == ShortcutKind::Toggle && entry.shortcut.same(&next))
            .expect("candidate registration");
        let actions = AtomicUsize::new(0);
        press(&core, ShortcutKind::Toggle, candidate.generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 0);

        release_update.send(()).unwrap();
        let mutation = worker.join().unwrap().unwrap();
        assert!(mutation.error.is_none());
        let status = mutation.status;
        assert!(status.toggle.active);
        assert_eq!(status.toggle.active_binding.as_ref().unwrap().key, "KeyK");
        press(&core, ShortcutKind::Toggle, candidate.generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn save_failure_restores_old_binding() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, shared) = enabled_core(defaults.clone());
        store.set_fail(true);
        let next = key(
            &[ShortcutModifier::Control, ShortcutModifier::Shift],
            "KeyK",
        );
        let mutation = core
            .update_binding(
                &backend,
                &settings,
                &store,
                ShortcutKind::Toggle,
                next.clone(),
            )
            .unwrap();
        assert_eq!(mutation.error.as_deref(), Some(SAVE_FAILED_TOP));
        assert!(mutation.status.toggle.active);
        assert!(mutation
            .status
            .toggle
            .active_binding
            .as_ref()
            .unwrap()
            .same(&defaults.toggle));
        assert!(backend
            .registrations()
            .iter()
            .any(|entry| entry.shortcut.same(&defaults.toggle)));
        assert!(!backend
            .registrations()
            .iter()
            .any(|entry| entry.shortcut.same(&next)));
        assert!(lock(&shared).bindings.toggle.same(&defaults.toggle));
    }

    #[test]
    fn rollback_failure_reports_saved_old_but_inactive() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, shared) = enabled_core(defaults.clone());
        store.set_fail(true);
        backend.fail_register(&defaults.toggle);
        let next = key(
            &[ShortcutModifier::Control, ShortcutModifier::Shift],
            "KeyK",
        );
        let mutation = core
            .update_binding(&backend, &settings, &store, ShortcutKind::Toggle, next)
            .unwrap();
        assert_eq!(mutation.error.as_deref(), Some(ROLLBACK_FAILED_TOP));
        assert!(mutation
            .snapshot
            .settings
            .shortcut_bindings
            .toggle
            .same(&defaults.toggle));
        assert!(!mutation.status.toggle.active);
        assert_eq!(
            mutation.status.toggle.error.as_deref(),
            Some(REGISTER_FAILED)
        );
        assert!(lock(&shared).bindings.toggle.same(&defaults.toggle));
    }

    #[test]
    fn swap_then_restore_all_rebinds_without_duplicate_registration() {
        let swapped = ShortcutBindings {
            toggle: ShortcutBinding::pause_default(),
            pause: ShortcutBinding::toggle_default(),
            ..ShortcutBindings::default()
        };
        let (core, backend, settings, store, shared) = enabled_core(swapped);
        let mutation = core
            .replace_bindings(&backend, &settings, &store, ShortcutBindings::default())
            .unwrap();
        assert!(mutation.error.is_none());
        assert!(mutation
            .status
            .toggle
            .active_binding
            .as_ref()
            .unwrap()
            .same(&ShortcutBinding::toggle_default()));
        assert!(mutation
            .status
            .pause
            .active_binding
            .as_ref()
            .unwrap()
            .same(&ShortcutBinding::pause_default()));
        assert!(mutation.status.toggle.stale_bindings.is_empty());
        let registrations = backend.registrations();
        assert_eq!(registrations.len(), 2);
        assert!(registrations.iter().any(|entry| {
            entry.kind == ShortcutKind::Toggle
                && entry.shortcut.same(&ShortcutBinding::toggle_default())
        }));
        assert!(registrations.iter().any(|entry| {
            entry.kind == ShortcutKind::Pause
                && entry.shortcut.same(&ShortcutBinding::pause_default())
        }));
        assert!(lock(&shared)
            .bindings
            .toggle
            .same(&ShortcutBinding::toggle_default()));
    }

    #[test]
    fn restore_all_keeps_total_switch_disabled() {
        let custom = ShortcutBindings {
            toggle: key(&[ShortcutModifier::Control], "KeyK"),
            pause: key(&[ShortcutModifier::Control], "KeyP"),
            ..ShortcutBindings::default()
        };
        let (core, backend, settings, store, shared) = core_with(custom, false);
        let status = core.reconcile(&backend, &settings);
        assert!(!status.enabled);
        let mutation = core
            .replace_bindings(&backend, &settings, &store, ShortcutBindings::default())
            .unwrap();
        assert!(mutation.error.is_none());
        assert!(!mutation.snapshot.settings.global_shortcuts);
        assert!(!mutation.status.enabled);
        assert!(!mutation.status.toggle.active && !mutation.status.pause.active);
        assert!(lock(&shared)
            .bindings
            .toggle
            .same(&ShortcutBinding::toggle_default()));
    }

    #[test]
    fn recording_releases_registrations_and_end_restores_them() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, _shared) = enabled_core(defaults);
        let old_generation = active_generation(&core, ShortcutKind::Toggle);
        let mutation = core.begin_recording(&backend, &settings, &store).unwrap();
        assert!(mutation.status.recording);
        assert!(backend.registrations().is_empty());
        let actions = AtomicUsize::new(0);
        press(&core, ShortcutKind::Toggle, old_generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 0);

        let mutation = core.end_recording(&backend, &settings, &store).unwrap();
        assert!(!mutation.status.recording);
        assert!(mutation.status.toggle.active && mutation.status.pause.active);
        assert_eq!(backend.registrations().len(), 2);
        let generation = active_generation(&core, ShortcutKind::Toggle);
        press(&core, ShortcutKind::Toggle, generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn duplicate_update_is_rejected_without_rebinding() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, _shared) = enabled_core(defaults.clone());
        let count = backend.registrations().len();
        let mutation = core
            .update_binding(
                &backend,
                &settings,
                &store,
                ShortcutKind::Toggle,
                defaults.pause,
            )
            .unwrap();
        assert_eq!(mutation.error.as_deref(), Some(DUPLICATE));
        assert_eq!(backend.registrations().len(), count);
        assert!(mutation.status.toggle.active);
    }

    #[test]
    fn single_restore_updates_only_the_binding() {
        let custom = ShortcutBindings {
            toggle: key(&[ShortcutModifier::Control], "KeyK"),
            pause: ShortcutBinding::pause_default(),
            ..ShortcutBindings::default()
        };
        let (core, backend, settings, store, shared) = enabled_core(custom);
        let mutation = core
            .update_binding(
                &backend,
                &settings,
                &store,
                ShortcutKind::Toggle,
                ShortcutBinding::toggle_default(),
            )
            .unwrap();
        assert!(mutation.error.is_none());
        assert!(mutation
            .status
            .toggle
            .active_binding
            .as_ref()
            .unwrap()
            .same(&ShortcutBinding::toggle_default()));
        assert!(mutation.snapshot.settings.global_shortcuts);
        assert!(lock(&shared)
            .bindings
            .toggle
            .same(&ShortcutBinding::toggle_default()));
    }

    #[test]
    fn shutdown_prevents_inflight_registration_from_activating() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, _store, _shared) = core_with(defaults, true);
        let core = Arc::new(core);
        let backend = Arc::new(backend);
        let settings = Arc::new(settings);
        let (entered, release_register) = backend.block_next_register();

        let worker_core = core.clone();
        let worker_backend = backend.clone();
        let worker_settings = settings.clone();
        let worker =
            std::thread::spawn(move || worker_core.reconcile(&*worker_backend, &*worker_settings));
        entered.recv().unwrap();
        core.begin_shutdown();
        release_register.send(()).unwrap();
        worker.join().unwrap();

        let status = core.status();
        assert!(!status.toggle.active);
        assert_eq!(status.toggle.stale_bindings.len(), 1);
        let generation = backend.registrations()[0].generation;
        let actions = AtomicUsize::new(0);
        press(&core, ShortcutKind::Toggle, generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn disabling_one_binding_keeps_other_active_and_persists_flag() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, shared) = enabled_core(defaults.clone());
        let old_toggle_generation = active_generation(&core, ShortcutKind::Toggle);

        let mutation = core
            .set_binding_enabled(&backend, &settings, &store, ShortcutKind::Toggle, false)
            .unwrap();

        assert!(mutation.error.is_none());
        assert!(!mutation.status.toggle.enabled);
        assert!(!mutation.status.toggle.active);
        assert!(mutation.status.pause.enabled);
        assert!(mutation.status.pause.active);
        assert!(!mutation.snapshot.settings.shortcut_bindings.toggle_enabled);
        assert!(!lock(&shared).bindings.toggle_enabled);
        assert_eq!(backend.registrations().len(), 1);
        assert_eq!(
            backend.registrations()[0].shortcut,
            defaults.pause.canonical()
        );

        let actions = AtomicUsize::new(0);
        press(&core, ShortcutKind::Toggle, old_toggle_generation, &actions);
        assert_eq!(actions.load(Ordering::SeqCst), 0);
        press(
            &core,
            ShortcutKind::Pause,
            active_generation(&core, ShortcutKind::Pause),
            &actions,
        );
        assert_eq!(actions.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn reenabling_a_binding_registers_it_and_allows_events() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, _shared) = enabled_core(defaults);

        core.set_binding_enabled(&backend, &settings, &store, ShortcutKind::Toggle, false)
            .unwrap();
        let mutation = core
            .set_binding_enabled(&backend, &settings, &store, ShortcutKind::Toggle, true)
            .unwrap();

        assert!(mutation.error.is_none());
        assert!(mutation.status.toggle.enabled && mutation.status.toggle.active);
        assert!(mutation.status.pause.enabled && mutation.status.pause.active);
        assert_eq!(backend.registrations().len(), 2);

        let actions = AtomicUsize::new(0);
        press(
            &core,
            ShortcutKind::Toggle,
            active_generation(&core, ShortcutKind::Toggle),
            &actions,
        );
        assert_eq!(actions.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn enable_registration_failure_rolls_back_to_disabled() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, _shared) = enabled_core(defaults.clone());
        core.set_binding_enabled(&backend, &settings, &store, ShortcutKind::Toggle, false)
            .unwrap();
        backend.fail_register(&defaults.toggle);

        let mutation = core
            .set_binding_enabled(&backend, &settings, &store, ShortcutKind::Toggle, true)
            .unwrap();

        assert_eq!(mutation.error.as_deref(), Some(REGISTER_FAILED_TOP));
        assert!(!mutation.status.toggle.enabled);
        assert!(!mutation.status.toggle.active);
        assert!(mutation.status.pause.enabled && mutation.status.pause.active);
        assert!(!mutation.snapshot.settings.shortcut_bindings.toggle_enabled);
        assert_eq!(backend.registrations().len(), 1);
    }

    #[test]
    fn disable_persist_failure_restores_enabled_binding() {
        let defaults = ShortcutBindings::default();
        let (core, backend, settings, store, shared) = enabled_core(defaults.clone());
        store.set_fail(true);

        let mutation = core
            .set_binding_enabled(&backend, &settings, &store, ShortcutKind::Toggle, false)
            .unwrap();

        assert_eq!(mutation.error.as_deref(), Some(SAVE_FAILED_TOP));
        assert!(mutation.status.toggle.enabled && mutation.status.toggle.active);
        assert!(mutation.status.pause.enabled && mutation.status.pause.active);
        assert!(mutation.snapshot.settings.shortcut_bindings.toggle_enabled);
        assert!(lock(&shared).bindings.toggle_enabled);
        assert_eq!(backend.registrations().len(), 2);
    }

    #[test]
    fn restore_all_preserves_binding_enabled_flags() {
        let custom = ShortcutBindings {
            toggle: key(&[ShortcutModifier::Control], "KeyK"),
            pause: key(&[ShortcutModifier::Control], "KeyP"),
            toggle_enabled: false,
            pause_enabled: true,
        };
        let (core, backend, settings, store, _shared) = core_with(custom, true);
        let status = core.reconcile(&backend, &settings);
        assert!(!status.toggle.enabled && !status.toggle.active);
        assert!(status.pause.enabled && status.pause.active);

        let mutation = core.restore_all(&backend, &settings, &store).unwrap();

        assert!(mutation.error.is_none());
        assert!(!mutation.status.toggle.enabled);
        assert!(!mutation.status.toggle.active);
        assert!(mutation.status.pause.enabled && mutation.status.pause.active);
        assert!(mutation
            .snapshot
            .settings
            .shortcut_bindings
            .toggle
            .same(&ShortcutBinding::toggle_default()));
        assert!(!mutation.snapshot.settings.shortcut_bindings.toggle_enabled);
        assert!(mutation.snapshot.settings.shortcut_bindings.pause_enabled);
        assert_eq!(backend.registrations().len(), 1);
        assert_eq!(
            backend.registrations()[0].shortcut,
            ShortcutBinding::pause_default().canonical()
        );
    }

    #[test]
    fn single_restore_preserves_disabled_flag() {
        let custom = ShortcutBindings {
            toggle: key(&[ShortcutModifier::Control], "KeyK"),
            pause: ShortcutBinding::pause_default(),
            toggle_enabled: false,
            pause_enabled: true,
        };
        let (core, backend, settings, store, _shared) = core_with(custom, true);
        core.reconcile(&backend, &settings);

        let mutation = core
            .update_binding(
                &backend,
                &settings,
                &store,
                ShortcutKind::Toggle,
                ShortcutBinding::toggle_default(),
            )
            .unwrap();

        assert!(mutation.error.is_none());
        assert!(!mutation.status.toggle.enabled);
        assert!(!mutation.status.toggle.active);
        assert!(mutation.status.pause.active);
        assert!(!mutation.snapshot.settings.shortcut_bindings.toggle_enabled);
        assert_eq!(backend.registrations().len(), 1);
    }
}
