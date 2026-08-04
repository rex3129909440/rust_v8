use std::collections::HashMap;

pub(crate) const TURBULENCE_UNKNOWN: i32 = 0;
pub(crate) const FRACTAL_NOISE: i32 = 1;
pub(crate) const TURBULENCE: i32 = 2;
pub(crate) const STITCH_UNKNOWN: i32 = 0;
pub(crate) const STITCH: i32 = 1;
pub(crate) const NO_STITCH: i32 = 2;

#[derive(Default)]
pub(crate) struct SvgFeTurbulenceElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) base_frequency_x: v8::Global<v8::Object>,
    pub(crate) base_frequency_y: v8::Global<v8::Object>,
    pub(crate) num_octaves: v8::Global<v8::Object>,
    pub(crate) seed: v8::Global<v8::Object>,
    pub(crate) stitch_tiles: v8::Global<v8::Object>,
    pub(crate) turbulence_type: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) result: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgFeTurbulenceElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGFETurbulenceElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgFeTurbulenceElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGFETurbulenceElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_fe_turbulence_element_base_frequency_x_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_base_frequency_y_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_num_octaves_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_seed_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_stitch_tiles_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_type_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_x_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_y_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_width_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_height_property::define(scope, prototype)?;
    super::svg_fe_turbulence_element_result_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgFeTurbulenceElementStore>()
        .ok_or_else(|| "SVGFETurbulenceElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_TURBULENCE_TYPE_UNKNOWN",
        TURBULENCE_UNKNOWN,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_TURBULENCE_TYPE_FRACTALNOISE",
        FRACTAL_NOISE,
    )?;
    crate::webidl::define_constant(scope, object, "SVG_TURBULENCE_TYPE_TURBULENCE", TURBULENCE)?;
    crate::webidl::define_constant(scope, object, "SVG_STITCHTYPE_UNKNOWN", STITCH_UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "SVG_STITCHTYPE_STITCH", STITCH)?;
    crate::webidl::define_constant(scope, object, "SVG_STITCHTYPE_NOSTITCH", NO_STITCH)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object =
        super::svg_element::create_with_constructor(scope, constructor, "feTurbulence", owner)?;
    let base_frequency_x = super::svg_animated_number::create(scope, 0.0)?;
    let base_frequency_y = super::svg_animated_number::create(scope, 0.0)?;
    let num_octaves = super::svg_animated_integer::create(scope, 1)?;
    let seed = super::svg_animated_number::create(scope, 0.0)?;
    let stitch_tiles = super::svg_animated_enumeration::create(scope, NO_STITCH as u32)?;
    let turbulence_type = super::svg_animated_enumeration::create(scope, TURBULENCE as u32)?;
    let x = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let y = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let width = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let height = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let result = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        base_frequency_x: v8::Global::new(scope, base_frequency_x),
        base_frequency_y: v8::Global::new(scope, base_frequency_y),
        num_octaves: v8::Global::new(scope, num_octaves),
        seed: v8::Global::new(scope, seed),
        stitch_tiles: v8::Global::new(scope, stitch_tiles),
        turbulence_type: v8::Global::new(scope, turbulence_type),
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        width: v8::Global::new(scope, width),
        height: v8::Global::new(scope, height),
        result: v8::Global::new(scope, result),
    };
    scope
        .get_slot_mut::<SvgFeTurbulenceElementStore>()
        .ok_or_else(|| "SVGFETurbulenceElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGFETurbulenceElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgFeTurbulenceElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_object(
    scope: &v8::PinScope<'_, '_>,
    value: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, value).into());
}
pub(crate) fn get_base_frequency_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.base_frequency_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_base_frequency_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.base_frequency_y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_num_octaves(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.num_octaves, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_seed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.seed, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_stitch_tiles(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.stitch_tiles, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.turbulence_type, r)
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
        return_object(s, &v.x, r)
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
        return_object(s, &v.y, r)
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
        return_object(s, &v.width, r)
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
        return_object(s, &v.height, r)
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
        return_object(s, &v.result, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
