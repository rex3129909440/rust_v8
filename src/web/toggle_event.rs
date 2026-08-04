use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ToggleEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, ToggleEventRecord>,
}

#[derive(Clone)]
pub(crate) struct ToggleEventRecord {
    pub(crate) old_state: String,
    pub(crate) new_state: String,
    pub(crate) source: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ToggleEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ToggleEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ToggleEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ToggleEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::toggle_event_old_state_property::define(scope, prototype)?;
    super::toggle_event_new_state_property::define(scope, prototype)?;
    super::toggle_event_source_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ToggleEventStore>()
        .ok_or_else(|| "ToggleEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "ToggleEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let old_state = init
        .and_then(|v| string_property(scope, v, "oldState"))
        .unwrap_or_default();
    let new_state = init
        .and_then(|v| string_property(scope, v, "newState"))
        .unwrap_or_default();
    let source = init
        .and_then(|v| object_property(scope, v, "source"))
        .map(|v| v8::Global::new(scope, v));
    let bubbles = init.is_some_and(|v| super::event::boolean_property(scope, v, "bubbles"));
    let cancelable = init.is_some_and(|v| super::event::boolean_property(scope, v, "cancelable"));
    let composed = init.is_some_and(|v| super::event::boolean_property(scope, v, "composed"));
    let object = arguments.this();
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    scope
        .get_slot_mut::<ToggleEventStore>()
        .expect("ToggleEvent state")
        .records
        .insert(
            object.get_identity_hash().get(),
            ToggleEventRecord {
                old_state,
                new_state,
                source,
            },
        );
    result.set(object.into());
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}
pub(crate) fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null() || value.is_undefined() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(value).ok()
    }
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ToggleEventRecord> {
    scope
        .get_slot::<ToggleEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ToggleEventRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}
pub(crate) fn get_old_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.old_state);
}
pub(crate) fn get_new_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.new_state);
}
pub(crate) fn get_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.source {
        result.set(v8::Local::new(scope, &value).into())
    } else {
        result.set(v8::null(scope).into())
    }
}
