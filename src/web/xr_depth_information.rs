use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XrDepthInformationStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrDepthInformationStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "XRDepthInformation", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrDepthInformationStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRDepthInformation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "width", width)?;
    crate::webidl::define_readonly_accessor(s, p, "height", height)?;
    crate::webidl::define_readonly_accessor(s, p, "normDepthBufferFromNormView", transform)?;
    crate::webidl::define_readonly_accessor(s, p, "rawValueToMeters", raw)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_readonly_accessor(s, p, "projectionMatrix", matrix)?;
    crate::webidl::define_readonly_accessor(s, p, "transform", transform)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrDepthInformationStore>()
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
pub(crate) fn attach(s: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    s.get_slot_mut::<XrDepthInformationStore>()
        .expect("XRDepthInformation state")
        .instances
        .insert(object.get_identity_hash().get());
}
fn require(s: &mut v8::PinScope<'_, '_>, a: &v8::FunctionCallbackArguments<'_>) -> bool {
    let valid = s
        .get_slot::<XrDepthInformationStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains(&a.this().get_identity_hash().get())
        });
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
    valid
}
fn width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Integer::new(s, 1280).into())
}
fn height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Integer::new(s, 720).into())
}
fn raw(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Number::new(s, 0.001).into())
}
fn transform(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    if let Ok(v) = super::xr_rigid_transform::create(s) {
        r.set(v.into())
    }
}
fn matrix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Array::new(s, 16).into())
}
