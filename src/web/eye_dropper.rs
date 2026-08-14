use std::collections::HashSet;
#[derive(Default)]
pub(crate) struct EyeDropperStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(EyeDropperStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "EyeDropper", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<EyeDropperStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "EyeDropper",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "open", 0, open)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<EyeDropperStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "constructor must be called with new");
        return;
    }
    s.get_slot_mut::<EyeDropperStore>()
        .unwrap()
        .instances
        .insert(a.this().get_identity_hash().get());
    r.set(a.this().into())
}
fn open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !s
        .get_slot::<EyeDropperStore>()
        .is_some_and(|x| x.instances.contains(&a.this().get_identity_hash().get()))
    {
        crate::webidl::reject_illegal_invocation_promise(s, "EyeDropper", "open", r);
        return;
    }
    let o = v8::Object::new(s);
    if let (Some(k), Some(v)) = (v8::String::new(s, "sRGBHex"), v8::String::new(s, "#000000")) {
        let _ = o.set(s, k.into(), v.into());
    }
    if let Ok(p) = super::writable_stream::resolved_promise(s, o.into()) {
        r.set(p.into())
    }
}
