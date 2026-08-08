use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CustomEventRecord {
    pub(crate) detail: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct CustomEventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, CustomEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CustomEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CustomEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CustomEventStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CustomEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::custom_event_detail_property::define(scope, prototype)?;
    super::custom_event_init_custom_event::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CustomEventStore>()
        .ok_or_else(|| "CustomEvent state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let event_type = crate::webidl::string(scope, "")?;
    constructor
        .new_instance(scope, &[event_type.into()])
        .ok_or_else(|| "cannot create CustomEvent".to_owned())
}

pub(crate) fn option_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    options: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = v8::Local::<v8::Object>::try_from(options).ok()?;
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CustomEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = arguments.get(1);
    let bubbles =
        option_value(scope, options, "bubbles").is_some_and(|value| value.boolean_value(scope));
    let cancelable =
        option_value(scope, options, "cancelable").is_some_and(|value| value.boolean_value(scope));
    let composed =
        option_value(scope, options, "composed").is_some_and(|value| value.boolean_value(scope));
    let detail = option_value(scope, options, "detail").unwrap_or_else(|| v8::null(scope).into());
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let detail = v8::Global::new(scope, detail);
    scope
        .get_slot_mut::<CustomEventStore>()
        .expect("CustomEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CustomEventRecord { detail },
        );
    result.set(arguments.this().into());
}

pub(crate) fn get_detail(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(detail) = scope
        .get_slot::<CustomEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.detail.clone())
    {
        result.set(v8::Local::new(scope, &detail));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn init_custom_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !scope.get_slot::<CustomEventStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&arguments.this().get_identity_hash().get())
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let detail = v8::Global::new(scope, arguments.get(3));
    super::event::reinitialize(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
    );
    if let Some(record) = scope.get_slot_mut::<CustomEventStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.detail = detail;
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CustomEventStore>() {
        store.constructors.remove(&realm_id);
    }
}
