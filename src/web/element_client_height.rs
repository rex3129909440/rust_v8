pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "clientHeight", get_client_height)
}

fn get_client_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::element::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.tag_name.eq_ignore_ascii_case("BODY")
        && let Some(height) = crate::fingerprint::edge(scope).document.body_client_height
    {
        result.set(v8::Integer::new(scope, super::element_layout::rounded(height)).into());
        return;
    }
    let metrics = super::element_layout::scroll_metrics(scope, arguments.this());
    result
        .set(v8::Integer::new(scope, super::element_layout::rounded(metrics.client_height)).into());
}
