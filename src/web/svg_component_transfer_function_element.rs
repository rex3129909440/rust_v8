use std::collections::HashMap;

pub(crate) const TYPE_UNKNOWN: i32 = 0;
pub(crate) const TYPE_IDENTITY: i32 = 1;
pub(crate) const TYPE_TABLE: i32 = 2;
pub(crate) const TYPE_DISCRETE: i32 = 3;
pub(crate) const TYPE_LINEAR: i32 = 4;
pub(crate) const TYPE_GAMMA: i32 = 5;

#[derive(Default)]
pub(crate) struct SvgComponentTransferFunctionElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) transfer_type: v8::Global<v8::Object>,
    pub(crate) table_values: v8::Global<v8::Object>,
    pub(crate) slope: v8::Global<v8::Object>,
    pub(crate) intercept: v8::Global<v8::Object>,
    pub(crate) amplitude: v8::Global<v8::Object>,
    pub(crate) exponent: v8::Global<v8::Object>,
    pub(crate) offset: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgComponentTransferFunctionElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "SVGComponentTransferFunctionElement",
        constructor.into(),
    )
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgComponentTransferFunctionElementStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGComponentTransferFunctionElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_component_transfer_function_element_type_property::define(scope, prototype)?;
    super::svg_component_transfer_function_element_table_values_property::define(scope, prototype)?;
    super::svg_component_transfer_function_element_slope_property::define(scope, prototype)?;
    super::svg_component_transfer_function_element_intercept_property::define(scope, prototype)?;
    super::svg_component_transfer_function_element_amplitude_property::define(scope, prototype)?;
    super::svg_component_transfer_function_element_exponent_property::define(scope, prototype)?;
    super::svg_component_transfer_function_element_offset_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgComponentTransferFunctionElementStore>()
        .ok_or_else(|| "SVGComponentTransferFunctionElement state was not prepared".to_owned())?
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
        "SVG_FECOMPONENTTRANSFER_TYPE_UNKNOWN",
        TYPE_UNKNOWN,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_FECOMPONENTTRANSFER_TYPE_IDENTITY",
        TYPE_IDENTITY,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_FECOMPONENTTRANSFER_TYPE_TABLE",
        TYPE_TABLE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_FECOMPONENTTRANSFER_TYPE_DISCRETE",
        TYPE_DISCRETE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_FECOMPONENTTRANSFER_TYPE_LINEAR",
        TYPE_LINEAR,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_FECOMPONENTTRANSFER_TYPE_GAMMA",
        TYPE_GAMMA,
    )
}
pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = super::svg_element::create_with_constructor(scope, constructor, tag_name, owner)?;
    let transfer_type = super::svg_animated_enumeration::create(scope, TYPE_IDENTITY as u32)?;
    let table_values = super::svg_animated_number_list::create(scope)?;
    let slope = super::svg_animated_number::create(scope, 1.0)?;
    let intercept = super::svg_animated_number::create(scope, 0.0)?;
    let amplitude = super::svg_animated_number::create(scope, 1.0)?;
    let exponent = super::svg_animated_number::create(scope, 1.0)?;
    let offset = super::svg_animated_number::create(scope, 0.0)?;
    let record = Record {
        transfer_type: v8::Global::new(scope, transfer_type),
        table_values: v8::Global::new(scope, table_values),
        slope: v8::Global::new(scope, slope),
        intercept: v8::Global::new(scope, intercept),
        amplitude: v8::Global::new(scope, amplitude),
        exponent: v8::Global::new(scope, exponent),
        offset: v8::Global::new(scope, offset),
    };
    scope
        .get_slot_mut::<SvgComponentTransferFunctionElementStore>()
        .ok_or_else(|| "SVGComponentTransferFunctionElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGComponentTransferFunctionElement': Illegal constructor",
    )
}
pub(crate) fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgComponentTransferFunctionElementStore>()?
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
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.transfer_type, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_table_values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.table_values, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_slope(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.slope, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_intercept(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.intercept, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_amplitude(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.amplitude, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_exponent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.exponent, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_offset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.offset, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
