pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let namespace = v8::Object::new(scope);
    crate::webidl::define_constant(scope, namespace, "COPY_SRC", 1)?;
    crate::webidl::define_constant(scope, namespace, "COPY_DST", 2)?;
    crate::webidl::define_constant(scope, namespace, "TEXTURE_BINDING", 4)?;
    crate::webidl::define_constant(scope, namespace, "STORAGE_BINDING", 8)?;
    crate::webidl::define_constant(scope, namespace, "RENDER_ATTACHMENT", 16)?;
    if crate::browser_surface::current_version(scope).major() >= 146 {
        crate::webidl::define_constant(scope, namespace, "TRANSIENT_ATTACHMENT", 32)?;
    }
    crate::webidl::define_to_string_tag(scope, namespace, "GPUTextureUsage")?;
    crate::webidl::define_global(scope, "GPUTextureUsage", namespace.into())
}
