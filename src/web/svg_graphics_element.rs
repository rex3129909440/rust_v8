use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgGraphicsElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) transform: v8::Global<v8::Object>,
    pub(crate) required_extensions: v8::Global<v8::Object>,
    pub(crate) system_language: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgGraphicsElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGGraphicsElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgGraphicsElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGGraphicsElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_graphics_element_transform_property::define(scope, prototype)?;
    super::svg_graphics_element_nearest_viewport_element_property::define(scope, prototype)?;
    super::svg_graphics_element_farthest_viewport_element_property::define(scope, prototype)?;
    super::svg_graphics_element_required_extensions_property::define(scope, prototype)?;
    super::svg_graphics_element_system_language_property::define(scope, prototype)?;
    super::svg_graphics_element_get_b_box::define(scope, prototype)?;
    super::svg_graphics_element_get_ctm::define(scope, prototype)?;
    super::svg_graphics_element_get_screen_ctm::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgGraphicsElementStore>()
        .ok_or_else(|| "SVGGraphicsElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = super::svg_element::create_with_constructor(scope, constructor, tag_name, owner)?;
    attach(scope, object)?;
    Ok(object)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let transform = super::svg_animated_transform_list::create(scope)?;
    let required_extensions = super::svg_string_list::create(scope, Vec::new())?;
    let system_language = super::svg_string_list::create(scope, Vec::new())?;
    let transform = v8::Global::new(scope, transform);
    let required_extensions = v8::Global::new(scope, required_extensions);
    let system_language = v8::Global::new(scope, system_language);
    scope
        .get_slot_mut::<SvgGraphicsElementStore>()
        .ok_or_else(|| "SVGGraphicsElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            Record {
                transform,
                required_extensions,
                system_language,
            },
        );
    Ok(())
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGGraphicsElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgGraphicsElementStore>()?
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

pub(crate) fn get_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.transform, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_nearest_viewport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_farthest_viewport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_nearest_viewport(scope, arguments, result);
}

pub(crate) fn get_required_extensions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.required_extensions, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_system_language(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.system_language, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_bbox(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::svg_rect::create_pair(
        scope,
        super::svg_rect::RectValue {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    ) {
        Ok((rect, _)) => result.set(rect.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn get_ctm(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::svg_matrix::create(scope, super::svg_matrix::MatrixValue::identity()) {
        Ok(matrix) => result.set(matrix.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
