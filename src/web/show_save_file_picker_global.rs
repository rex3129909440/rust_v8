pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "showSaveFilePicker",
        0,
        v8::ConstructorBehavior::Throw,
        show_save_file_picker,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "showSaveFilePicker")?;
    match global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot define window.showSaveFilePicker".to_owned()),
    }
}

fn show_save_file_picker(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::file_system_file_handle::create(scope, "untitled".to_owned()) {
        Ok(handle) => {
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, handle.into()) {
                result.set(promise.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
