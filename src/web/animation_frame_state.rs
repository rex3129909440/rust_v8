use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AnimationFrameState {
    realms: HashMap<i32, AnimationFrameRealmState>,
}

struct AnimationFrameRealmState {
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
    isolate.set_slot(AnimationFrameState::default());
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<AnimationFrameState>().and_then(|state| {
        state
            .realms
            .values()
            .filter_map(|realm| realm.next_due_ms)
            .min_by(f64::total_cmp)
    })
}

pub(crate) fn cancel(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(realm) = scope
        .get_slot_mut::<AnimationFrameState>()
        .and_then(|state| state.realms.get_mut(&realm_id))
    {
        realm.callbacks.remove(&id);
        if realm.callbacks.is_empty() {
            realm.next_due_ms = None;
        }
    }
}

pub(crate) fn reserve(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
) -> i32 {
    let stored_callback = v8::Global::new(scope, callback);
    let context = v8::Global::new(scope, scope.get_current_context());
    let realm_id = crate::webidl::realm_id(scope);
    let due_ms = crate::determinism::monotonic_snapshot_milliseconds(scope) + FRAME_INTERVAL_MS;
    let Some(state) = scope.get_slot_mut::<AnimationFrameState>() else {
        return 0;
    };
    let realm = state
        .realms
        .entry(realm_id)
        .or_insert_with(|| AnimationFrameRealmState {
            next_id: 1,
            callbacks: HashMap::new(),
            next_due_ms: None,
        });
    let id = realm.next_id;
    realm.next_id = realm.next_id.saturating_add(1).max(1);
    realm.callbacks.insert(
        id,
        AnimationFrameRecord {
            callback: stored_callback,
            context,
        },
    );
    realm.next_due_ms.get_or_insert(due_ms);
    id
}

pub(crate) fn run_ready(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let monotonic_now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let mut ready_realms = scope
        .get_slot::<AnimationFrameState>()
        .map(|state| {
            state
                .realms
                .iter()
                .filter(|(_, realm)| realm.next_due_ms.is_some_and(|due| due <= monotonic_now))
                .map(|(realm_id, _)| *realm_id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ready_realms.sort_unstable();

    let mut ran = false;
    for realm_id in ready_realms {
        let callbacks = scope
            .get_slot_mut::<AnimationFrameState>()
            .and_then(|state| state.realms.get_mut(&realm_id))
            .map(|realm| {
                realm.next_due_ms = None;
                std::mem::take(&mut realm.callbacks)
            })
            .unwrap_or_default();
        if callbacks.is_empty() {
            continue;
        }
        let timestamp = super::performance::now_for_realm_at(scope, realm_id, monotonic_now)
            .unwrap_or_else(|| {
                crate::determinism::relative_high_resolution_milliseconds(scope, monotonic_now, 0.0)
            });
        let mut callbacks = callbacks.into_iter().collect::<Vec<_>>();
        callbacks.sort_by_key(|(id, _)| *id);
        for (_, record) in callbacks {
            let context = v8::Local::new(scope, &record.context);
            let callback_scope = &mut v8::ContextScope::new(scope, context);
            super::animation_timeline::sample_realm_at(callback_scope, realm_id, timestamp);
            let receiver: v8::Local<v8::Value> = callback_scope
                .get_current_context()
                .global(callback_scope)
                .into();
            let timestamp: v8::Local<v8::Value> = v8::Number::new(callback_scope, timestamp).into();
            let callback = v8::Local::new(callback_scope, &record.callback);
            let _ = callback.call(callback_scope, receiver, &[timestamp]);
            callback_scope.perform_microtask_checkpoint();
        }
        ran = true;
    }
    ran
}
