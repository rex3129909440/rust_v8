pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "childElementCount",
        get_child_element_count,
    )
}

fn get_child_element_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_fragment::valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    result.set(
        v8::Integer::new_from_unsigned(
            scope,
            super::document_fragment::elements(scope, arguments.this()).len() as u32,
        )
        .into(),
    );
}
