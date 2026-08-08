use super::svg_text_content_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "selectSubString", 2, select_sub_string)
}

fn select_sub_string(
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
