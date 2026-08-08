use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgFeSpotLightElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) z: v8::Global<v8::Object>,
    pub(crate) points_at_x: v8::Global<v8::Object>,
    pub(crate) points_at_y: v8::Global<v8::Object>,
    pub(crate) points_at_z: v8::Global<v8::Object>,
    pub(crate) specular_exponent: v8::Global<v8::Object>,
    pub(crate) limiting_cone_angle: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgFeSpotLightElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGFESpotLightElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgFeSpotLightElementStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGFESpotLightElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_fe_spot_light_element_x_property::define(scope, prototype)?;
    super::svg_fe_spot_light_element_y_property::define(scope, prototype)?;
    super::svg_fe_spot_light_element_z_property::define(scope, prototype)?;
    super::svg_fe_spot_light_element_points_at_x_property::define(scope, prototype)?;
    super::svg_fe_spot_light_element_points_at_y_property::define(scope, prototype)?;
    super::svg_fe_spot_light_element_points_at_z_property::define(scope, prototype)?;
    super::svg_fe_spot_light_element_specular_exponent_property::define(scope, prototype)?;
    super::svg_fe_spot_light_element_limiting_cone_angle_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgFeSpotLightElementStore>()
        .ok_or_else(|| "SVGFESpotLightElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object =
        super::svg_element::create_with_constructor(scope, constructor, "feSpotLight", owner)?;
    let x = super::svg_animated_number::create(scope, 0.0)?;
    let y = super::svg_animated_number::create(scope, 0.0)?;
    let z = super::svg_animated_number::create(scope, 0.0)?;
    let points_at_x = super::svg_animated_number::create(scope, 0.0)?;
    let points_at_y = super::svg_animated_number::create(scope, 0.0)?;
    let points_at_z = super::svg_animated_number::create(scope, 0.0)?;
    let specular_exponent = super::svg_animated_number::create(scope, 1.0)?;
    let limiting_cone_angle = super::svg_animated_number::create(scope, 0.0)?;
    let record = Record {
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        z: v8::Global::new(scope, z),
        points_at_x: v8::Global::new(scope, points_at_x),
        points_at_y: v8::Global::new(scope, points_at_y),
        points_at_z: v8::Global::new(scope, points_at_z),
        specular_exponent: v8::Global::new(scope, specular_exponent),
        limiting_cone_angle: v8::Global::new(scope, limiting_cone_angle),
    };
    scope
        .get_slot_mut::<SvgFeSpotLightElementStore>()
        .ok_or_else(|| "SVGFESpotLightElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGFESpotLightElement': Illegal constructor",
    );
}
pub(crate) fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeSpotLightElementStore>()?
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
pub(crate) fn get_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.z, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_points_at_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.points_at_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_points_at_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.points_at_y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_points_at_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.points_at_z, r)
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
pub(crate) fn get_limiting_cone_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.limiting_cone_angle, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
