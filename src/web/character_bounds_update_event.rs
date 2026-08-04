use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(crate) struct BoundsRecord {
    pub(crate) start: u32,
    pub(crate) end: u32,
}

#[derive(Default)]
pub(crate) struct CharacterBoundsUpdateEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, BoundsRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CharacterBoundsUpdateEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CharacterBoundsUpdateEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CharacterBoundsUpdateEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CharacterBoundsUpdateEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::character_bounds_update_event_range_start_property::define(scope, prototype)?;
    super::character_bounds_update_event_range_end_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CharacterBoundsUpdateEventStore>()
        .ok_or_else(|| "CharacterBoundsUpdateEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn option_u32(
    scope: &v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Value>,
    name: &str,
) -> u32 {
    let Ok(object) = v8::Local::<v8::Object>::try_from(options) else {
        return 0;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return 0;
    };
    object
        .get(scope, key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0)
}

pub(crate) fn option_bool(
    scope: &v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Value>,
    name: &str,
) -> bool {
    let Ok(object) = v8::Local::<v8::Object>::try_from(options) else {
        return false;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return false;
    };
    object
        .get(scope, key.into())
        .is_some_and(|value| value.boolean_value(scope))
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CharacterBoundsUpdateEvent requires an event type");
        return;
    }
    let options = arguments.get(1);
    let start = option_u32(scope, options, "rangeStart");
    let end = option_u32(scope, options, "rangeEnd");
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        option_bool(scope, options, "bubbles"),
        option_bool(scope, options, "cancelable"),
        option_bool(scope, options, "composed"),
    );
    scope
        .get_slot_mut::<CharacterBoundsUpdateEventStore>()
        .expect("CharacterBoundsUpdateEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            BoundsRecord { start, end },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BoundsRecord> {
    scope
        .get_slot::<CharacterBoundsUpdateEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn get_range_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.start).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_range_end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.end).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
