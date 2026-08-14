pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getHTML", 0, get_html)
}

fn get_html(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let Some(options) = super::dom_html::get_html_options(scope, arguments.get(0), "Element")
    else {
        return;
    };
    let html =
        super::dom_html::serialize_element_contents_for_get_html(scope, arguments.this(), &options);
    if let Some(html) = v8::String::new(scope, &html) {
        result.set(html.into());
    }
}
