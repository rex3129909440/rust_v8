use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XrLightEstimateStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrLightEstimateStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "XRLightEstimate", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrLightEstimateStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRLightEstimate",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "sphericalHarmonicsCoefficients", array)?;
    crate::webidl::define_readonly_accessor(s, p, "primaryLightDirection", object)?;
    crate::webidl::define_readonly_accessor(s, p, "primaryLightIntensity", object)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrLightEstimateStore>()
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
fn array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Array::new(s, 27).into())
}
fn object(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Object::new(s).into())
}

pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create XRLightEstimate".to_owned());
    }
    s.get_slot_mut::<XrLightEstimateStore>()
        .expect("XRLightEstimate state")
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn require(s: &mut v8::PinScope<'_, '_>, a: &v8::FunctionCallbackArguments<'_>) -> bool {
    let valid = s.get_slot::<XrLightEstimateStore>().is_some_and(|store| {
        store
            .instances
            .contains(&a.this().get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
    valid
}
