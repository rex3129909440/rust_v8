pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getSelection", 0, get_selection)
}
fn get_selection(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(document) = super::node::record(scope, a.this())
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document))
    else {
        r.set(v8::null(scope).into());
        return;
    };
    match super::selection::for_document(scope, document) {
        Ok(v) => r.set(v.into()),
        Err(m) => crate::webidl::throw_type_error(scope, &m),
    }
}
