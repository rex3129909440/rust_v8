use std::collections::HashMap;
#[derive(Clone)]
struct PresentationRecord {
    default_request: Option<v8::Global<v8::Object>>,
    receiver: v8::Global<v8::Object>,
}
#[derive(Default)]
pub(crate) struct PresentationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PresentationRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PresentationStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "Presentation", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PresentationStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "Presentation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "defaultRequest", get_default, set_default)?;
    crate::webidl::define_readonly_accessor(s, p, "receiver", receiver)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PresentationStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create Presentation".to_owned());
    }
    let receiver = super::presentation_receiver::create(s)?;
    let receiver = v8::Global::new(s, receiver);
    s.get_slot_mut::<PresentationStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            PresentationRecord {
                default_request: None,
                receiver,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<PresentationRecord> {
    s.get_slot::<PresentationStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_default(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(x) = v.default_request {
            r.set(v8::Local::new(s, &x).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_default(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .map(|x| v8::Global::new(s, x));
    if let Some(v) = s
        .get_slot_mut::<PresentationStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.default_request = value
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn receiver(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.receiver).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
