pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "queryLocalFonts",
        0,
        v8::ConstructorBehavior::Throw,
        query_local_fonts,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "queryLocalFonts")?;
    match global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot define window.queryLocalFonts".to_owned()),
    }
}

fn query_local_fonts(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if crate::fingerprint::edge(scope).permissions.local_fonts != "granted" {
        if let Ok(error) = super::dom_exception::create(
            scope,
            "Local font access requires permission".to_owned(),
            "NotAllowedError".to_owned(),
        ) && let Ok(promise) = super::writable_stream::rejected_promise(scope, error.into())
        {
            result.set(promise.into());
        }
        return;
    }
    let configured = crate::fingerprint::edge(scope).fonts.local_fonts.clone();
    let fonts = v8::Array::new(scope, configured.len() as i32);
    for (index, configured_font) in configured.into_iter().enumerate() {
        let font = match super::font_data::create(
            scope,
            configured_font.postscript_name,
            configured_font.full_name,
            configured_font.family,
            configured_font.style,
            Vec::new(),
        ) {
            Ok(font) => font,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        let _ = fonts.set_index(scope, index as u32, font.into());
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, fonts.into()) {
        result.set(promise.into());
    }
}
