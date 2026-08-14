use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XrHitTestResultStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrHitTestResultStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "XRHitTestResult", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrHitTestResultStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRHitTestResult",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "getPose", 1, pose)?;
    crate::webidl::define_method(s, p, "createAnchor", 0, anchor)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrHitTestResultStore>()
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
fn pose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    if let Ok(v) = super::xr_pose::create(s) {
        r.set(v.into())
    }
}
fn anchor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !is_instance(s, a.this()) {
        let message = "Failed to execute 'createAnchor' on 'XRHitTestResult': Illegal invocation";
        if let Some(promise) = crate::webidl::rejected_type_error_promise(s, message) {
            r.set(promise.into());
        }
        return;
    }
    if let Ok(v) = super::xr_anchor::create(s)
        && let Ok(p) = super::writable_stream::resolved_promise(s, v.into())
    {
        r.set(p.into())
    }
}
fn is_instance(s: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<XrHitTestResultStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}
fn require(s: &mut v8::PinScope<'_, '_>, a: &v8::FunctionCallbackArguments<'_>) -> bool {
    let valid = is_instance(s, a.this());
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
    valid
}
