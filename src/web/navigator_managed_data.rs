use std::collections::HashMap;
#[derive(Clone, Default)]
struct ManagedRecord {
    handler: Option<v8::Global<v8::Value>>,
    values: HashMap<String, String>,
}
#[derive(Default)]
pub(crate) struct NavigatorManagedDataStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ManagedRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigatorManagedDataStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigatorManagedData", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<NavigatorManagedDataStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NavigatorManagedData",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onmanagedconfigurationchange",
        get_handler,
        set_handler,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getManagedConfiguration",
        1,
        get_configuration,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<NavigatorManagedDataStore>()
        .ok_or_else(|| "NavigatorManagedData state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(s)?;
    let prototype = crate::webidl::prototype(s, constructor)?;
    let object = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, object, prototype.into()) != Some(true) {
        return Err("cannot create NavigatorManagedData".to_owned());
    }
    super::event_target::attach(s, object);
    s.get_slot_mut::<NavigatorManagedDataStore>()
        .ok_or_else(|| "NavigatorManagedData state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), ManagedRecord::default());
    Ok(object)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ManagedRecord> {
    s.get_slot::<NavigatorManagedDataStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.handler, r)
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<NavigatorManagedDataStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = handler
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_configuration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "NavigatorManagedData",
            "getManagedConfiguration",
            r,
        );
        return;
    };
    let object = v8::Object::new(s);
    if let Ok(keys) = v8::Local::<v8::Array>::try_from(a.get(0)) {
        for index in 0..keys.length() {
            if let Some(value) = keys.get_index(s, index) {
                let name = crate::webidl::value_to_string(s, value);
                if let Some(stored) = v.values.get(&name) {
                    if let (Some(key), Some(value)) =
                        (v8::String::new(s, &name), v8::String::new(s, stored))
                    {
                        let _ = object.set(s, key.into(), value.into());
                    }
                }
            }
        }
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(s, object.into()) {
        r.set(promise.into())
    }
}
