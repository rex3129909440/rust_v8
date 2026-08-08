use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgUseElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) href: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgUseElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGUseElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgUseElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGUseElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_use_element_x_property::define(scope, prototype)?;
    super::svg_use_element_y_property::define(scope, prototype)?;
    super::svg_use_element_width_property::define(scope, prototype)?;
    super::svg_use_element_height_property::define(scope, prototype)?;
    super::svg_use_element_href_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_graphics_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgUseElementStore>()
        .ok_or_else(|| "SVGUseElement state was not prepared".to_owned())?
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
        super::svg_graphics_element::create_with_constructor(scope, constructor, "use", owner)?;
    let x = super::svg_animated_length::create(scope, 0.0)?;
    let y = super::svg_animated_length::create(scope, 0.0)?;
    let width = super::svg_animated_length::create(scope, 0.0)?;
    let height = super::svg_animated_length::create(scope, 0.0)?;
    let href = super::svg_animated_string::create(scope, "")?;
    let x = v8::Global::new(scope, x);
    let y = v8::Global::new(scope, y);
    let width = v8::Global::new(scope, width);
    let height = v8::Global::new(scope, height);
    let href = v8::Global::new(scope, href);
    scope
        .get_slot_mut::<SvgUseElementStore>()
        .ok_or_else(|| "SVGUseElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            Record {
                x,
                y,
                width,
                height,
                href,
            },
        );
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGUseElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgUseElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_value(
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
        return_value(scope, &record.x, result);
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
        return_value(scope, &record.y, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_value(scope, &record.width, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_value(scope, &record.height, result);
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
        return_value(scope, &record.href, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
