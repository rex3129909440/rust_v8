pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "activeElement", get_active_element)
}
fn get_active_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    if let Some(value) = super::document::stored_value(s, a.this(), "activeElement")
        && let Ok(mut element) = v8::Local::<v8::Object>::try_from(v8::Local::new(s, &value))
    {
        loop {
            let root = super::node::root_node(s, element);
            let Some(host) = super::shadow_root::host(s, root) else {
                break;
            };
            element = host;
        }
        r.set(element.into());
        return;
    }
    let value =
        super::document_property_support::find_html_element(s, a.this(), "BODY").or_else(|| {
            super::document::document_child_elements(s, a.this())
                .into_iter()
                .next()
        });
    super::document_property_support::return_optional(s, value, r);
}
