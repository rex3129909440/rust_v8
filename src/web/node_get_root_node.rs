pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getRootNode", 0, call)
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
    let composed = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|options| {
            let key = v8::String::new(scope, "composed")?;
            options.get(scope, key.into())
        })
        .is_some_and(|value| value.boolean_value(scope));
    let mut node = arguments.this();
    loop {
        if let Some(parent) = super::node::parent(scope, node) {
            node = parent;
        } else if composed {
            let Some(host) = super::shadow_root::host(scope, node) else {
                break;
            };
            node = host;
        } else {
            break;
        }
    }
    result.set(node.into());
}
