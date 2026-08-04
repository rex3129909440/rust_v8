pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "matchMedia",
        1,
        v8::ConstructorBehavior::Throw,
        match_media,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "matchMedia")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.matchMedia".to_owned())
    }
}

fn match_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'matchMedia' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let query = crate::webidl::value_to_string(scope, arguments.get(0));
    let width = super::window_view_state::inner_width(scope);
    let height = super::window_view_state::inner_height(scope);
    let device_pixel_ratio = super::window_view_state::device_pixel_ratio(scope);
    match super::media_query_list::create(scope, query, width, height, device_pixel_ratio) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
