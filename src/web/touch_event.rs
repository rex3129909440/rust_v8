use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TouchEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TouchEventRecord>,
}

#[derive(Clone)]
pub(crate) struct TouchEventRecord {
    pub(crate) touches: v8::Global<v8::Object>,
    pub(crate) target_touches: v8::Global<v8::Object>,
    pub(crate) changed_touches: v8::Global<v8::Object>,
    pub(crate) alt_key: bool,
    pub(crate) meta_key: bool,
    pub(crate) ctrl_key: bool,
    pub(crate) shift_key: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TouchEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TouchEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TouchEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TouchEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::touch_event_touches_property::define(scope, prototype)?;
    super::touch_event_target_touches_property::define(scope, prototype)?;
    super::touch_event_changed_touches_property::define(scope, prototype)?;
    super::touch_event_alt_key_property::define(scope, prototype)?;
    super::touch_event_meta_key_property::define(scope, prototype)?;
    super::touch_event_ctrl_key_property::define(scope, prototype)?;
    super::touch_event_shift_key_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let ui_event = super::ui_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, ui_event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TouchEventStore>()
        .ok_or_else(|| "TouchEvent state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "TouchEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let touches = match touch_list_property(scope, init, "touches") {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let target_touches = match touch_list_property(scope, init, "targetTouches") {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let changed_touches = match touch_list_property(scope, init, "changedTouches") {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let bubbles = init.is_some_and(|value| super::event::boolean_property(scope, value, "bubbles"));
    let cancelable =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "cancelable"));
    let composed =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "composed"));
    let detail = init
        .map(|value| super::event::number_property(scope, value, "detail", 0.0) as i32)
        .unwrap_or(0);
    let view = init.and_then(|value| value_property(scope, value, "view"));
    let source_capabilities =
        init.and_then(|value| value_property(scope, value, "sourceCapabilities"));
    let record = TouchEventRecord {
        touches: v8::Global::new(scope, touches),
        target_touches: v8::Global::new(scope, target_touches),
        changed_touches: v8::Global::new(scope, changed_touches),
        alt_key: init.is_some_and(|value| super::event::boolean_property(scope, value, "altKey")),
        meta_key: init.is_some_and(|value| super::event::boolean_property(scope, value, "metaKey")),
        ctrl_key: init.is_some_and(|value| super::event::boolean_property(scope, value, "ctrlKey")),
        shift_key: init
            .is_some_and(|value| super::event::boolean_property(scope, value, "shiftKey")),
    };
    let object = arguments.this();
    super::ui_event::attach(
        scope,
        object,
        event_type,
        bubbles,
        cancelable,
        composed,
        view,
        detail,
        source_capabilities,
    );
    scope
        .get_slot_mut::<TouchEventStore>()
        .expect("TouchEvent state")
        .records
        .insert(object.get_identity_hash().get(), record);
    result.set(object.into());
}

pub(crate) fn touch_list_property<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    init: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let value = init
        .and_then(|object| {
            let key = v8::String::new(scope, name)?;
            object.get(scope, key.into())
        })
        .unwrap_or_else(|| v8::undefined(scope).into());
    super::touch_list::from_value(scope, value)
}

pub(crate) fn value_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TouchEventRecord> {
    scope
        .get_slot::<TouchEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TouchEventRecord) -> &v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_touches(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_list(s, a, r, |v| &v.touches);
}
pub(crate) fn get_target_touches(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_list(s, a, r, |v| &v.target_touches);
}
pub(crate) fn get_changed_touches(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_list(s, a, r, |v| &v.changed_touches);
}

pub(crate) fn return_bool(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TouchEventRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_alt_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.alt_key);
}
pub(crate) fn get_meta_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.meta_key);
}
pub(crate) fn get_ctrl_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.ctrl_key);
}
pub(crate) fn get_shift_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.shift_key);
}
