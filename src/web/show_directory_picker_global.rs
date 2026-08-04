pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "showDirectoryPicker",
        0,
        v8::ConstructorBehavior::Throw,
        show_directory_picker,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "showDirectoryPicker")?;
    match global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot define window.showDirectoryPicker".to_owned()),
    }
}

fn show_directory_picker(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::file_system_directory_handle::create(scope, "sandbox".to_owned()) {
        Ok(handle) => {
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, handle.into()) {
                result.set(promise.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
