pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "values", 0, v8::ConstructorBehavior::Throw, values)?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, function, &format!("{owner}.values"));
    }
    let key = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define NamedNodeMap iterator".to_owned())
    }
}
fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(attributes) = super::named_node_map::attributes(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, attributes.len() as i32);
    for (index, attribute) in attributes.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, (*attribute).into());
    }
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(function) = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    if let Some(iterator) = function.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}
