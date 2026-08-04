pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "expand", 0, expand)
}
fn expand(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let unit = if arguments.length() == 0 || arguments.get(0).is_undefined() {
        "word".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase()
    };
    if unit != "word" {
        return;
    }

    let start_container = v8::Local::new(scope, &record.start_container);
    let end_container = v8::Local::new(scope, &record.end_container);
    let Some(start_data) = super::character_data::data_if_character(scope, start_container) else {
        return;
    };
    let Some(end_data) = super::character_data::data_if_character(scope, end_container) else {
        return;
    };
    let start_units: Vec<u16> = start_data.encode_utf16().collect();
    let end_units: Vec<u16> = end_data.encode_utf16().collect();
    let is_word = |unit: u16| {
        char::from_u32(unit as u32).is_some_and(|character| {
            character.is_alphanumeric() || character == '_' || character == '-'
        })
    };
    let mut start = (record.start_offset as usize).min(start_units.len());
    while start > 0 && is_word(start_units[start - 1]) {
        start -= 1;
    }
    let mut end = (record.end_offset as usize).min(end_units.len());
    while end < end_units.len() && is_word(end_units[end]) {
        end += 1;
    }
    super::abstract_range::update(scope, arguments.this(), |range| {
        range.start_offset = start as u32;
        range.end_offset = end as u32;
    });
}
