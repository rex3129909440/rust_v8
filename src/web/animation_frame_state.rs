use std::collections::HashMap;

pub(crate) struct AnimationFrameState {
    next_id: i32,
    callbacks: HashMap<i32, AnimationFrameRecord>,
    next_due_ms: Option<f64>,
}

struct AnimationFrameRecord {
    callback: v8::Global<v8::Function>,
    context: v8::Global<v8::Context>,
}

const FRAME_INTERVAL_MS: f64 = 1_000.0 / 60.0;

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationFrameState {
        next_id: 1,
        callbacks: HashMap::new(),
        next_due_ms: None,
    });
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope
        .get_slot::<AnimationFrameState>()
        .and_then(|state| state.next_due_ms)
}

pub(crate) fn cancel(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    if let Some(state) = scope.get_slot_mut::<AnimationFrameState>() {
        state.callbacks.remove(&id);
        if state.callbacks.is_empty() {
            state.next_due_ms = None;
        }
    }
}

pub(crate) fn reserve(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
) -> i32 {
    let stored_callback = v8::Global::new(scope, callback);
    let context = v8::Global::new(scope, scope.get_current_context());
    let due_ms = crate::determinism::elapsed_milliseconds(scope) + FRAME_INTERVAL_MS;
    let Some(state) = scope.get_slot_mut::<AnimationFrameState>() else {
        return 0;
    };
    let id = state.next_id;
    state.next_id = state.next_id.saturating_add(1).max(1);
    state.callbacks.insert(
        id,
        AnimationFrameRecord {
            callback: stored_callback,
            context,
        },
    );
    state.next_due_ms.get_or_insert(due_ms);
    id
}

pub(crate) fn run_ready(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let now = crate::determinism::elapsed_milliseconds(scope);
    if !scope
        .get_slot::<AnimationFrameState>()
        .and_then(|state| state.next_due_ms)
        .is_some_and(|due| due <= now)
    {
        return false;
    }
    let callbacks = scope
        .get_slot_mut::<AnimationFrameState>()
        .map(|state| {
            state.next_due_ms = None;
            std::mem::take(&mut state.callbacks)
        })
        .unwrap_or_default();
    let ran = !callbacks.is_empty();
    let mut callbacks = callbacks.into_iter().collect::<Vec<_>>();
    callbacks.sort_by_key(|(id, _)| *id);
    for (_, record) in callbacks {
        let context = v8::Local::new(scope, &record.context);
        let callback_scope = &mut v8::ContextScope::new(scope, context);
        let receiver: v8::Local<v8::Value> = callback_scope
            .get_current_context()
            .global(callback_scope)
            .into();
        let timestamp =
            super::performance::now_for_current_realm(callback_scope).unwrap_or_else(|| {
                crate::determinism::relative_high_resolution_milliseconds(
                    callback_scope,
                    crate::determinism::elapsed_milliseconds(callback_scope),
                    0.0,
                )
            });
        let timestamp: v8::Local<v8::Value> = v8::Number::new(callback_scope, timestamp).into();
        let callback = v8::Local::new(callback_scope, &record.callback);
        let _ = callback.call(callback_scope, receiver, &[timestamp]);
        callback_scope.perform_microtask_checkpoint();
    }
    ran
}
