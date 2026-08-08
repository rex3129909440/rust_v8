pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let namespace = v8::Object::new(scope);
    crate::webidl::define_constant(scope, namespace, "RED", 1)?;
    crate::webidl::define_constant(scope, namespace, "GREEN", 2)?;
    crate::webidl::define_constant(scope, namespace, "BLUE", 4)?;
    crate::webidl::define_constant(scope, namespace, "ALPHA", 8)?;
    crate::webidl::define_constant(scope, namespace, "ALL", 15)?;
    let tag = v8::Symbol::get_to_string_tag(scope);
    let value = crate::webidl::string(scope, "GPUColorWrite")?;
    let _ = namespace.define_own_property(
        scope,
        tag.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    );
    crate::webidl::define_global(scope, "GPUColorWrite", namespace.into())
}
