pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "body", get_body, set_body)
}
fn get_body(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = super::document_property_support::find_html_element(s, a.this(), "BODY")
        .or_else(|| super::document_property_support::find_html_element(s, a.this(), "FRAMESET"));
    super::document_property_support::return_optional(s, value, r);
}
fn set_body(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let existing = super::document_property_support::find_html_element(s, a.this(), "BODY")
        .or_else(|| super::document_property_support::find_html_element(s, a.this(), "FRAMESET"));
    if a.get(0).is_null_or_undefined() {
        if let Some(existing) = existing {
            super::node::detach(s, existing);
        }
        return;
    }
    let Ok(value) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "The new body must be a body or frameset element");
        return;
    };
    let valid = super::element::record(s, value).is_some_and(|record| {
        record.tag_name.eq_ignore_ascii_case("BODY")
            || record.tag_name.eq_ignore_ascii_case("FRAMESET")
    });
    if !valid {
        super::node::throw_dom_exception(
            s,
            "HierarchyRequestError",
            "The new body must be a body or frameset element",
        );
        return;
    }
    let Some(root) = super::document::document_child_elements(s, a.this())
        .into_iter()
        .next()
    else {
        super::node::throw_dom_exception(
            s,
            "HierarchyRequestError",
            "The document has no document element",
        );
        return;
    };
    let index = existing
        .and_then(|old| {
            let children = super::node::children(s, root);
            let index = children
                .iter()
                .position(|child| child.strict_equals(old.into()));
            super::node::detach(s, old);
            index
        })
        .unwrap_or_else(|| super::node::children(s, root).len());
    if let Err((name, message)) = super::node::insert_node(s, root, value, index) {
        super::node::throw_dom_exception(s, name, message);
    }
}
