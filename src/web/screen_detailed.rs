#[derive(Default)]
pub(crate) struct ScreenDetailedStore {
    constructor: crate::webidl::RealmConstructor,
    instances: std::collections::HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(ScreenDetailedStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "ScreenDetailed", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<ScreenDetailedStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "ScreenDetailed",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "left", left)?;
    crate::webidl::define_readonly_accessor(s, p, "top", top)?;
    crate::webidl::define_readonly_accessor(s, p, "isPrimary", yes)?;
    crate::webidl::define_readonly_accessor(s, p, "isInternal", yes)?;
    crate::webidl::define_readonly_accessor(s, p, "devicePixelRatio", ratio)?;
    crate::webidl::define_readonly_accessor(s, p, "label", label)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::screen::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let g = v8::Global::new(s, c);
    let realm_id = crate::webidl::realm_id(s);
    s.get_slot_mut::<ScreenDetailedStore>()
        .ok_or_else(|| "ScreenDetailed state missing".to_owned())?
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
        return Err("cannot create ScreenDetailed".to_owned());
    }
    s.get_slot_mut::<ScreenDetailedStore>()
        .unwrap()
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<ScreenDetailedStore>()
        .is_some_and(|x| x.instances.contains(&o.get_identity_hash().get()))
}
fn left(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        r.set(v8::Integer::new(s, 0).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn top(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    left(s, a, r)
}
fn yes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        r.set(v8::Boolean::new(s, true).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn ratio(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        r.set(v8::Number::new(s, 1.0).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        r.set(v8::String::new(s, "Primary Display").unwrap().into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
