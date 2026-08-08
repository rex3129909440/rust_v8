pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "childNodes", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::node::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(cached) = record.child_nodes {
        result.set(v8::Local::new(scope, &cached).into());
        return;
    }
    match super::node_list::create_live_child_nodes(scope, arguments.this()) {
        Ok(list) => {
            super::node::cache_child_nodes(scope, arguments.this(), list);
            result.set(list.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
