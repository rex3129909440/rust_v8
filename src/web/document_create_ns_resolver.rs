pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createNSResolver", 1, create_ns_resolver)
}

fn create_ns_resolver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let value = arguments.get(0);
    let Ok(node) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    if super::node::record(scope, node).is_none() {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    }
    result.set(node.into());
}
