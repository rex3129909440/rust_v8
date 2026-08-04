use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdleCallbackState {
    next_id: i32,
    callbacks: HashMap<i32, v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdleCallbackState {
        next_id: 1,
        callbacks: HashMap::new(),
    });
}

pub(crate) fn cancel(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    if let Some(state) = scope.get_slot_mut::<IdleCallbackState>() {
        state.callbacks.remove(&id);
    }
}

pub(crate) fn reserve(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
) -> i32 {
    let stored_callback = v8::Global::new(scope, callback);
    let Some(state) = scope.get_slot_mut::<IdleCallbackState>() else {
        return 0;
    };
    let id = state.next_id;
    state.next_id = state.next_id.saturating_add(1).max(1);
    state.callbacks.insert(id, stored_callback);
    id
}

pub(crate) fn run(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let callbacks = scope
        .get_slot_mut::<IdleCallbackState>()
        .map(|state| std::mem::take(&mut state.callbacks))
        .unwrap_or_default();
    let ran = !callbacks.is_empty();
    let receiver: v8::Local<v8::Value> = scope.get_current_context().global(scope).into();
    let mut callbacks = callbacks.into_iter().collect::<Vec<_>>();
    callbacks.sort_by_key(|(id, _)| *id);
    for (_, callback) in callbacks {
        let Ok(deadline) = super::idle_deadline::create(scope, false, 50.0) else {
            continue;
        };
        let callback = v8::Local::new(scope, &callback);
        let _ = callback.call(scope, receiver, &[deadline.into()]);
    }
    ran
}
