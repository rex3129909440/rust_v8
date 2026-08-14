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
    crate::webidl::return_array_like_iterator(
        scope,
        arguments.this(),
        crate::webidl::ArrayLikeIteratorKind::Values,
        result,
    );
}
