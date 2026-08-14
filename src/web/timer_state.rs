use std::collections::HashMap;

#[derive(Clone)]
enum TimerCallback {
    Function(v8::Global<v8::Function>),
    Source(String),
}

#[derive(Clone)]
struct TimerRecord {
    callback: TimerCallback,
    context: v8::Global<v8::Context>,
    arguments: Vec<v8::Global<v8::Value>>,
    delay_ms: f64,
    due_ms: f64,
    nesting_level: u32,
    sequence: u64,
}

struct RealmTimerState {
    next_id: i32,
    timeouts: HashMap<i32, TimerRecord>,
    intervals: HashMap<i32, TimerRecord>,
    running_nesting_level: Option<u32>,
}

#[derive(Default)]
pub(crate) struct TimerState {
    realms: HashMap<i32, RealmTimerState>,
    next_sequence: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TimerState::default());
}

fn new_realm_state() -> RealmTimerState {
    RealmTimerState {
        next_id: 1,
        timeouts: HashMap::new(),
        intervals: HashMap::new(),
        running_nesting_level: None,
    }
}

pub(crate) fn clear(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(realm) = scope
        .get_slot_mut::<TimerState>()
        .and_then(|state| state.realms.get_mut(&realm_id))
    {
        realm.timeouts.remove(&id);
        realm.intervals.remove(&id);
    }
}

pub(crate) fn reserve_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Value>,
    arguments: Vec<v8::Global<v8::Value>>,
    delay_ms: f64,
) -> i32 {
    reserve(scope, callback, arguments, delay_ms, false)
}

pub(crate) fn reserve_interval(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Value>,
    arguments: Vec<v8::Global<v8::Value>>,
    delay_ms: f64,
) -> i32 {
    reserve(scope, callback, arguments, delay_ms, true)
}

fn reserve(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Value>,
    arguments: Vec<v8::Global<v8::Value>>,
    delay_ms: f64,
    repeating: bool,
) -> i32 {
    let callback = timer_callback(scope, callback);
    let context = v8::Global::new(scope, scope.get_current_context());
    let realm_id = crate::webidl::realm_id(scope);
    let nesting_level = scope
        .get_slot::<TimerState>()
        .and_then(|state| state.realms.get(&realm_id))
        .and_then(|realm| realm.running_nesting_level)
        .map_or(0, |level| level.saturating_add(1));
    let delay_ms = timer_delay_for_nesting(delay_ms, nesting_level);
    let due_ms = crate::determinism::monotonic_snapshot_milliseconds(scope) + delay_ms;
    let Some(state) = scope.get_slot_mut::<TimerState>() else {
        return 0;
    };
    let sequence = state.next_sequence;
    state.next_sequence = sequence.saturating_add(1);
    let realm = state.realms.entry(realm_id).or_insert_with(new_realm_state);
    let id = realm.next_id;
    realm.next_id = realm.next_id.saturating_add(1).max(1);
    let record = TimerRecord {
        callback,
        context,
        arguments,
        delay_ms,
        due_ms,
        nesting_level,
        sequence,
    };
    if repeating {
        realm.intervals.insert(id, record);
    } else {
        realm.timeouts.insert(id, record);
    }
    id
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<TimerState>().and_then(|state| {
        state
            .realms
            .values()
            .flat_map(|realm| realm.timeouts.values().chain(realm.intervals.values()))
            .map(|record| record.due_ms)
            .min_by(f64::total_cmp)
    })
}

pub(crate) fn run_ready(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let mut ready = scope
        .get_slot::<TimerState>()
        .map(|state| {
            let mut ready = Vec::new();
            for (realm_id, realm) in &state.realms {
                ready.extend(
                    realm
                        .timeouts
                        .iter()
                        .filter(|(_, timer)| timer.due_ms <= now)
                        .map(|(id, timer)| (timer.due_ms, timer.sequence, *realm_id, *id, false)),
                );
                ready.extend(
                    realm
                        .intervals
                        .iter()
                        .filter(|(_, timer)| timer.due_ms <= now)
                        .map(|(id, timer)| (timer.due_ms, timer.sequence, *realm_id, *id, true)),
                );
            }
            ready
        })
        .unwrap_or_default();
    ready.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.3.cmp(&right.3))
    });

    let mut ran = false;
    for (_, _, realm_id, id, repeating) in ready {
        let timer = scope
            .get_slot_mut::<TimerState>()
            .and_then(|state| state.realms.get_mut(&realm_id))
            .and_then(|realm| {
                if repeating {
                    let timer = realm.intervals.get_mut(&id)?;
                    let snapshot = timer.clone();
                    timer.due_ms += timer.delay_ms.max(1.0);
                    Some(snapshot)
                } else {
                    realm.timeouts.remove(&id)
                }
            });
        let Some(timer) = timer else {
            continue;
        };
        if let Some(realm) = scope
            .get_slot_mut::<TimerState>()
            .and_then(|state| state.realms.get_mut(&realm_id))
        {
            realm.running_nesting_level = Some(timer.nesting_level);
        }
        let context = v8::Local::new(scope, &timer.context);
        let callback_scope = &mut v8::ContextScope::new(scope, context);
        super::animation_frame_state::sample_task_realm(callback_scope, realm_id);
        let task_start = super::performance_observer::task_start(callback_scope);
        let receiver: v8::Local<v8::Value> = callback_scope
            .get_current_context()
            .global(callback_scope)
            .into();
        match timer.callback {
            TimerCallback::Function(callback) => {
                let callback = v8::Local::new(callback_scope, &callback);
                let arguments = timer
                    .arguments
                    .iter()
                    .map(|argument| v8::Local::new(callback_scope, argument))
                    .collect::<Vec<_>>();
                let _ = callback.call(callback_scope, receiver, &arguments);
            }
            TimerCallback::Source(source) => {
                if let Some(source) = v8::String::new(callback_scope, &source)
                    && let Some(script) = v8::Script::compile(callback_scope, source, None)
                {
                    let _ = script.run(callback_scope);
                }
            }
        }
        // Promise reactions queued by a timer callback still run inside the
        // same HTML timer task and therefore inherit its nesting level.  Only
        // clear the task-local level after both microtask checkpoints.
        callback_scope.perform_microtask_checkpoint();
        if super::performance_observer::record_completed_task(callback_scope, task_start, false) {
            callback_scope.perform_microtask_checkpoint();
        }
        if let Some(realm) = callback_scope
            .get_slot_mut::<TimerState>()
            .and_then(|state| state.realms.get_mut(&realm_id))
        {
            realm.running_nesting_level = None;
        }
        ran = true;
    }
    ran
}

fn timer_callback(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Value>,
) -> TimerCallback {
    v8::Local::<v8::Function>::try_from(callback).map_or_else(
        |_| TimerCallback::Source(crate::webidl::value_to_string(scope, callback)),
        |callback| TimerCallback::Function(v8::Global::new(scope, callback)),
    )
}

pub(crate) fn normalized_delay(delay_ms: f64) -> f64 {
    if !delay_ms.is_finite() || delay_ms == 0.0 {
        0.0
    } else {
        let truncated = delay_ms.trunc();
        let unsigned = truncated.rem_euclid(4_294_967_296.0);
        let signed = if unsigned >= 2_147_483_648.0 {
            unsigned - 4_294_967_296.0
        } else {
            unsigned
        };
        signed.max(0.0)
    }
}

pub(crate) fn timer_delay_for_nesting(delay_ms: f64, nesting_level: u32) -> f64 {
    let delay_ms = normalized_delay(delay_ms);
    if nesting_level > 5 {
        delay_ms.max(4.0)
    } else {
        delay_ms
    }
}
