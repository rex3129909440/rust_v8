use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgFeDiffuseLightingElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) input: v8::Global<v8::Object>,
    pub(crate) surface_scale: v8::Global<v8::Object>,
    pub(crate) diffuse_constant: v8::Global<v8::Object>,
    pub(crate) kernel_x: v8::Global<v8::Object>,
    pub(crate) kernel_y: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) result: v8::Global<v8::Object>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgFeDiffuseLightingElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGFEDiffuseLightingElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = scope
        .get_slot::<SvgFeDiffuseLightingElementStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(scope, &old));
    }
    let c = crate::webidl::create_function(
        scope,
        "SVGFEDiffuseLightingElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::svg_fe_diffuse_lighting_element_in1_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_surface_scale_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_diffuse_constant_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_kernel_unit_length_x_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_kernel_unit_length_y_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_x_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_y_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_width_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_height_property::define(scope, p)?;
    super::svg_fe_diffuse_lighting_element_result_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SvgFeDiffuseLightingElementStore>()
        .ok_or_else(|| "SVGFEDiffuseLightingElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(scope, c, "feDiffuseLighting", owner)?;
    let input = super::svg_animated_string::create(scope, "")?;
    let surface_scale = super::svg_animated_number::create(scope, 1.0)?;
    let diffuse_constant = super::svg_animated_number::create(scope, 1.0)?;
    let kernel_x = super::svg_animated_number::create(scope, 0.0)?;
    let kernel_y = super::svg_animated_number::create(scope, 0.0)?;
    let x = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let y = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let width = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let height = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let result = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        input: v8::Global::new(scope, input),
        surface_scale: v8::Global::new(scope, surface_scale),
        diffuse_constant: v8::Global::new(scope, diffuse_constant),
        kernel_x: v8::Global::new(scope, kernel_x),
        kernel_y: v8::Global::new(scope, kernel_y),
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        width: v8::Global::new(scope, width),
        height: v8::Global::new(scope, height),
        result: v8::Global::new(scope, result),
    };
    scope
        .get_slot_mut::<SvgFeDiffuseLightingElementStore>()
        .ok_or_else(|| "SVGFEDiffuseLightingElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGFEDiffuseLightingElement': Illegal constructor",
    )
}
pub(crate) fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeDiffuseLightingElementStore>()?
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
pub(crate) fn get_input(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.input, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_surface_scale(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.surface_scale, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_diffuse_constant(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.diffuse_constant, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_kernel_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.kernel_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_kernel_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.kernel_y, r)
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
