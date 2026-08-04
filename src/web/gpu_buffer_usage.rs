pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let namespace = v8::Object::new(scope);
    crate::webidl::define_constant(scope, namespace, "MAP_READ", 1)?;
    crate::webidl::define_constant(scope, namespace, "MAP_WRITE", 2)?;
    crate::webidl::define_constant(scope, namespace, "COPY_SRC", 4)?;
    crate::webidl::define_constant(scope, namespace, "COPY_DST", 8)?;
    crate::webidl::define_constant(scope, namespace, "INDEX", 16)?;
    crate::webidl::define_constant(scope, namespace, "VERTEX", 32)?;
    crate::webidl::define_constant(scope, namespace, "UNIFORM", 64)?;
    crate::webidl::define_constant(scope, namespace, "STORAGE", 128)?;
    crate::webidl::define_constant(scope, namespace, "INDIRECT", 256)?;
    crate::webidl::define_constant(scope, namespace, "QUERY_RESOLVE", 512)?;
    let tag = v8::Symbol::get_to_string_tag(scope);
    let value = crate::webidl::string(scope, "GPUBufferUsage")?;
    let _ = namespace.define_own_property(
        scope,
        tag.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    );
    crate::webidl::define_global(scope, "GPUBufferUsage", namespace.into())
}
