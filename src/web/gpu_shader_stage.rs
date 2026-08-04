pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let namespace = v8::Object::new(scope);
    crate::webidl::define_constant(scope, namespace, "VERTEX", 1)?;
    crate::webidl::define_constant(scope, namespace, "FRAGMENT", 2)?;
    crate::webidl::define_constant(scope, namespace, "COMPUTE", 4)?;
    crate::webidl::define_to_string_tag(scope, namespace, "GPUShaderStage")?;
    crate::webidl::define_global(scope, "GPUShaderStage", namespace.into())
}
