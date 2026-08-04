use super::dom_parser::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "parseFromString", 2, parse_from_string)
}

fn parse_from_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_instance(scope, arguments.this()) {
        return;
    }
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let content_type = crate::webidl::value_to_string(scope, arguments.get(1));
    let document = match content_type.as_str() {
        "text/html" => {
            let title = html_title(&source);
            match super::html_document::create_from_source(scope, html_source(&source)) {
                Ok(document) => {
                    if let Some(title) = title {
                        super::document::set_string_value(scope, document, "title", title.trim());
                    }
                    Ok(document)
                }
                Err(message) => Err(message),
            }
        }
        "text/xml" => super::xml_document::create_with_type(scope, source, "text/xml"),
        "application/xml" => {
            super::xml_document::create_with_type(scope, source, "application/xml")
        }
        "application/xhtml+xml" => {
            super::xml_document::create_with_type(scope, source, "application/xhtml+xml")
        }
        "image/svg+xml" => super::xml_document::create_with_type(scope, source, "image/svg+xml"),
        _ => {
            crate::webidl::throw_type_error(
                scope,
                "The provided value is not a valid enum value of type SupportedType",
            );
            return;
        }
    };
    match document {
        Ok(document) => result.set(document.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
