use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextEventStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TextEvent",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::text_event_data_property::define(scope, p)?;
    super::text_event_init_text_event::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let ui = super::ui_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, ui)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TextEventStore>()
        .ok_or_else(|| "TextEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TextEvent': Illegal constructor",
    );
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: String,
    data: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create TextEvent".to_owned());
    }
    super::ui_event::attach(scope, o, event_type, false, false, false, None, 0, None);
    scope
        .get_slot_mut::<TextEventStore>()
        .ok_or_else(|| "TextEvent state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), data);
    Ok(o)
}
pub(crate) fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(data) = scope
        .get_slot::<TextEventStore>()
        .and_then(|s| s.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(scope, &data) {
        r.set(v.into())
    }
}
pub(crate) fn init_text_event(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<TextEventStore>()
        .is_some_and(|s| s.records.contains_key(&a.this().get_identity_hash().get()))
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let data = crate::webidl::value_to_string(scope, a.get(4));
    if let Some(v) = scope
        .get_slot_mut::<TextEventStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        *v = data;
    }
}
