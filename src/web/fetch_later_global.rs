pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "fetchLater",
        1,
        v8::ConstructorBehavior::Throw,
        fetch_later,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "fetchLater")?;
    match global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot define window.fetchLater".to_owned()),
    }
}

fn fetch_later(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fetchLater' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    if url::Url::parse(&input).is_err() {
        crate::webidl::throw_type_error(scope, "Failed to parse the deferred fetch URL.");
        return;
    }
    match super::fetch_later_result::create(scope, true) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
