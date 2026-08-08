use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct AnimationEventRecord {
    pub(crate) animation_name: String,
    pub(crate) elapsed_time: f64,
    pub(crate) pseudo_element: String,
    pub(crate) pseudo_target: Option<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct AnimationEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, AnimationEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AnimationEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AnimationEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AnimationEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::animation_event_animation_name_property::define(scope, prototype)?;
    super::animation_event_elapsed_time_property::define(scope, prototype)?;
    super::animation_event_pseudo_element_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let tag_key = v8::Symbol::get_to_string_tag(scope);
    let tag_value = prototype
        .get(scope, tag_key.into())
        .ok_or_else(|| "AnimationEvent tag is unavailable".to_owned())?;
    if prototype.delete(scope, tag_key.into()) != Some(true) {
        return Err("cannot reorder AnimationEvent tag".to_owned());
    }
    super::animation_event_pseudo_target_property::define(scope, prototype)?;
    if prototype.define_own_property(
        scope,
        tag_key.into(),
        tag_value,
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot restore AnimationEvent tag".to_owned());
    }
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AnimationEventStore>()
        .ok_or_else(|| "AnimationEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: Option<v8::Local<'s, v8::Object>>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = object?;
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then_some(value)
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> String {
    property(scope, object, name)
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AnimationEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles =
        property(scope, options, "bubbles").is_some_and(|value| value.boolean_value(scope));
    let cancelable =
        property(scope, options, "cancelable").is_some_and(|value| value.boolean_value(scope));
    let composed =
        property(scope, options, "composed").is_some_and(|value| value.boolean_value(scope));
    let animation_name = string_property(scope, options, "animationName");
    let elapsed_time = property(scope, options, "elapsedTime")
        .and_then(|value| value.number_value(scope))
        .unwrap_or(0.0);
    let pseudo_element = string_property(scope, options, "pseudoElement");
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<AnimationEventStore>()
        .expect("AnimationEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            AnimationEventRecord {
                animation_name,
                elapsed_time,
                pseudo_element,
                pseudo_target: None,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AnimationEventRecord> {
    scope
        .get_slot::<AnimationEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AnimationEventRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_animation_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.animation_name);
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
    match record(scope, arguments.this()) {
        Some(record) => match record.pseudo_target {
            Some(target) => result.set(v8::Local::new(scope, &target).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
