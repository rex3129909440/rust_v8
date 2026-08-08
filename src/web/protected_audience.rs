#[derive(Default)]
pub(crate) struct ProtectedAudienceStore {
    constructor: crate::webidl::RealmConstructor,
    instances: std::collections::HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(ProtectedAudienceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "ProtectedAudience", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<ProtectedAudienceStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "ProtectedAudience",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "queryFeatureSupport", 1, query)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let g = v8::Global::new(s, c);
    let realm_id = crate::webidl::realm_id(s);
    s.get_slot_mut::<ProtectedAudienceStore>()
        .ok_or_else(|| "ProtectedAudience state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
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
        return Err("cannot create ProtectedAudience".to_owned());
    }
    s.get_slot_mut::<ProtectedAudienceStore>()
        .unwrap()
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn query(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !s
        .get_slot::<ProtectedAudienceStore>()
        .is_some_and(|x| x.instances.contains(&a.this().get_identity_hash().get()))
    {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let o = v8::Object::new(s);
    let k = v8::String::new(s, "supported").unwrap();
    let _ = o.set(s, k.into(), v8::Boolean::new(s, true).into());
    if let Ok(p) = super::writable_stream::resolved_promise(s, o.into()) {
        r.set(p.into())
    }
}
