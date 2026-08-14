pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "appendChild", 1, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::node::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(child) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "appendChild requires a Node");
        return;
    };
    let index = super::node::children(scope, arguments.this()).len();
    match super::node::insert_node(scope, arguments.this(), child, index) {
        Ok(()) => result.set(child.into()),
        Err((name, message)) => super::node::throw_dom_exception(scope, name, message),
    }
}
