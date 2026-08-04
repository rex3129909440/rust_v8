use std::collections::HashMap;

pub(crate) const CHANNEL_UNKNOWN: i32 = 0;
pub(crate) const CHANNEL_R: i32 = 1;
pub(crate) const CHANNEL_G: i32 = 2;
pub(crate) const CHANNEL_B: i32 = 3;
pub(crate) const CHANNEL_A: i32 = 4;
#[derive(Default)]
pub(crate) struct SvgFeDisplacementMapElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) input1: v8::Global<v8::Object>,
    pub(crate) input2: v8::Global<v8::Object>,
    pub(crate) scale: v8::Global<v8::Object>,
    pub(crate) x_channel: v8::Global<v8::Object>,
    pub(crate) y_channel: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) result: v8::Global<v8::Object>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgFeDisplacementMapElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGFEDisplacementMapElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = scope
        .get_slot::<SvgFeDisplacementMapElementStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(scope, &old));
    }
    let c = crate::webidl::create_function(
        scope,
        "SVGFEDisplacementMapElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::svg_fe_displacement_map_element_in1_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_in2_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_scale_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_x_channel_selector_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_y_channel_selector_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_x_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_y_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_width_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_height_property::define(scope, p)?;
    super::svg_fe_displacement_map_element_result_property::define(scope, p)?;
    define_constants(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    define_constants(scope, c.into())?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SvgFeDisplacementMapElementStore>()
        .ok_or_else(|| "SVGFEDisplacementMapElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn define_constants(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(s, o, "SVG_CHANNEL_UNKNOWN", CHANNEL_UNKNOWN)?;
    crate::webidl::define_constant(s, o, "SVG_CHANNEL_R", CHANNEL_R)?;
    crate::webidl::define_constant(s, o, "SVG_CHANNEL_G", CHANNEL_G)?;
    crate::webidl::define_constant(s, o, "SVG_CHANNEL_B", CHANNEL_B)?;
    crate::webidl::define_constant(s, o, "SVG_CHANNEL_A", CHANNEL_A)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(scope, c, "feDisplacementMap", owner)?;
    let input1 = super::svg_animated_string::create(scope, "")?;
    let input2 = super::svg_animated_string::create(scope, "")?;
    let scale = super::svg_animated_number::create(scope, 0.0)?;
    let x_channel = super::svg_animated_enumeration::create(scope, CHANNEL_A as u32)?;
    let y_channel = super::svg_animated_enumeration::create(scope, CHANNEL_A as u32)?;
    let x = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let y = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let width = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let height = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let result = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        input1: v8::Global::new(scope, input1),
        input2: v8::Global::new(scope, input2),
        scale: v8::Global::new(scope, scale),
        x_channel: v8::Global::new(scope, x_channel),
        y_channel: v8::Global::new(scope, y_channel),
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        width: v8::Global::new(scope, width),
        height: v8::Global::new(scope, height),
        result: v8::Global::new(scope, result),
    };
    scope
        .get_slot_mut::<SvgFeDisplacementMapElementStore>()
        .ok_or_else(|| "SVGFEDisplacementMapElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGFEDisplacementMapElement': Illegal constructor",
    )
}
pub(crate) fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeDisplacementMapElementStore>()?
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
    if let Some(v) = record(s, a.this()) {
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
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.input2, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_scale(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.scale, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_x_channel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.x_channel, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_y_channel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.y_channel, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
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
    if let Some(v) = record(s, a.this()) {
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
    if let Some(v) = record(s, a.this()) {
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
    if let Some(v) = record(s, a.this()) {
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
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.result, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
