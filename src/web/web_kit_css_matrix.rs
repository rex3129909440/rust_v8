pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = super::dom_matrix::ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WebKitCSSMatrix", constructor.into())
}
