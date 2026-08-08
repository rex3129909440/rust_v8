pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "webkitResolveLocalFileSystemURL",
        2,
        v8::ConstructorBehavior::Throw,
        execute,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "webkitResolveLocalFileSystemURL")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.webkitResolveLocalFileSystemURL".to_owned())
    }
}

pub(crate) fn execute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'webkitResolveLocalFileSystemURL': the success callback is required.",
        );
        return;
    };
    let error_callback = v8::Local::<v8::Function>::try_from(arguments.get(2)).ok();
    if super::webkit_request_file_system::secure_origin(scope).is_none() {
        if let Some(error_callback) = error_callback
            && let Ok(error) = super::webkit_request_file_system::security_error(scope)
        {
            super::webkit_request_file_system::schedule_callback(
                scope,
                error_callback,
                error.into(),
            );
        }
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(entry) = super::webkit_request_file_system::resolve_entry(scope, &input) {
        super::webkit_request_file_system::schedule_callback(scope, success_callback, entry.into());
        return;
    }
    if let Some(error_callback) = error_callback
        && let Ok(error) = super::dom_exception::create(
            scope,
            format!("No file system entry exists for '{input}'."),
            "NotFoundError".to_owned(),
        )
    {
        super::webkit_request_file_system::schedule_callback(scope, error_callback, error.into());
    }
}
