pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "previousSibling", get)
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
    let value = super::node::parent(scope, arguments.this()).and_then(|parent| {
        let children = super::node::children(scope, parent);
        let identity = arguments.this().get_identity_hash().get();
        let index = children
            .iter()
            .position(|node| node.get_identity_hash().get() == identity)?;
        index.checked_sub(1).map(|index| children[index])
    });
    match value {
        Some(value) => result.set(value.into()),
        None => result.set(v8::null(scope).into()),
    }
}
