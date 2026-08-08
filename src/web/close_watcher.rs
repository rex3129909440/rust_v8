use std::collections::HashMap;

#[derive(Clone, Default)]
struct CloseWatcherRecord {
    oncancel: Option<v8::Global<v8::Function>>,
    onclose: Option<v8::Global<v8::Function>>,
    active: bool,
}

#[derive(Default)]
pub(crate) struct CloseWatcherStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CloseWatcherRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CloseWatcherStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CloseWatcher", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CloseWatcherStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CloseWatcher",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "oncancel", get_oncancel, set_oncancel)?;
    crate::webidl::define_accessor(scope, prototype, "onclose", get_onclose, set_onclose)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "destroy", 0, destroy)?;
    crate::webidl::define_method(scope, prototype, "requestClose", 0, request_close)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CloseWatcherStore>()
        .ok_or_else(|| "CloseWatcher state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "CloseWatcher requires new");
        return;
    }
    super::event_target::attach(scope, arguments.this());
    scope
        .get_slot_mut::<CloseWatcherStore>()
        .expect("CloseWatcher state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CloseWatcherRecord {
                active: true,
                ..CloseWatcherRecord::default()
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CloseWatcherRecord> {
    scope
        .get_slot::<CloseWatcherStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn handler_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    cancel: bool,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let handler = if cancel {
        record.oncancel
    } else {
        record.onclose
    };
    match handler {
        Some(handler) => result.set(v8::Local::new(scope, &handler).into()),
        None => result.set(v8::null(scope).into()),
    }
}

fn get_oncancel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, true);
}

fn get_onclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, false);
}

fn handler_set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    cancel: bool,
) {
    let handler = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|handler| v8::Global::new(scope, handler));
    let Some(record) = scope.get_slot_mut::<CloseWatcherStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if cancel {
        record.oncancel = handler;
    } else {
        record.onclose = handler;
    }
}

fn set_oncancel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, true);
}

fn set_onclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, false);
}

fn invoke_handler(
    scope: &mut v8::PinScope<'_, '_>,
    watcher: v8::Local<'_, v8::Object>,
    handler: Option<v8::Global<v8::Function>>,
    event: v8::Local<'_, v8::Object>,
) {
    if let Some(handler) = handler {
        let handler = v8::Local::new(scope, &handler);
        let _ = handler.call(scope, watcher.into(), &[event.into()]);
    }
}

fn fire_close(scope: &mut v8::PinScope<'_, '_>, watcher: v8::Local<'_, v8::Object>) {
    let Some(record) = record(scope, watcher) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.active {
        return;
    }
    let event = super::event_target::create_event(scope, "close");
    super::event_target::dispatch(scope, watcher, event);
    invoke_handler(scope, watcher, record.onclose, event);
    if let Some(current) = scope
        .get_slot_mut::<CloseWatcherStore>()
        .and_then(|store| store.records.get_mut(&watcher.get_identity_hash().get()))
    {
        current.active = false;
    }
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    fire_close(scope, arguments.this());
}

fn destroy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<CloseWatcherStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.active = false;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn request_close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.active {
        return;
    }
    let event = super::event_target::create_event(scope, "cancel");
    super::event::reinitialize(scope, event, "cancel".to_owned(), false, true, false);
    let allowed = super::event_target::dispatch(scope, arguments.this(), event);
    invoke_handler(scope, arguments.this(), record.oncancel, event);
    if allowed && !super::event::default_prevented(scope, event).unwrap_or(false) {
        fire_close(scope, arguments.this());
    }
}
