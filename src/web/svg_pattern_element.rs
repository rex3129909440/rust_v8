use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgPatternElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) pattern_units: v8::Global<v8::Object>,
    pub(crate) pattern_content_units: v8::Global<v8::Object>,
    pub(crate) pattern_transform: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) view_box: v8::Global<v8::Object>,
    pub(crate) preserve_aspect_ratio: v8::Global<v8::Object>,
    pub(crate) href: v8::Global<v8::Object>,
    pub(crate) required_extensions: v8::Global<v8::Object>,
    pub(crate) system_language: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgPatternElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGPatternElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgPatternElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGPatternElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_pattern_element_pattern_units_property::define(scope, prototype)?;
    super::svg_pattern_element_pattern_content_units_property::define(scope, prototype)?;
    super::svg_pattern_element_pattern_transform_property::define(scope, prototype)?;
    super::svg_pattern_element_x_property::define(scope, prototype)?;
    super::svg_pattern_element_y_property::define(scope, prototype)?;
    super::svg_pattern_element_width_property::define(scope, prototype)?;
    super::svg_pattern_element_height_property::define(scope, prototype)?;
    super::svg_pattern_element_view_box_property::define(scope, prototype)?;
    super::svg_pattern_element_preserve_aspect_ratio_property::define(scope, prototype)?;
    super::svg_pattern_element_href_property::define(scope, prototype)?;
    super::svg_pattern_element_required_extensions_property::define(scope, prototype)?;
    super::svg_pattern_element_system_language_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgPatternElementStore>()
        .ok_or_else(|| "SVGPatternElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(scope, constructor, "pattern", owner)?;
    let pattern_units = super::svg_animated_enumeration::create(scope, 2)?;
    let pattern_content_units = super::svg_animated_enumeration::create(scope, 1)?;
    let pattern_transform = super::svg_animated_transform_list::create(scope)?;
    let x = super::svg_animated_length::create(scope, 0.0)?;
    let y = super::svg_animated_length::create(scope, 0.0)?;
    let width = super::svg_animated_length::create(scope, 0.0)?;
    let height = super::svg_animated_length::create(scope, 0.0)?;
    let view_box = super::svg_animated_rect::create(
        scope,
        super::svg_rect::RectValue {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    )?;
    let preserve_aspect_ratio = super::svg_animated_preserve_aspect_ratio::create(
        scope,
        super::svg_preserve_aspect_ratio::PreserveAspectRatioValue {
            align: 6,
            meet_or_slice: 1,
        },
    )?;
    let href = super::svg_animated_string::create(scope, "")?;
    let required_extensions = super::svg_string_list::create(scope, Vec::new())?;
    let system_language = super::svg_string_list::create(scope, Vec::new())?;
    let record = Record {
        pattern_units: v8::Global::new(scope, pattern_units),
        pattern_content_units: v8::Global::new(scope, pattern_content_units),
        pattern_transform: v8::Global::new(scope, pattern_transform),
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        width: v8::Global::new(scope, width),
        height: v8::Global::new(scope, height),
        view_box: v8::Global::new(scope, view_box),
        preserve_aspect_ratio: v8::Global::new(scope, preserve_aspect_ratio),
        href: v8::Global::new(scope, href),
        required_extensions: v8::Global::new(scope, required_extensions),
        system_language: v8::Global::new(scope, system_language),
    };
    scope
        .get_slot_mut::<SvgPatternElementStore>()
        .ok_or_else(|| "SVGPatternElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGPatternElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgPatternElementStore>()?
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

pub(crate) fn get_pattern_units(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.pattern_units, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_pattern_content_units(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.pattern_content_units, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_pattern_transform(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.pattern_transform, r)
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
pub(crate) fn get_view_box(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.view_box, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_preserve_aspect_ratio(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.preserve_aspect_ratio, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.href, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_required_extensions(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.required_extensions, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_system_language(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.system_language, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
