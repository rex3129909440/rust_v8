use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgFeSpecularLightingElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) input: v8::Global<v8::Object>,
    pub(crate) surface_scale: v8::Global<v8::Object>,
    pub(crate) specular_constant: v8::Global<v8::Object>,
    pub(crate) specular_exponent: v8::Global<v8::Object>,
    pub(crate) kernel_unit_length_x: v8::Global<v8::Object>,
    pub(crate) kernel_unit_length_y: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) result: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgFeSpecularLightingElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGFESpecularLightingElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgFeSpecularLightingElementStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGFESpecularLightingElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_fe_specular_lighting_element_in1_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_surface_scale_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_specular_constant_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_specular_exponent_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_kernel_unit_length_x_property::define(
        scope, prototype,
    )?;
    super::svg_fe_specular_lighting_element_kernel_unit_length_y_property::define(
        scope, prototype,
    )?;
    super::svg_fe_specular_lighting_element_x_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_y_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_width_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_height_property::define(scope, prototype)?;
    super::svg_fe_specular_lighting_element_result_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgFeSpecularLightingElementStore>()
        .ok_or_else(|| "SVGFESpecularLightingElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(
        scope,
        constructor,
        "feSpecularLighting",
        owner,
    )?;
    let input = super::svg_animated_string::create(scope, "")?;
    let surface_scale = super::svg_animated_number::create(scope, 1.0)?;
    let specular_constant = super::svg_animated_number::create(scope, 1.0)?;
    let specular_exponent = super::svg_animated_number::create(scope, 1.0)?;
    let kernel_unit_length_x = super::svg_animated_number::create(scope, 0.0)?;
    let kernel_unit_length_y = super::svg_animated_number::create(scope, 0.0)?;
    let x = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let y = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let width = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let height = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let result = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        input: v8::Global::new(scope, input),
        surface_scale: v8::Global::new(scope, surface_scale),
        specular_constant: v8::Global::new(scope, specular_constant),
        specular_exponent: v8::Global::new(scope, specular_exponent),
        kernel_unit_length_x: v8::Global::new(scope, kernel_unit_length_x),
        kernel_unit_length_y: v8::Global::new(scope, kernel_unit_length_y),
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        width: v8::Global::new(scope, width),
        height: v8::Global::new(scope, height),
        result: v8::Global::new(scope, result),
    };
    scope
        .get_slot_mut::<SvgFeSpecularLightingElementStore>()
        .ok_or_else(|| "SVGFESpecularLightingElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGFESpecularLightingElement': Illegal constructor",
    )
}
pub(crate) fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeSpecularLightingElementStore>()?
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
pub(crate) fn get_specular_constant(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.specular_constant, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_specular_exponent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.specular_exponent, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_kernel_unit_length_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.kernel_unit_length_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_kernel_unit_length_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.kernel_unit_length_y, r)
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
