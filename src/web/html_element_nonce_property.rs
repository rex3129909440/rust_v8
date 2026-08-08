use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "nonce", get_value, set_value)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let reflected = reflected_attribute_name("nonce")
        .and_then(|attribute| super::element::attribute_value(scope, arguments.this(), attribute));
    let value = reflected
        .as_deref()
        .or_else(|| record.strings.get("nonce").map(String::as_str))
        .unwrap_or("");
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(attribute) = reflected_attribute_name("nonce") {
        super::element::set_attribute_full(
            scope,
            arguments.this(),
            attribute.to_owned(),
            value.clone(),
            None,
        );
    }
    if let Some(record) = scope.get_slot_mut::<HtmlElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.strings.insert("nonce".to_owned(), value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
