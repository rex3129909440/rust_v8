pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = super::mutation_observer::ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WebKitMutationObserver", constructor.into())
}
