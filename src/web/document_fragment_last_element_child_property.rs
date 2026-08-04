pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "lastElementChild",
        get_last_element_child,
    )
}

fn get_last_element_child(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_fragment::valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::document_fragment::elements(scope, arguments.this()).last() {
        Some(element) => result.set((*element).into()),
        None => result.set(v8::null(scope).into()),
    }
}
