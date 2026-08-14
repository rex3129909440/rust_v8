use std::collections::HashMap;

#[derive(Clone, Default)]
struct RemotePlaybackRecord {
    on_connecting: Option<v8::Global<v8::Value>>,
    on_connect: Option<v8::Global<v8::Value>>,
    on_disconnect: Option<v8::Global<v8::Value>>,
    next_watch_id: u32,
}

#[derive(Default)]
pub(crate) struct RemotePlaybackStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RemotePlaybackRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RemotePlaybackStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RemotePlayback", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<RemotePlaybackStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "RemotePlayback",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onconnecting",
        get_on_connecting,
        set_on_connecting,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onconnect",
        get_on_connect,
        set_on_connect,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ondisconnect",
        get_on_disconnect,
        set_on_disconnect,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "cancelWatchAvailability",
        0,
        cancel_watch_availability,
    )?;
    crate::webidl::define_method(scope, prototype, "prompt", 0, prompt)?;
    crate::webidl::define_method(scope, prototype, "watchAvailability", 1, watch_availability)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RemotePlaybackStore>()
        .ok_or_else(|| "RemotePlayback state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create RemotePlayback".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<RemotePlaybackStore>()
        .ok_or_else(|| "RemotePlayback state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            RemotePlaybackRecord::default(),
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RemotePlaybackRecord> {
    scope
        .get_slot::<RemotePlaybackStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        if let Some(value) = v8::String::new(scope, "disconnected") {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    }
}

fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&RemotePlaybackRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_on_connecting(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.on_connecting.clone());
}
fn set_on_connecting(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(s, a.get(0));
    if let Some(record) = s
        .get_slot_mut::<RemotePlaybackStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.on_connecting = value;
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}
fn get_on_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.on_connect.clone());
}
fn set_on_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(s, a.get(0));
    if let Some(record) = s
        .get_slot_mut::<RemotePlaybackStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.on_connect = value;
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}
fn get_on_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.on_disconnect.clone());
}
fn set_on_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(s, a.get(0));
    if let Some(record) = s
        .get_slot_mut::<RemotePlaybackStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.on_disconnect = value;
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}

fn resolved_undefined(scope: &mut v8::PinScope<'_, '_>, mut result: v8::ReturnValue<'_>) {
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn cancel_watch_availability(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        resolved_undefined(scope, result);
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "RemotePlayback",
            "cancelWatchAvailability",
            result,
        );
    }
}

fn prompt(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        resolved_undefined(scope, result);
    } else {
        crate::webidl::reject_illegal_invocation_promise(scope, "RemotePlayback", "prompt", result);
    }
}

fn watch_availability(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "RemotePlayback",
            "watchAvailability",
            result,
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The availability callback must be a function");
        return;
    };
    let watch_id = if let Some(record) =
        scope
            .get_slot_mut::<RemotePlaybackStore>()
            .and_then(|store| {
                store
                    .records
                    .get_mut(&arguments.this().get_identity_hash().get())
            }) {
        record.next_watch_id = record.next_watch_id.saturating_add(1);
        record.next_watch_id
    } else {
        return;
    };
    let receiver = v8::undefined(scope);
    let available = v8::Boolean::new(scope, false);
    let _ = callback.call(scope, receiver.into(), &[available.into()]);
    let value = v8::Integer::new_from_unsigned(scope, watch_id);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}
