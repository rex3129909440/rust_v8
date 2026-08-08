use super::dom_implementation::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createHTMLDocument",
        0,
        create_html_document,
    )
}

fn create_html_document(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_instance(scope, arguments.this()) {
        return;
    }
    let document = match super::html_document::create(scope) {
        Ok(document) => document,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let doctype = match super::document_type::create(scope, "html", "", "") {
        Ok(doctype) => doctype,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let html = match super::html_html_element::create(scope) {
        Ok(html) => html,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let head = match super::html_head_element::create(scope) {
        Ok(head) => head,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let body = match super::html_body_element::create(scope) {
        Ok(body) => body,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    super::node::set_owner_document(scope, doctype, document);
    super::node::set_owner_document(scope, html, document);
    super::node::set_owner_document(scope, head, document);
    super::node::set_owner_document(scope, body, document);
    super::node::insert_child(scope, document, doctype, 0);
    super::node::insert_child(scope, document, html, 1);
    super::node::insert_child(scope, html, head, 0);
    super::node::insert_child(scope, html, body, 1);
    if !arguments.get(0).is_undefined() {
        let title_value = crate::webidl::value_to_string(scope, arguments.get(0));
        let title = match super::html_title_element::create(scope) {
            Ok(title) => title,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        let text = match super::text::create(scope, title_value.clone()) {
            Ok(text) => text,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        super::html_title_element::set_text_value(scope, title, title_value.clone());
        super::document::set_string_value(scope, document, "title", &title_value);
        super::node::set_owner_document(scope, title, document);
        super::node::set_owner_document(scope, text, document);
        super::node::insert_child(scope, title, text, 0);
        super::node::insert_child(scope, head, title, 0);
    }
    result.set(document.into());
}
