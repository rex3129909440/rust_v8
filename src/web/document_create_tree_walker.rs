pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createTreeWalker", 1, create_tree_walker)
}

fn create_tree_walker(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(root) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The root must be a Node");
        return;
    };
    if super::node::record(scope, root).is_none() {
        crate::webidl::throw_type_error(scope, "The root must be a Node");
        return;
    }
    let what_to_show = if arguments.get(1).is_undefined() {
        None
    } else {
        Some(arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX))
    };
    let filter = if arguments.get(2).is_null_or_undefined() {
        None
    } else {
        Some(arguments.get(2))
    };
    match super::tree_walker::create(scope, root, what_to_show, filter) {
        Ok(walker) => result.set(walker.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
