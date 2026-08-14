use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XrInputSourceStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrInputSourceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "XRInputSource", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrInputSourceStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRInputSource",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "handedness", none)?;
    crate::webidl::define_readonly_accessor(s, p, "targetRayMode", ray_mode)?;
    crate::webidl::define_readonly_accessor(s, p, "targetRaySpace", space)?;
    crate::webidl::define_readonly_accessor(s, p, "gripSpace", space)?;
    crate::webidl::define_readonly_accessor(s, p, "gamepad", null)?;
    crate::webidl::define_readonly_accessor(s, p, "hand", null)?;
    crate::webidl::define_readonly_accessor(s, p, "profiles", profiles)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrInputSourceStore>()
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
fn text(s: &mut v8::PinScope<'_, '_>, mut r: v8::ReturnValue<'_>, v: &str) {
    if let Some(x) = v8::String::new(s, v) {
        r.set(x.into())
    }
}
fn require(s: &mut v8::PinScope<'_, '_>, a: &v8::FunctionCallbackArguments<'_>) -> bool {
    let valid = s.get_slot::<XrInputSourceStore>().is_some_and(|store| {
        store
            .instances
            .contains(&a.this().get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
    valid
}
fn none(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    text(s, r, "none")
}
fn ray_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    text(s, r, "tracked-pointer")
}
fn space(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    if let Ok(v) = super::xr_space::create(s) {
        r.set(v.into())
    }
}
fn null(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::null(s).into())
}
fn profiles(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Array::new(s, 0).into())
}
