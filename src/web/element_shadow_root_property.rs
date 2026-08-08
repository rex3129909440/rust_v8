pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "shadowRoot", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::element::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(root) = record.shadow_root else {
        result.set(v8::null(scope).into());
        return;
    };
    let root = v8::Local::new(scope, &root);
    if super::shadow_root::is_closed(scope, root) {
        result.set(v8::null(scope).into());
    } else {
        result.set(root.into());
    }
}
