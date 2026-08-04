pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getAttributeNames",
        0,
        get_attribute_names,
    )
}

fn get_attribute_names(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(attributes) = super::element::attributes_snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let names = v8::Array::new(scope, attributes.len() as i32);
    for (index, attribute) in attributes.into_iter().enumerate() {
        if let Some(name) = v8::String::new(scope, &attribute.name) {
            let _ = names.set_index(scope, index as u32, name.into());
        }
    }
    result.set(names.into());
}
