pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "parentElement", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::node::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let parent = super::node::parent(scope, arguments.this()).filter(|parent| {
        super::node::record(scope, *parent)
            .is_some_and(|record| record.node_type == super::node::ELEMENT_NODE)
    });
    match parent {
        Some(parent) => result.set(parent.into()),
        None => result.set(v8::null(scope).into()),
    }
}
