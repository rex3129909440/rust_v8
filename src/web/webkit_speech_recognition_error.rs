pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = super::speech_recognition_error_event::ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "webkitSpeechRecognitionError", constructor.into())
}
