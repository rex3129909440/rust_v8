use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct SensorErrorEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) errors: HashMap<i32, v8::Global<v8::Object>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SensorErrorEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "SensorErrorEvent", c.into())
}
pub(crate) fn ensure<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<SensorErrorEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "SensorErrorEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::sensor_error_event_error_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let g = v8::Global::new(s, c);
    let realm_id = crate::webidl::realm_id(s);
    s.get_slot_mut::<SensorErrorEventStore>()
        .ok_or_else(|| "SensorErrorEvent state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
    Ok(c)
}
pub(crate) fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(s, "2 arguments required");
        return;
    }
    let t = crate::webidl::value_to_string(s, a.get(0));
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let err = init
        .and_then(|o| v8::String::new(s, "error").and_then(|k| o.get(s, k.into())))
        .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok());
    let Some(err) = err else {
        crate::webidl::throw_type_error(s, "error is required");
        return;
    };
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(s, a.this(), t, bubbles, cancelable, composed);
    let err = v8::Global::new(s, err);
    s.get_slot_mut::<SensorErrorEventStore>()
        .unwrap()
        .errors
        .insert(a.this().get_identity_hash().get(), err);
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    error: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(s)?;
    let prototype = crate::webidl::prototype(s, constructor)?;
    let event = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, event, prototype.into()) != Some(true) {
        return Err("cannot create SensorErrorEvent".to_owned());
    }
    super::event::attach(s, event, "error".to_owned(), false, false, false);
    let stored_error = v8::Global::new(s, error);
    s.get_slot_mut::<SensorErrorEventStore>()
        .ok_or_else(|| "SensorErrorEvent state missing".to_owned())?
        .errors
        .insert(event.get_identity_hash().get(), stored_error);
    Ok(event)
}
pub(crate) fn error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot::<SensorErrorEventStore>()
        .and_then(|x| x.errors.get(&a.this().get_identity_hash().get()))
        .cloned()
    {
        r.set(v8::Local::new(s, &v).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
