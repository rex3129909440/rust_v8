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
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'createTreeWalker' on 'Document': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(root) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'createTreeWalker' on 'Document': parameter 1 is not of type 'Node'.",
        );
        return;
    };
    if super::node::record(scope, root).is_none() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'createTreeWalker' on 'Document': parameter 1 is not of type 'Node'.",
        );
        return;
    }
    let what_to_show = if arguments.get(1).is_undefined() {
        None
    } else {
        let value = arguments.get(1);
        if value.is_symbol() || value.is_big_int() {
            let kind = if value.is_symbol() {
                "Symbol"
            } else {
                "BigInt"
            };
            crate::webidl::throw_type_error(
                scope,
                &format!(
                    "Failed to execute 'createTreeWalker' on 'Document': Cannot convert a {kind} value to a number"
                ),
            );
            return;
        }
        let Some(value) = value.uint32_value(scope) else {
            return;
        };
        Some(value)
    };
    let filter = if arguments.get(2).is_null_or_undefined() {
        None
    } else {
        let Ok(filter) = v8::Local::<v8::Object>::try_from(arguments.get(2)) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'createTreeWalker' on 'Document': parameter 3 is not of type 'Object'.",
            );
            return;
        };
        Some(filter)
    };
    match super::tree_walker::create(scope, root, what_to_show, filter) {
        Ok(walker) => result.set(walker.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
