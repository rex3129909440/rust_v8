use std::collections::HashMap;

struct IdleCallbackRecord {
    callback: v8::Global<v8::Function>,
    context: v8::Global<v8::Context>,
    timeout_due_ms: Option<f64>,
    not_before_ms: f64,
}

struct IdleCallbackRealmState {
    next_id: i32,
    callbacks: HashMap<i32, IdleCallbackRecord>,
}

#[derive(Default)]
pub(crate) struct IdleCallbackState {
    realms: HashMap<i32, IdleCallbackRealmState>,
    active_deadline_ms: Option<f64>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdleCallbackState::default());
}

pub(crate) fn cancel(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(realm) = scope
        .get_slot_mut::<IdleCallbackState>()
        .and_then(|state| state.realms.get_mut(&realm_id))
    {
        realm.callbacks.remove(&id);
    }
}

pub(crate) fn reserve(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
    timeout_ms: Option<f64>,
) -> i32 {
    let stored_callback = v8::Global::new(scope, callback);
    let context = v8::Global::new(scope, scope.get_current_context());
    let realm_id = crate::webidl::realm_id(scope);
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let timeout_due_ms = timeout_ms.map(|timeout| now + timeout.max(0.0));
    let Some(state) = scope.get_slot_mut::<IdleCallbackState>() else {
        return 0;
    };
    let active_deadline = state.active_deadline_ms;
    let realm = state
        .realms
        .entry(realm_id)
        .or_insert_with(|| IdleCallbackRealmState {
            next_id: 1,
            callbacks: HashMap::new(),
        });
    let id = realm.next_id;
    realm.next_id = realm.next_id.saturating_add(1).max(1);
    realm.callbacks.insert(
        id,
        IdleCallbackRecord {
            callback: stored_callback,
            context,
            timeout_due_ms,
            not_before_ms: active_deadline.unwrap_or(now),
        },
    );
    id
}

pub(crate) fn run(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let mut callbacks = scope
        .get_slot_mut::<IdleCallbackState>()
        .map(|state| {
            let mut callbacks = Vec::new();
            for (realm_id, realm) in &mut state.realms {
                let mut pending = HashMap::new();
                for (id, callback) in std::mem::take(&mut realm.callbacks) {
                    let timed_out = callback.timeout_due_ms.is_some_and(|due| due <= now);
                    if timed_out || callback.not_before_ms <= now {
                        callbacks.push((*realm_id, id, callback));
                    } else {
                        pending.insert(id, callback);
                    }
                }
                realm.callbacks = pending;
            }
            callbacks
        })
        .unwrap_or_default();
    callbacks.sort_by_key(|(realm_id, id, _)| (*realm_id, *id));
    let ran = !callbacks.is_empty();
    for (realm_id, _, record) in callbacks {
        let context = v8::Local::new(scope, &record.context);
        let callback_scope = &mut v8::ContextScope::new(scope, context);
        super::animation_frame_state::sample_task_realm(callback_scope, realm_id);
        let task_start = super::performance_observer::task_start(callback_scope);
        let did_timeout = record.timeout_due_ms.is_some_and(|due| due <= now);
        let deadline_ms = if did_timeout {
            now
        } else {
            let rendering_deadline = super::animation_frame_state::next_rendering_opportunity(
                callback_scope,
                realm_id,
                now,
            );
            (now + 50.0).min(rendering_deadline)
        };
        let Ok(deadline) =
            super::idle_deadline::create_at(callback_scope, did_timeout, deadline_ms)
        else {
            continue;
        };
        let receiver: v8::Local<v8::Value> = callback_scope
            .get_current_context()
            .global(callback_scope)
            .into();
        let callback = v8::Local::new(callback_scope, &record.callback);
        if let Some(state) = callback_scope.get_slot_mut::<IdleCallbackState>() {
            state.active_deadline_ms = Some(deadline_ms);
        }
        let _ = callback.call(callback_scope, receiver, &[deadline.into()]);
        if let Some(state) = callback_scope.get_slot_mut::<IdleCallbackState>() {
            state.active_deadline_ms = None;
        }
        callback_scope.perform_microtask_checkpoint();
        if super::performance_observer::record_completed_task(callback_scope, task_start, false) {
            callback_scope.perform_microtask_checkpoint();
        }
    }
    ran
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<IdleCallbackState>().and_then(|state| {
        state
            .realms
            .values()
            .flat_map(|realm| realm.callbacks.values())
            .map(|callback| {
                callback
                    .timeout_due_ms
                    .map_or(callback.not_before_ms, |timeout| {
                        timeout.min(callback.not_before_ms)
                    })
            })
            .min_by(f64::total_cmp)
    })
}
