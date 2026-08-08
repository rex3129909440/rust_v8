use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct KeyboardEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, KeyboardRecord>,
}

#[derive(Clone)]
pub(crate) struct KeyboardRecord {
    pub(crate) key: String,
    pub(crate) code: String,
    pub(crate) location: u32,
    pub(crate) ctrl_key: bool,
    pub(crate) shift_key: bool,
    pub(crate) alt_key: bool,
    pub(crate) meta_key: bool,
    pub(crate) repeat: bool,
    pub(crate) is_composing: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(KeyboardEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "KeyboardEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<KeyboardEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "KeyboardEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::keyboard_event_key_property::define(scope, prototype)?;
    super::keyboard_event_code_property::define(scope, prototype)?;
    super::keyboard_event_location_property::define(scope, prototype)?;
    super::keyboard_event_ctrl_key_property::define(scope, prototype)?;
    super::keyboard_event_shift_key_property::define(scope, prototype)?;
    super::keyboard_event_alt_key_property::define(scope, prototype)?;
    super::keyboard_event_meta_key_property::define(scope, prototype)?;
    super::keyboard_event_repeat_property::define(scope, prototype)?;
    super::keyboard_event_is_composing_property::define(scope, prototype)?;
    super::keyboard_event_char_code_property::define(scope, prototype)?;
    super::keyboard_event_key_code_property::define(scope, prototype)?;
    crate::webidl::define_constant(scope, prototype, "DOM_KEY_LOCATION_STANDARD", 0)?;
    crate::webidl::define_constant(scope, prototype, "DOM_KEY_LOCATION_LEFT", 1)?;
    crate::webidl::define_constant(scope, prototype, "DOM_KEY_LOCATION_RIGHT", 2)?;
    crate::webidl::define_constant(scope, prototype, "DOM_KEY_LOCATION_NUMPAD", 3)?;
    super::keyboard_event_get_modifier_state::define(scope, prototype)?;
    super::keyboard_event_init_keyboard_event::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_constant(scope, constructor.into(), "DOM_KEY_LOCATION_STANDARD", 0)?;
    crate::webidl::define_constant(scope, constructor.into(), "DOM_KEY_LOCATION_LEFT", 1)?;
    crate::webidl::define_constant(scope, constructor.into(), "DOM_KEY_LOCATION_RIGHT", 2)?;
    crate::webidl::define_constant(scope, constructor.into(), "DOM_KEY_LOCATION_NUMPAD", 3)?;
    let parent = super::ui_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<KeyboardEventStore>()
        .ok_or_else(|| "KeyboardEvent state was not prepared".to_owned())?
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
        .ok_or_else(|| "cannot create KeyboardEvent".to_owned())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'KeyboardEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let key = init
        .and_then(|init| string_property(scope, init, "key"))
        .unwrap_or_default();
    let code = init
        .and_then(|init| string_property(scope, init, "code"))
        .unwrap_or_default();
    let location = init
        .map(|init| number_property(scope, init, "location", 0.0) as u32)
        .unwrap_or(0);
    let ctrl_key = init.is_some_and(|init| super::event::boolean_property(scope, init, "ctrlKey"));
    let shift_key =
        init.is_some_and(|init| super::event::boolean_property(scope, init, "shiftKey"));
    let alt_key = init.is_some_and(|init| super::event::boolean_property(scope, init, "altKey"));
    let meta_key = init.is_some_and(|init| super::event::boolean_property(scope, init, "metaKey"));
    let repeat = init.is_some_and(|init| super::event::boolean_property(scope, init, "repeat"));
    let is_composing =
        init.is_some_and(|init| super::event::boolean_property(scope, init, "isComposing"));
    let bubbles = init.is_some_and(|init| super::event::boolean_property(scope, init, "bubbles"));
    let cancelable =
        init.is_some_and(|init| super::event::boolean_property(scope, init, "cancelable"));
    let composed = init.is_some_and(|init| super::event::boolean_property(scope, init, "composed"));
    let view = init
        .and_then(|init| value_property(scope, init, "view"))
        .map(|value| v8::Global::new(scope, value));
    super::ui_event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        bubbles,
        cancelable,
        composed,
        view,
        0,
        None,
    );
    scope
        .get_slot_mut::<KeyboardEventStore>()
        .expect("KeyboardEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            KeyboardRecord {
                key,
                code,
                location,
                ctrl_key,
                shift_key,
                alt_key,
                meta_key,
                repeat,
                is_composing,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<KeyboardRecord> {
    scope
        .get_slot::<KeyboardEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&KeyboardRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.key);
}
pub(crate) fn get_code(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.code);
}

pub(crate) fn get_location(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.location).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&KeyboardRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_ctrl_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.ctrl_key);
}
pub(crate) fn get_shift_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.shift_key);
}
pub(crate) fn get_alt_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.alt_key);
}
pub(crate) fn get_meta_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.meta_key);
}
pub(crate) fn get_repeat(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.repeat);
}
pub(crate) fn get_is_composing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.is_composing);
}

pub(crate) fn get_zero(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_modifier_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let modifier = crate::webidl::value_to_string(scope, arguments.get(0));
    let active = match modifier.as_str() {
        "Control" => record.ctrl_key,
        "Shift" => record.shift_key,
        "Alt" => record.alt_key,
        "Meta" => record.meta_key,
        _ => false,
    };
    result.set(v8::Boolean::new(scope, active).into());
}

pub(crate) fn init_keyboard_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(mut record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.location = arguments.get(5).uint32_value(scope).unwrap_or(0);
    let modifiers = crate::webidl::value_to_string(scope, arguments.get(6));
    record.ctrl_key = modifiers.split_whitespace().any(|value| value == "Control");
    record.shift_key = modifiers.split_whitespace().any(|value| value == "Shift");
    record.alt_key = modifiers.split_whitespace().any(|value| value == "Alt");
    record.meta_key = modifiers.split_whitespace().any(|value| value == "Meta");
    record.repeat = false;
    super::ui_event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        arguments.get(1).boolean_value(scope),
        arguments.get(2).boolean_value(scope),
        false,
        Some(v8::Global::new(scope, arguments.get(3))),
        0,
        None,
    );
    if let Some(stored) = scope
        .get_slot_mut::<KeyboardEventStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *stored = record;
    }
}

pub(crate) fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then_some(value)
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    value_property(scope, object, name).map(|value| crate::webidl::value_to_string(scope, value))
}

pub(crate) fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    default: f64,
) -> f64 {
    value_property(scope, object, name)
        .and_then(|value| value.number_value(scope))
        .unwrap_or(default)
}
