use super::text_format_update_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getTextFormats", 0, get_text_formats)
}

fn get_text_formats(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = scope
        .get_slot::<TextFormatUpdateEventStore>()
        .and_then(|s| s.records.get(&arguments.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, values.len() as i32);
    for (i, v) in values.iter().enumerate() {
        let value = v8::Local::new(scope, v);
        let _ = array.set_index(scope, i as u32, value.into());
    }
    result.set(array.into());
}
