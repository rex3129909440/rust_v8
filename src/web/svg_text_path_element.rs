use std::collections::HashMap;

pub(crate) const METHOD_UNKNOWN: i32 = 0;
pub(crate) const METHOD_ALIGN: i32 = 1;
pub(crate) const METHOD_STRETCH: i32 = 2;
pub(crate) const SPACING_UNKNOWN: i32 = 0;
pub(crate) const SPACING_AUTO: i32 = 1;
pub(crate) const SPACING_EXACT: i32 = 2;

#[derive(Default)]
pub(crate) struct SvgTextPathElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) start_offset: v8::Global<v8::Object>,
    pub(crate) method: v8::Global<v8::Object>,
    pub(crate) spacing: v8::Global<v8::Object>,
    pub(crate) href: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgTextPathElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGTextPathElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgTextPathElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGTextPathElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_text_path_element_start_offset_property::define(scope, prototype)?;
    super::svg_text_path_element_method_property::define(scope, prototype)?;
    super::svg_text_path_element_spacing_property::define(scope, prototype)?;
    super::svg_text_path_element_href_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_text_content_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgTextPathElementStore>()
        .ok_or_else(|| "SVGTextPathElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "TEXTPATH_METHODTYPE_UNKNOWN", METHOD_UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "TEXTPATH_METHODTYPE_ALIGN", METHOD_ALIGN)?;
    crate::webidl::define_constant(scope, object, "TEXTPATH_METHODTYPE_STRETCH", METHOD_STRETCH)?;
    crate::webidl::define_constant(
        scope,
        object,
        "TEXTPATH_SPACINGTYPE_UNKNOWN",
        SPACING_UNKNOWN,
    )?;
    crate::webidl::define_constant(scope, object, "TEXTPATH_SPACINGTYPE_AUTO", SPACING_AUTO)?;
    crate::webidl::define_constant(scope, object, "TEXTPATH_SPACINGTYPE_EXACT", SPACING_EXACT)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
    text: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object = super::svg_text_content_element::create_with_constructor(
        scope,
        constructor,
        "textPath",
        owner,
        text,
    )?;
    let start_offset = super::svg_animated_length::create(scope, 0.0)?;
    let method = super::svg_animated_enumeration::create(scope, METHOD_ALIGN as u32)?;
    let spacing = super::svg_animated_enumeration::create(scope, SPACING_EXACT as u32)?;
    let href = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        start_offset: v8::Global::new(scope, start_offset),
        method: v8::Global::new(scope, method),
        spacing: v8::Global::new(scope, spacing),
        href: v8::Global::new(scope, href),
    };
    scope
        .get_slot_mut::<SvgTextPathElementStore>()
        .ok_or_else(|| "SVGTextPathElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGTextPathElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgTextPathElementStore>()?
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

pub(crate) fn get_start_offset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.start_offset, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_method(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.method, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_spacing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.spacing, result);
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
