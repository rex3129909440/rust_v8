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
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let count = super::node::children(scope, arguments.this())
        .into_iter()
        .filter(|child| super::element::record(scope, *child).is_some())
        .count() as u32;
    result.set(v8::Integer::new_from_unsigned(scope, count).into());
}
