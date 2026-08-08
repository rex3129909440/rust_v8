use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgLinearGradientElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) x1: v8::Global<v8::Object>,
    pub(crate) y1: v8::Global<v8::Object>,
    pub(crate) x2: v8::Global<v8::Object>,
    pub(crate) y2: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgLinearGradientElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGLinearGradientElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgLinearGradientElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGLinearGradientElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_linear_gradient_element_x1_property::define(scope, prototype)?;
    super::svg_linear_gradient_element_y1_property::define(scope, prototype)?;
    super::svg_linear_gradient_element_x2_property::define(scope, prototype)?;
    super::svg_linear_gradient_element_y2_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_gradient_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgLinearGradientElementStore>()
        .ok_or_else(|| "SVGLinearGradientElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object = super::svg_gradient_element::create_with_constructor(
        scope,
        constructor,
        "linearGradient",
        owner,
    )?;
    let x1 = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let y1 = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let x2 = super::svg_animated_length::create_with_unit(scope, 2, 100.0)?;
    let y2 = super::svg_animated_length::create_with_unit(scope, 2, 0.0)?;
    let record = Record {
        x1: v8::Global::new(scope, x1),
        y1: v8::Global::new(scope, y1),
        x2: v8::Global::new(scope, x2),
        y2: v8::Global::new(scope, y2),
    };
    scope
        .get_slot_mut::<SvgLinearGradientElementStore>()
        .ok_or_else(|| "SVGLinearGradientElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGLinearGradientElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgLinearGradientElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_object(
    scope: &v8::PinScope<'_, '_>,
    object: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, object).into());
}

pub(crate) fn get_x1(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.x1, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn get_y1(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.y1, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn get_x2(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.x2, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn get_y2(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.y2, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
