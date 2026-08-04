use std::collections::HashMap;

pub(crate) const LENGTH_ADJUST_UNKNOWN: i32 = 0;
pub(crate) const LENGTH_ADJUST_SPACING: i32 = 1;
pub(crate) const LENGTH_ADJUST_SPACING_AND_GLYPHS: i32 = 2;

#[derive(Default)]
pub(crate) struct SvgTextContentElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) text_length: v8::Global<v8::Object>,
    pub(crate) length_adjust: v8::Global<v8::Object>,
    pub(crate) text: String,
    pub(crate) selected: Option<(u32, u32)>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgTextContentElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGTextContentElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgTextContentElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGTextContentElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_text_content_element_text_length_property::define(scope, prototype)?;
    super::svg_text_content_element_length_adjust_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    super::svg_text_content_element_get_char_num_at_position::define(scope, prototype)?;
    super::svg_text_content_element_get_computed_text_length::define(scope, prototype)?;
    super::svg_text_content_element_get_end_position_of_char::define(scope, prototype)?;
    super::svg_text_content_element_get_extent_of_char::define(scope, prototype)?;
    super::svg_text_content_element_get_number_of_chars::define(scope, prototype)?;
    super::svg_text_content_element_get_rotation_of_char::define(scope, prototype)?;
    super::svg_text_content_element_get_start_position_of_char::define(scope, prototype)?;
    super::svg_text_content_element_get_sub_string_length::define(scope, prototype)?;
    super::svg_text_content_element_select_sub_string::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_graphics_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgTextContentElementStore>()
        .ok_or_else(|| "SVGTextContentElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "LENGTHADJUST_UNKNOWN", LENGTH_ADJUST_UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "LENGTHADJUST_SPACING", LENGTH_ADJUST_SPACING)?;
    crate::webidl::define_constant(
        scope,
        object,
        "LENGTHADJUST_SPACINGANDGLYPHS",
        LENGTH_ADJUST_SPACING_AND_GLYPHS,
    )
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner: Option<v8::Local<'s, v8::Object>>,
    text: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object =
        super::svg_graphics_element::create_with_constructor(scope, constructor, tag_name, owner)?;
    attach(scope, object, text)?;
    Ok(object)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    text: &str,
) -> Result<(), String> {
    let text_length = super::svg_animated_length::create(scope, glyph_count(text) as f64 * 10.0)?;
    let length_adjust =
        super::svg_animated_enumeration::create(scope, LENGTH_ADJUST_SPACING as u32)?;
    let record = Record {
        text_length: v8::Global::new(scope, text_length),
        length_adjust: v8::Global::new(scope, length_adjust),
        text: text.to_owned(),
        selected: None,
    };
    scope
        .get_slot_mut::<SvgTextContentElementStore>()
        .ok_or_else(|| "SVGTextContentElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(())
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGTextContentElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgTextContentElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn glyph_count(text: &str) -> usize {
    text.chars().count()
}

pub(crate) fn get_text_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.text_length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_length_adjust(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.length_adjust).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_number_of_chars(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, glyph_count(&record.text) as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_computed_text_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, glyph_count(&record.text) as f64 * 10.0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn character_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<(Record, usize)> {
    let record = record(scope, arguments.this())?;
    let index = arguments.get(0).uint32_value(scope)? as usize;
    (index < glyph_count(&record.text)).then_some((record, index))
}

pub(crate) fn return_point(
    scope: &mut v8::PinScope<'_, '_>,
    x: f64,
    y: f64,
    mut result: v8::ReturnValue<'_>,
) {
    match super::svg_point::create(scope, super::svg_point::PointValue { x, y }) {
        Ok(point) => result.set(point.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn get_start_position_of_char(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some((_, index)) = character_index(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    };
    return_point(scope, index as f64 * 10.0, 0.0, result);
}

pub(crate) fn get_end_position_of_char(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some((_, index)) = character_index(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    };
    return_point(scope, (index + 1) as f64 * 10.0, 0.0, result);
}

pub(crate) fn get_extent_of_char(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some((_, index)) = character_index(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    };
    match super::svg_rect::create_pair(
        scope,
        super::svg_rect::RectValue {
            x: index as f64 * 10.0,
            y: -10.0,
            width: 10.0,
            height: 12.0,
        },
    ) {
        Ok((rect, _)) => result.set(rect.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn get_rotation_of_char(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if character_index(scope, &arguments).is_some() {
        result.set(v8::Number::new(scope, 0.0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
    }
}

pub(crate) fn get_sub_string_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let start = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let count = arguments.get(1).uint32_value(scope).unwrap_or(0) as usize;
    let total = glyph_count(&record.text);
    if start >= total && count != 0 {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    }
    result.set(v8::Number::new(scope, count.min(total.saturating_sub(start)) as f64 * 10.0).into());
}

pub(crate) fn select_sub_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let start = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX);
    let count = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let identity = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<SvgTextContentElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if start as usize >= glyph_count(&record.text) && count != 0 {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    }
    record.selected = Some((start, count));
}

pub(crate) fn get_char_num_at_position(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let x = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|point| {
            let key = v8::String::new(scope, "x")?;
            point.get(scope, key.into())?.number_value(scope)
        })
        .unwrap_or(-1.0);
    let index = (x / 10.0).floor() as i32;
    let output = if index >= 0 && index < glyph_count(&record.text) as i32 {
        index
    } else {
        -1
    };
    result.set(v8::Integer::new(scope, output).into());
}
