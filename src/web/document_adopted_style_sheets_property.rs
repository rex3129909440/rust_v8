pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "adoptedStyleSheets", get_sheets, set_sheets)
}
fn get_sheets(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    if super::document_property_support::return_stored(s, a.this(), "adoptedStyleSheets", r) {
        return;
    }
    let value = v8::Array::new(s, 0);
    super::document::remember_value(s, a.this(), "adoptedStyleSheets", value.into());
    r.set(value.into());
}
fn set_sheets(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let Ok(sequence) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "adoptedStyleSheets must be a sequence");
        return;
    };
    let length = v8::String::new(s, "length")
        .and_then(|key| sequence.get(s, key.into()))
        .and_then(|value| value.uint32_value(s))
        .unwrap_or(0);
    let array = v8::Array::new(s, length as i32);
    for index in 0..length {
        let Some(sheet) = sequence.get_index(s, index) else {
            crate::webidl::throw_type_error(s, "Cannot read adoptedStyleSheets");
            return;
        };
        let Ok(sheet_object) = v8::Local::<v8::Object>::try_from(sheet) else {
            crate::webidl::throw_type_error(s, "Failed to convert value to 'CSSStyleSheet'.");
            return;
        };
        if !super::css_style_sheet::is_constructed(s, sheet_object) {
            crate::webidl::throw_type_error(s, "Failed to convert value to 'CSSStyleSheet'.");
            return;
        }
        let _ = array.set_index(s, index, sheet);
    }
    super::document::remember_value(s, a.this(), "adoptedStyleSheets", array.into());
}

pub(crate) fn sheets<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let Some(value) = super::document::stored_value(scope, document, "adoptedStyleSheets") else {
        return Vec::new();
    };
    let value = v8::Local::new(scope, &value);
    let Ok(array) = v8::Local::<v8::Object>::try_from(value) else {
        return Vec::new();
    };
    let length = v8::String::new(scope, "length")
        .and_then(|key| array.get(scope, key.into()))
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    (0..length)
        .filter_map(|index| array.get_index(scope, index))
        .filter_map(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .collect()
}
