use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CloseEventRecord {
    pub(crate) was_clean: bool,
    pub(crate) code: u16,
    pub(crate) reason: String,
}

#[derive(Default)]
pub(crate) struct CloseEventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, CloseEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CloseEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CloseEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CloseEventStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CloseEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::close_event_was_clean_property::define(scope, prototype)?;
    super::close_event_code_property::define(scope, prototype)?;
    super::close_event_reason_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CloseEventStore>()
        .ok_or_else(|| "CloseEvent state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn option<'s>(
    scope: &v8::PinScope<'s, '_>,
    options: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = v8::Local::<v8::Object>::try_from(options).ok()?;
    object.get(scope, v8::String::new(scope, name)?.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CloseEvent requires an event type");
        return;
    }
    let options = arguments.get(1);
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = option(scope, options, "bubbles").is_some_and(|value| value.boolean_value(scope));
    let cancelable =
        option(scope, options, "cancelable").is_some_and(|value| value.boolean_value(scope));
    let composed =
        option(scope, options, "composed").is_some_and(|value| value.boolean_value(scope));
    let was_clean =
        option(scope, options, "wasClean").is_some_and(|value| value.boolean_value(scope));
    let code = option(scope, options, "code")
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0) as u16;
    let reason = option(scope, options, "reason")
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<CloseEventStore>()
        .expect("CloseEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CloseEventRecord {
                was_clean,
                code,
                reason,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CloseEventRecord> {
    scope
        .get_slot::<CloseEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_was_clean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.was_clean).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, u32::from(record.code)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_reason(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.reason) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CloseEventStore>() {
        store.constructors.remove(&realm_id);
    }
}
