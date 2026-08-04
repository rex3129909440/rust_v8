use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgAnimatedBooleanStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, bool>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgAnimatedBooleanStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGAnimatedBoolean", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = scope
        .get_slot::<SvgAnimatedBooleanStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(scope, &old));
    }
    let c = crate::webidl::create_function(
        scope,
        "SVGAnimatedBoolean",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_accessor(scope, p, "baseVal", get_value, set_value)?;
    crate::webidl::define_readonly_accessor(scope, p, "animVal", get_value)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SvgAnimatedBooleanStore>()
        .ok_or_else(|| "SVGAnimatedBoolean state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create SVGAnimatedBoolean".to_owned());
    }
    scope
        .get_slot_mut::<SvgAnimatedBooleanStore>()
        .ok_or_else(|| "SVGAnimatedBoolean state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), initial);
    Ok(object)
}
fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'SVGAnimatedBoolean': Illegal constructor",
    )
}
fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let v = s
        .get_slot::<SvgAnimatedBooleanStore>()
        .and_then(|store| store.records.get(&a.this().get_identity_hash().get()))
        .copied();
    if let Some(v) = v {
        r.set(v8::Boolean::new(s, v).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(s);
    if let Some(value) = s
        .get_slot_mut::<SvgAnimatedBooleanStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        *value = v
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
