pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = super::media_stream::ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "webkitMediaStream", constructor.into())
}
