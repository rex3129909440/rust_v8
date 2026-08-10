use std::collections::HashMap;

struct IdleCallbackRecord {
    callback: v8::Global<v8::Function>,
    context: v8::Global<v8::Context>,
    timeout_due_ms: Option<f64>,
}

struct IdleCallbackRealmState {
    next_id: i32,
    callbacks: HashMap<i32, IdleCallbackRecord>,
}

#[derive(Default)]
pub(crate) struct IdleCallbackState {
    realms: HashMap<i32, IdleCallbackRealmState>,
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
    let timeout_due_ms = timeout_ms.map(|timeout| {
        crate::determinism::monotonic_snapshot_milliseconds(scope) + timeout.max(0.0)
    });
    let Some(state) = scope.get_slot_mut::<IdleCallbackState>() else {
        return 0;
    };
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
                callbacks.extend(
                    std::mem::take(&mut realm.callbacks)
                        .into_iter()
                        .map(|(id, callback)| (*realm_id, id, callback)),
                );
            }
            callbacks
        })
        .unwrap_or_default();
    callbacks.sort_by_key(|(realm_id, id, _)| (*realm_id, *id));
    let ran = !callbacks.is_empty();
    for (_, _, record) in callbacks {
        let context = v8::Local::new(scope, &record.context);
        let callback_scope = &mut v8::ContextScope::new(scope, context);
        let did_timeout = record.timeout_due_ms.is_some_and(|due| due <= now);
        let deadline = if did_timeout { now } else { now + 50.0 };
        let Ok(deadline) = super::idle_deadline::create_at(callback_scope, did_timeout, deadline)
        else {
            continue;
        };
        let receiver: v8::Local<v8::Value> = callback_scope
            .get_current_context()
            .global(callback_scope)
            .into();
        let callback = v8::Local::new(callback_scope, &record.callback);
        let _ = callback.call(callback_scope, receiver, &[deadline.into()]);
        callback_scope.perform_microtask_checkpoint();
    }
    ran
}
