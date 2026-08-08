use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgTextPositioningElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) dx: v8::Global<v8::Object>,
    pub(crate) dy: v8::Global<v8::Object>,
    pub(crate) rotate: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgTextPositioningElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGTextPositioningElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgTextPositioningElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGTextPositioningElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_text_positioning_element_x_property::define(scope, prototype)?;
    super::svg_text_positioning_element_y_property::define(scope, prototype)?;
    super::svg_text_positioning_element_dx_property::define(scope, prototype)?;
    super::svg_text_positioning_element_dy_property::define(scope, prototype)?;
    super::svg_text_positioning_element_rotate_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_text_content_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgTextPositioningElementStore>()
        .ok_or_else(|| "SVGTextPositioningElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner: Option<v8::Local<'s, v8::Object>>,
    text: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = super::svg_text_content_element::create_with_constructor(
        scope,
        constructor,
        tag_name,
        owner,
        text,
    )?;
    let x = super::svg_animated_length_list::create(scope)?;
    let y = super::svg_animated_length_list::create(scope)?;
    let dx = super::svg_animated_length_list::create(scope)?;
    let dy = super::svg_animated_length_list::create(scope)?;
    let rotate = super::svg_animated_number_list::create(scope)?;
    let record = Record {
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        dx: v8::Global::new(scope, dx),
        dy: v8::Global::new(scope, dy),
        rotate: v8::Global::new(scope, rotate),
    };
    scope
        .get_slot_mut::<SvgTextPositioningElementStore>()
        .ok_or_else(|| "SVGTextPositioningElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGTextPositioningElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgTextPositioningElementStore>()?
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

pub(crate) fn get_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.x, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.y, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_dx(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.dx, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_dy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.dy, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_rotate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.rotate, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
