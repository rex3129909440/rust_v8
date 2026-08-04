use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgFeDistantLightElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) azimuth: v8::Global<v8::Object>,
    pub(crate) elevation: v8::Global<v8::Object>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgFeDistantLightElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGFEDistantLightElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = scope
        .get_slot::<SvgFeDistantLightElementStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(scope, &old));
    }
    let c = crate::webidl::create_function(
        scope,
        "SVGFEDistantLightElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::svg_fe_distant_light_element_azimuth_property::define(scope, p)?;
    super::svg_fe_distant_light_element_elevation_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SvgFeDistantLightElementStore>()
        .ok_or_else(|| "SVGFEDistantLightElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(scope, c, "feDistantLight", owner)?;
    let azimuth = super::svg_animated_number::create(scope, 0.0)?;
    let elevation = super::svg_animated_number::create(scope, 0.0)?;
    let record = Record {
        azimuth: v8::Global::new(scope, azimuth),
        elevation: v8::Global::new(scope, elevation),
    };
    scope
        .get_slot_mut::<SvgFeDistantLightElementStore>()
        .ok_or_else(|| "SVGFEDistantLightElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}
pub(crate) fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'SVGFEDistantLightElement': Illegal constructor",
    )
}
pub(crate) fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeDistantLightElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn ret(
    s: &v8::PinScope<'_, '_>,
    v: &v8::Global<v8::Object>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::Local::new(s, v).into())
}
pub(crate) fn get_azimuth(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.azimuth, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_elevation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.elevation, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
