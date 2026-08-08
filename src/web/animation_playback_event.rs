use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct AnimationPlaybackEventRecord {
    pub(crate) current_time: Option<f64>,
    pub(crate) timeline_time: Option<f64>,
}

#[derive(Default)]
pub(crate) struct AnimationPlaybackEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, AnimationPlaybackEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationPlaybackEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AnimationPlaybackEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AnimationPlaybackEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AnimationPlaybackEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::animation_playback_event_current_time_property::define(scope, prototype)?;
    super::animation_playback_event_timeline_time_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AnimationPlaybackEventStore>()
        .ok_or_else(|| "AnimationPlaybackEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn option_number(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<f64> {
    let options = options?;
    let key = v8::String::new(scope, name)?;
    let value = options.get(scope, key.into())?;
    if value.is_null_or_undefined() {
        None
    } else {
        value.number_value(scope)
    }
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AnimationPlaybackEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let current_time = option_number(scope, options, "currentTime");
    let timeline_time = option_number(scope, options, "timelineTime");
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<AnimationPlaybackEventStore>()
        .expect("AnimationPlaybackEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            AnimationPlaybackEventRecord {
                current_time,
                timeline_time,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AnimationPlaybackEventRecord> {
    scope
        .get_slot::<AnimationPlaybackEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_optional_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AnimationPlaybackEventRecord) -> Option<f64>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Number::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_optional_number(scope, arguments, result, |record| record.current_time);
}

pub(crate) fn get_timeline_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_optional_number(scope, arguments, result, |record| record.timeline_time);
}
