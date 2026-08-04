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
}

#[derive(Default)]
pub(crate) struct TimerState {
    next_id: i32,
    timeouts: HashMap<i32, TimerRecord>,
    intervals: HashMap<i32, TimerRecord>,
    running_nesting_level: u32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TimerState {
        next_id: 1,
        timeouts: HashMap::new(),
        intervals: HashMap::new(),
        running_nesting_level: 0,
    });
}

pub(crate) fn clear(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    if let Some(state) = scope.get_slot_mut::<TimerState>() {
        state.timeouts.remove(&id);
        state.intervals.remove(&id);
    }
}

pub(crate) fn reserve_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Value>,
    arguments: Vec<v8::Global<v8::Value>>,
    delay_ms: f64,
) -> i32 {
    let callback = timer_callback(scope, callback);
    let context = v8::Global::new(scope, scope.get_current_context());
    let nesting_level = scope
        .get_slot::<TimerState>()
        .map_or(1, |state| state.running_nesting_level.saturating_add(1));
    let delay_ms = timer_delay_for_nesting(delay_ms, nesting_level);
    let due_ms = crate::determinism::elapsed_milliseconds(scope) + delay_ms;
    let Some(state) = scope.get_slot_mut::<TimerState>() else {
        return 0;
    };
    let id = state.next_id;
    state.next_id = state.next_id.saturating_add(1).max(1);
    state.timeouts.insert(
        id,
        TimerRecord {
            callback,
            context,
            arguments,
            delay_ms,
            due_ms,
            nesting_level,
        },
    );
    id
}

pub(crate) fn reserve_interval(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Value>,
    arguments: Vec<v8::Global<v8::Value>>,
    delay_ms: f64,
) -> i32 {
    let callback = timer_callback(scope, callback);
    let context = v8::Global::new(scope, scope.get_current_context());
    let nesting_level = scope
        .get_slot::<TimerState>()
        .map_or(1, |state| state.running_nesting_level.saturating_add(1));
    let delay_ms = timer_delay_for_nesting(delay_ms, nesting_level);
    let due_ms = crate::determinism::elapsed_milliseconds(scope) + delay_ms;
    let Some(state) = scope.get_slot_mut::<TimerState>() else {
        return 0;
    };
    let id = state.next_id;
    state.next_id = state.next_id.saturating_add(1).max(1);
    state.intervals.insert(
        id,
        TimerRecord {
            callback,
            context,
            arguments,
            delay_ms,
            due_ms,
            nesting_level,
        },
    );
    id
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<TimerState>().and_then(|state| {
        state
            .timeouts
            .values()
            .chain(state.intervals.values())
            .map(|record| record.due_ms)
            .min_by(f64::total_cmp)
    })
}

pub(crate) fn run_ready(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let now = crate::determinism::elapsed_milliseconds(scope);
    let mut ready = Vec::new();
    if let Some(state) = scope.get_slot_mut::<TimerState>() {
        let mut timeout_ids = state
            .timeouts
            .iter()
            .filter(|(_, record)| record.due_ms <= now)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        timeout_ids.sort_unstable();
        for id in timeout_ids {
            if let Some(record) = state.timeouts.remove(&id) {
                ready.push((record.due_ms, id, record));
            }
        }
        let mut interval_ids = state
            .intervals
            .iter()
            .filter(|(_, record)| record.due_ms <= now)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        interval_ids.sort_unstable();
        for id in interval_ids {
            if let Some(record) = state.intervals.get_mut(&id) {
                let snapshot = record.clone();
                record.due_ms += record.delay_ms.max(1.0);
                ready.push((snapshot.due_ms, id, snapshot));
            }
        }
    }
    ready.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
    });
    let ran = !ready.is_empty();
    for (_, _, record) in ready {
        if let Some(state) = scope.get_slot_mut::<TimerState>() {
            state.running_nesting_level = record.nesting_level;
        }
        let context = v8::Local::new(scope, &record.context);
        let callback_scope = &mut v8::ContextScope::new(scope, context);
        let receiver: v8::Local<v8::Value> = callback_scope
            .get_current_context()
            .global(callback_scope)
            .into();
        match record.callback {
            TimerCallback::Function(callback) => {
                let callback = v8::Local::new(callback_scope, &callback);
                let arguments = record
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
        callback_scope.perform_microtask_checkpoint();
        if let Some(state) = callback_scope.get_slot_mut::<TimerState>() {
            state.running_nesting_level = 0;
        }
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
