use std::collections::HashMap;
pub(crate) const UNKNOWN: i32 = 0;
pub(crate) const NORMAL: i32 = 1;
pub(crate) const MULTIPLY: i32 = 2;
pub(crate) const SCREEN: i32 = 3;
pub(crate) const DARKEN: i32 = 4;
pub(crate) const LIGHTEN: i32 = 5;
pub(crate) const OVERLAY: i32 = 6;
pub(crate) const COLOR_DODGE: i32 = 7;
pub(crate) const COLOR_BURN: i32 = 8;
pub(crate) const HARD_LIGHT: i32 = 9;
pub(crate) const SOFT_LIGHT: i32 = 10;
pub(crate) const DIFFERENCE: i32 = 11;
pub(crate) const EXCLUSION: i32 = 12;
pub(crate) const HUE: i32 = 13;
pub(crate) const SATURATION: i32 = 14;
pub(crate) const COLOR: i32 = 15;
pub(crate) const LUMINOSITY: i32 = 16;
#[derive(Default)]
pub(crate) struct SvgFeBlendElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) input1: v8::Global<v8::Object>,
    pub(crate) input2: v8::Global<v8::Object>,
    pub(crate) mode: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) result: v8::Global<v8::Object>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SvgFeBlendElementStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "SVGFEBlendElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = s
        .get_slot::<SvgFeBlendElementStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(s, &old));
    }
    let c = crate::webidl::create_function(
        s,
        "SVGFEBlendElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::svg_fe_blend_element_in1_property::define(s, p)?;
    super::svg_fe_blend_element_in2_property::define(s, p)?;
    super::svg_fe_blend_element_mode_property::define(s, p)?;
    super::svg_fe_blend_element_x_property::define(s, p)?;
    super::svg_fe_blend_element_y_property::define(s, p)?;
    super::svg_fe_blend_element_width_property::define(s, p)?;
    super::svg_fe_blend_element_height_property::define(s, p)?;
    super::svg_fe_blend_element_result_property::define(s, p)?;
    constants(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    constants(s, c.into())?;
    let parent = super::svg_element::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SvgFeBlendElementStore>()
        .ok_or_else(|| "SVGFEBlendElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn constants(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_UNKNOWN", UNKNOWN)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_NORMAL", NORMAL)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_MULTIPLY", MULTIPLY)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_SCREEN", SCREEN)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_DARKEN", DARKEN)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_LIGHTEN", LIGHTEN)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_OVERLAY", OVERLAY)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_COLOR_DODGE", COLOR_DODGE)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_COLOR_BURN", COLOR_BURN)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_HARD_LIGHT", HARD_LIGHT)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_SOFT_LIGHT", SOFT_LIGHT)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_DIFFERENCE", DIFFERENCE)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_EXCLUSION", EXCLUSION)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_HUE", HUE)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_SATURATION", SATURATION)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_COLOR", COLOR)?;
    crate::webidl::define_constant(s, o, "SVG_FEBLEND_MODE_LUMINOSITY", LUMINOSITY)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let o = super::svg_element::create_with_constructor(s, c, "feBlend", owner)?;
    let input1 = super::svg_animated_string::create(s, "")?;
    let input2 = super::svg_animated_string::create(s, "")?;
    let mode = super::svg_animated_enumeration::create(s, NORMAL as u32)?;
    let x = super::svg_animated_length::create_with_unit(s, 2, 0.0)?;
    let y = super::svg_animated_length::create_with_unit(s, 2, 0.0)?;
    let width = super::svg_animated_length::create_with_unit(s, 2, 100.0)?;
    let height = super::svg_animated_length::create_with_unit(s, 2, 100.0)?;
    let result = super::svg_animated_string::create(s, "")?;
    let r = Record {
        input1: v8::Global::new(s, input1),
        input2: v8::Global::new(s, input2),
        mode: v8::Global::new(s, mode),
        x: v8::Global::new(s, x),
        y: v8::Global::new(s, y),
        width: v8::Global::new(s, width),
        height: v8::Global::new(s, height),
        result: v8::Global::new(s, result),
    };
    s.get_slot_mut::<SvgFeBlendElementStore>()
        .ok_or_else(|| "SVGFEBlendElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGFEBlendElement': Illegal constructor",
    )
}
pub(crate) fn rec(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeBlendElementStore>()?
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
pub(crate) fn get_input1(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.input1, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_input2(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.input2, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.mode, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.width, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.height, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_result(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.result, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
