use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextUpdateEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TextUpdateEventRecord>,
}
#[derive(Clone)]
pub(crate) struct TextUpdateEventRecord {
    pub(crate) update_range_start: u32,
    pub(crate) update_range_end: u32,
    pub(crate) text: String,
    pub(crate) selection_start: u32,
    pub(crate) selection_end: u32,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextUpdateEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextUpdateEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextUpdateEventStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TextUpdateEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::text_update_event_update_range_start_property::define(scope, p)?;
    super::text_update_event_update_range_end_property::define(scope, p)?;
    super::text_update_event_text_property::define(scope, p)?;
    super::text_update_event_selection_start_property::define(scope, p)?;
    super::text_update_event_selection_end_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TextUpdateEventStore>()
        .ok_or_else(|| "TextUpdateEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "TextUpdateEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let record = TextUpdateEventRecord {
        update_range_start: init
            .map(|v| unsigned_property(scope, v, "updateRangeStart"))
            .unwrap_or(0),
        update_range_end: init
            .map(|v| unsigned_property(scope, v, "updateRangeEnd"))
            .unwrap_or(0),
        text: init
            .and_then(|v| string_property(scope, v, "text"))
            .unwrap_or_default(),
        selection_start: init
            .map(|v| unsigned_property(scope, v, "selectionStart"))
            .unwrap_or(0),
        selection_end: init
            .map(|v| unsigned_property(scope, v, "selectionEnd"))
            .unwrap_or(0),
    };
    let bubbles = init.is_some_and(|v| super::event::boolean_property(scope, v, "bubbles"));
    let cancelable = init.is_some_and(|v| super::event::boolean_property(scope, v, "cancelable"));
    let composed = init.is_some_and(|v| super::event::boolean_property(scope, v, "composed"));
    let object = arguments.this();
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    scope
        .get_slot_mut::<TextUpdateEventStore>()
        .expect("TextUpdateEvent state")
        .records
        .insert(object.get_identity_hash().get(), record);
    result.set(object.into());
}
pub(crate) fn unsigned_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> u32 {
    v8::String::new(scope, name)
        .and_then(|k| object.get(scope, k.into()))
        .filter(|v| !v.is_undefined())
        .and_then(|v| v.uint32_value(scope))
        .unwrap_or(0)
}
pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let k = v8::String::new(scope, name)?;
    let v = object.get(scope, k.into())?;
    if v.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, v))
    }
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TextUpdateEventRecord> {
    scope
        .get_slot::<TextUpdateEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn return_unsigned(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TextUpdateEventRecord) -> u32,
) {
    if let Some(v) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, select(&v)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_update_range_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_unsigned(s, a, r, |v| v.update_range_start)
}
pub(crate) fn get_update_range_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_unsigned(s, a, r, |v| v.update_range_end)
}
pub(crate) fn get_selection_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_unsigned(s, a, r, |v| v.selection_start)
}
pub(crate) fn get_selection_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_unsigned(s, a, r, |v| v.selection_end)
}
pub(crate) fn get_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(text) = v8::String::new(scope, &v.text) {
        result.set(text.into())
    }
}
