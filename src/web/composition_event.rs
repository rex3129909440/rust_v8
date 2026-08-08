use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CompositionEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) data: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CompositionEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CompositionEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CompositionEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CompositionEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::composition_event_data_property::define(scope, prototype)?;
    super::composition_event_init_composition_event::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::ui_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CompositionEventStore>()
        .ok_or_else(|| "CompositionEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let event_type = crate::webidl::string(scope, "")?;
    constructor
        .new_instance(scope, &[event_type.into()])
        .ok_or_else(|| "cannot create CompositionEvent".to_owned())
}

pub(crate) fn option<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CompositionEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = arguments.get(1);
    let bubbles = option(scope, options, "bubbles").is_some_and(|value| value.boolean_value(scope));
    let cancelable =
        option(scope, options, "cancelable").is_some_and(|value| value.boolean_value(scope));
    let composed =
        option(scope, options, "composed").is_some_and(|value| value.boolean_value(scope));
    let detail = option(scope, options, "detail")
        .and_then(|value| value.int32_value(scope))
        .unwrap_or(0);
    let view = option(scope, options, "view")
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| v8::Global::new(scope, value));
    let data = option(scope, options, "data")
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    super::ui_event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
        view,
        detail,
        None,
    );
    scope
        .get_slot_mut::<CompositionEventStore>()
        .expect("CompositionEvent state")
        .data
        .insert(arguments.this().get_identity_hash().get(), data);
    result.set(arguments.this().into());
}

pub(crate) fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(data) = scope
        .get_slot::<CompositionEventStore>()
        .and_then(|store| store.data.get(&arguments.this().get_identity_hash().get()))
    {
        if let Some(data) = v8::String::new(scope, data) {
            result.set(data.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn init_composition_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<CompositionEventStore>()
        .is_some_and(|store| {
            store
                .data
                .contains_key(&arguments.this().get_identity_hash().get())
        })
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let view = (!arguments.get(3).is_null_or_undefined())
        .then(|| v8::Global::new(scope, arguments.get(3)));
    let data = crate::webidl::value_to_string(scope, arguments.get(4));
    super::ui_event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
        view,
        0,
        None,
    );
    if let Some(current) = scope
        .get_slot_mut::<CompositionEventStore>()
        .and_then(|store| {
            store
                .data
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = data;
    }
}
