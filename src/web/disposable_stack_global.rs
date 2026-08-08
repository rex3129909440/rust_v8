pub(crate) fn install(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "DisposableStack")?;
    if global.define_own_property(scope, key.into(), value, v8::PropertyAttribute::DONT_ENUM)
        == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.DisposableStack".to_owned())
    }
}
