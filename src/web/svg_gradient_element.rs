use std::collections::HashMap;

pub(crate) const SPREAD_UNKNOWN: i32 = 0;
pub(crate) const SPREAD_PAD: i32 = 1;
pub(crate) const SPREAD_REFLECT: i32 = 2;
pub(crate) const SPREAD_REPEAT: i32 = 3;

#[derive(Default)]
pub(crate) struct SvgGradientElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) gradient_units: v8::Global<v8::Object>,
    pub(crate) gradient_transform: v8::Global<v8::Object>,
    pub(crate) spread_method: v8::Global<v8::Object>,
    pub(crate) href: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgGradientElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGGradientElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgGradientElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGGradientElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_gradient_element_gradient_units_property::define(scope, prototype)?;
    super::svg_gradient_element_gradient_transform_property::define(scope, prototype)?;
    super::svg_gradient_element_spread_method_property::define(scope, prototype)?;
    super::svg_gradient_element_href_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgGradientElementStore>()
        .ok_or_else(|| "SVGGradientElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "SVG_SPREADMETHOD_UNKNOWN", SPREAD_UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "SVG_SPREADMETHOD_PAD", SPREAD_PAD)?;
    crate::webidl::define_constant(scope, object, "SVG_SPREADMETHOD_REFLECT", SPREAD_REFLECT)?;
    crate::webidl::define_constant(scope, object, "SVG_SPREADMETHOD_REPEAT", SPREAD_REPEAT)
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = super::svg_element::create_with_constructor(scope, constructor, tag_name, owner)?;
    let gradient_units = super::svg_animated_enumeration::create(scope, 2)?;
    let gradient_transform = super::svg_animated_transform_list::create(scope)?;
    let spread_method = super::svg_animated_enumeration::create(scope, SPREAD_PAD as u32)?;
    let href = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        gradient_units: v8::Global::new(scope, gradient_units),
        gradient_transform: v8::Global::new(scope, gradient_transform),
        spread_method: v8::Global::new(scope, spread_method),
        href: v8::Global::new(scope, href),
    };
    scope
        .get_slot_mut::<SvgGradientElementStore>()
        .ok_or_else(|| "SVGGradientElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGGradientElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgGradientElementStore>()?
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

pub(crate) fn get_gradient_units(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.gradient_units, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_gradient_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.gradient_transform, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_spread_method(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.spread_method, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.href, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
