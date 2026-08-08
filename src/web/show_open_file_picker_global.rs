pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "showOpenFilePicker",
        0,
        v8::ConstructorBehavior::Throw,
        show_open_file_picker,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "showOpenFilePicker")?;
    match global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot define window.showOpenFilePicker".to_owned()),
    }
}

fn show_open_file_picker(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let handles = v8::Array::new(scope, 1);
    match super::file_system_file_handle::create(scope, "selected-file".to_owned()) {
        Ok(handle) => {
            let _ = handles.set_index(scope, 0, handle.into());
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, handles.into()) {
                result.set(promise.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
