use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct SvgCircleElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) cx: v8::Global<v8::Object>,
    pub(crate) cy: v8::Global<v8::Object>,
    pub(crate) radius: v8::Global<v8::Object>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SvgCircleElementStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "SVGCircleElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = s
        .get_slot::<SvgCircleElementStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(s, &old));
    }
    let c = crate::webidl::create_function(
        s,
        "SVGCircleElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::svg_circle_element_cx_property::define(s, p)?;
    super::svg_circle_element_cy_property::define(s, p)?;
    super::svg_circle_element_r_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::svg_geometry_element::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SvgCircleElementStore>()
        .ok_or_else(|| "SVGCircleElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let o = super::svg_geometry_element::create_with_constructor(s, c, "circle", owner, 0.0)?;
    let cx = super::svg_animated_length::create(s, 0.0)?;
    let cy = super::svg_animated_length::create(s, 0.0)?;
    let radius = super::svg_animated_length::create(s, 0.0)?;
    let r = Record {
        cx: v8::Global::new(s, cx),
        cy: v8::Global::new(s, cy),
        radius: v8::Global::new(s, radius),
    };
    s.get_slot_mut::<SvgCircleElementStore>()
        .ok_or_else(|| "SVGCircleElement state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), r);
    Ok(o)
}
pub(crate) fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'SVGCircleElement': Illegal constructor",
    )
}
pub(crate) fn rec(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgCircleElementStore>()?
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
pub(crate) fn get_cx(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.cx, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_cy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.cy, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_radius(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.radius, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
