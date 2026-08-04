use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextFormatUpdateEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextFormatUpdateEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextFormatUpdateEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextFormatUpdateEventStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TextFormatUpdateEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::text_format_update_event_get_text_formats::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TextFormatUpdateEventStore>()
        .ok_or_else(|| "TextFormatUpdateEvent state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "TextFormatUpdateEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let mut formats = Vec::new();
    if let Some(sequence) = init.and_then(|o| property_object(scope, o, "textFormats")) {
        let length = v8::String::new(scope, "length")
            .and_then(|k| sequence.get(scope, k.into()))
            .and_then(|v| v.uint32_value(scope))
            .unwrap_or(0);
        for i in 0..length {
            let Some(value) = sequence.get_index(scope, i) else {
                continue;
            };
            let Ok(format) = v8::Local::<v8::Object>::try_from(value) else {
                crate::webidl::throw_type_error(scope, "textFormats contains a non-object");
                return;
            };
            if !super::text_format::is_instance(scope, format) {
                crate::webidl::throw_type_error(scope, "textFormats contains a non-TextFormat");
                return;
            }
            formats.push(v8::Global::new(scope, format));
        }
    }
    let bubbles = init.is_some_and(|v| super::event::boolean_property(scope, v, "bubbles"));
    let cancelable = init.is_some_and(|v| super::event::boolean_property(scope, v, "cancelable"));
    let composed = init.is_some_and(|v| super::event::boolean_property(scope, v, "composed"));
    let object = arguments.this();
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    scope
        .get_slot_mut::<TextFormatUpdateEventStore>()
        .expect("TextFormatUpdateEvent state")
        .records
        .insert(object.get_identity_hash().get(), formats);
    result.set(object.into());
}
pub(crate) fn property_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let k = v8::String::new(scope, n)?;
    let v = o.get(scope, k.into())?;
    v8::Local::<v8::Object>::try_from(v).ok()
}
pub(crate) fn get_text_formats(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = scope
        .get_slot::<TextFormatUpdateEventStore>()
        .and_then(|s| s.records.get(&arguments.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, values.len() as i32);
    for (i, v) in values.iter().enumerate() {
        let value = v8::Local::new(scope, v);
        let _ = array.set_index(scope, i as u32, value.into());
    }
    result.set(array.into());
}
