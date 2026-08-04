pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let namespace = v8::Object::new(scope);
    crate::webidl::define_constant(scope, namespace, "READ", 1)?;
    crate::webidl::define_constant(scope, namespace, "WRITE", 2)?;
    crate::webidl::define_to_string_tag(scope, namespace, "GPUMapMode")?;
    crate::webidl::define_global(scope, "GPUMapMode", namespace.into())
}
