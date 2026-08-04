use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TransitionEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TransitionEventRecord>,
}

#[derive(Clone)]
pub(crate) struct TransitionEventRecord {
    pub(crate) property_name: String,
    pub(crate) elapsed_time: f64,
    pub(crate) pseudo_element: String,
    pub(crate) pseudo_target: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TransitionEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TransitionEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TransitionEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TransitionEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::transition_event_property_name_property::define(scope, prototype)?;
    super::transition_event_elapsed_time_property::define(scope, prototype)?;
    super::transition_event_pseudo_element_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::transition_event_pseudo_target_property::define(scope, prototype)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TransitionEventStore>()
        .ok_or_else(|| "TransitionEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'TransitionEvent': use the new operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let property_name = init
        .and_then(|object| string_property(scope, object, "propertyName"))
        .unwrap_or_default();
    let elapsed_time = init
        .map(|object| super::event::number_property(scope, object, "elapsedTime", 0.0))
        .unwrap_or(0.0);
    let pseudo_element = init
        .and_then(|object| string_property(scope, object, "pseudoElement"))
        .unwrap_or_default();
    let pseudo_target = init.and_then(|object| object_property(scope, object, "pseudoTarget"));
    let bubbles =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "bubbles"));
    let cancelable =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "cancelable"));
    let composed =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "composed"));
    let object = arguments.this();
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    let pseudo_target = pseudo_target.map(|value| v8::Global::new(scope, value));
    scope
        .get_slot_mut::<TransitionEventStore>()
        .expect("TransitionEvent state")
        .records
        .insert(
            object.get_identity_hash().get(),
            TransitionEventRecord {
                property_name,
                elapsed_time,
                pseudo_element,
                pseudo_target,
            },
        );
    result.set(object.into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TransitionEventRecord> {
    scope
        .get_slot::<TransitionEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
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

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TransitionEventRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

pub(crate) fn get_property_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.property_name);
}

pub(crate) fn get_pseudo_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.pseudo_element);
}

pub(crate) fn get_elapsed_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.elapsed_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_pseudo_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(target) = record.pseudo_target {
        result.set(v8::Local::new(scope, &target).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
