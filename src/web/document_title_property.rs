pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "title", get_title, set_title)
}
fn get_title(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = super::document_property_support::find_html_element(s, a.this(), "TITLE")
        .map(|title| super::node::text_content(s, title))
        .unwrap_or_default();
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into());
    }
}
fn set_title(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    let title = if let Some(title) =
        super::document_property_support::find_html_element(s, a.this(), "TITLE")
    {
        title
    } else {
        let Some(head) = super::document_property_support::find_html_element(s, a.this(), "HEAD")
        else {
            return;
        };
        let Ok(title) = super::document::create_html_element_by_name(s, "title") else {
            return;
        };
        super::node::set_owner_document(s, title, a.this());
        let index = super::node::children(s, head).len();
        if super::node::insert_node(s, head, title, index).is_err() {
            return;
        }
        title
    };
    for child in super::node::children(s, title) {
        super::node::detach(s, child);
    }
    if !value.is_empty()
        && let Ok(text) = super::text::create(s, value)
    {
        super::node::set_owner_document(s, text, a.this());
        let _ = super::node::insert_node(s, title, text, 0);
    }
}
