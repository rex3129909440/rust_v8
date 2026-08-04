pub(crate) fn ensure(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
) -> bool {
    if super::document::is_document(scope, document) {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn return_stored(
    scope: &v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
    mut result: v8::ReturnValue<'_>,
) -> bool {
    match super::document::stored_value(scope, document, name) {
        Some(value) => {
            result.set(v8::Local::new(scope, &value));
            true
        }
        None => false,
    }
}

pub(crate) fn find_html_element<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    super::document::document_descendants(scope, document)
        .into_iter()
        .find(|element| {
            super::node::record(scope, *element)
                .is_some_and(|record| record.node_name.eq_ignore_ascii_case(name))
        })
}

pub(crate) fn return_optional(
    scope: &v8::PinScope<'_, '_>,
    value: Option<v8::Local<'_, v8::Object>>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Some(value) => result.set(value.into()),
        None => result.set(v8::null(scope).into()),
    }
}

pub(crate) fn legacy_collection(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    property: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if return_stored(scope, document, property, result) {
        return;
    }
    match super::html_collection::create_live(
        scope,
        document,
        super::html_collection::HtmlCollectionQuery::Legacy(property.to_owned()),
    ) {
        Ok(collection) => {
            super::document::remember_value(scope, document, property, collection.into());
            result.set(collection.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn get_nullable_stored(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    if !ensure(scope, arguments.this()) {
        return;
    }
    if !return_stored(scope, arguments.this(), name, result) {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
    fallback: &str,
) {
    if !ensure(scope, arguments.this()) {
        return;
    }
    if return_stored(scope, arguments.this(), name, result) {
        return;
    }
    if let Some(value) = v8::String::new(scope, fallback) {
        result.set(value.into());
    }
}

pub(crate) fn get_body_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    attribute: &str,
) {
    if !ensure(scope, arguments.this()) {
        return;
    }
    let value = find_html_element(scope, arguments.this(), "BODY")
        .and_then(|body| super::element::attribute_value(scope, body, attribute))
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn set_body_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    attribute: &str,
) {
    if !ensure(scope, arguments.this()) {
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(body) = find_html_element(scope, arguments.this(), "BODY") {
        super::element::set_attribute_value(scope, body, attribute.to_owned(), value);
    }
}
