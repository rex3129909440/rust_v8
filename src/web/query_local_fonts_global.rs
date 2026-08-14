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
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::user_activation::current_realm_has_been_active(scope) {
        if let Ok(error) = super::dom_exception::create(
            scope,
            "User activation is required.".to_owned(),
            "SecurityError".to_owned(),
        ) && let Ok(promise) = super::writable_stream::rejected_promise(scope, error.into())
        {
            result.set(promise.into());
        }
        return;
    }
    let permission = crate::fingerprint::edge(scope)
        .permissions
        .local_fonts
        .clone();
    if permission == "prompt" && !super::user_activation::consume_current_realm(scope) {
        if let Ok(error) = super::dom_exception::create(
            scope,
            "User activation is required.".to_owned(),
            "SecurityError".to_owned(),
        ) && let Ok(promise) = super::writable_stream::rejected_promise(scope, error.into())
        {
            result.set(promise.into());
        }
        return;
    }
    if permission != "granted" {
        let fonts = v8::Array::new(scope, 0);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, fonts.into()) {
            result.set(promise.into());
        }
        return;
    }
    let filter = match postscript_name_filter(scope, arguments.get(0)) {
        Ok(filter) => filter,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let configured = crate::fingerprint::edge(scope)
        .fonts
        .local_fonts
        .clone()
        .into_iter()
        .filter(|font| {
            filter
                .as_ref()
                .is_none_or(|names| names.contains(&font.postscript_name))
        })
        .collect::<Vec<_>>();
    let fonts = v8::Array::new(scope, configured.len() as i32);
    for (index, configured_font) in configured.into_iter().enumerate() {
        let bytes = crate::font_shaping::local_font_bytes(
            scope,
            &configured_font.postscript_name,
            &configured_font.family,
        );
        let font = match super::font_data::create(
            scope,
            configured_font.postscript_name,
            configured_font.full_name,
            configured_font.family,
            configured_font.style,
            bytes,
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

fn postscript_name_filter(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Option<std::collections::HashSet<String>>, String> {
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let options = v8::Local::<v8::Object>::try_from(value).map_err(|_| {
        "Failed to execute 'queryLocalFonts' on 'Window': parameter 1 is not of type 'QueryOptions'."
            .to_owned()
    })?;
    let key = v8::String::new(scope, "postscriptNames")
        .ok_or_else(|| "cannot create postscriptNames key".to_owned())?;
    let names = options
        .get(scope, key.into())
        .ok_or_else(|| "cannot read QueryOptions.postscriptNames".to_owned())?;
    if names.is_undefined() {
        return Ok(None);
    }
    let values = crate::webidl::sequence_values(scope, names).map_err(|_| {
        "Failed to execute 'queryLocalFonts' on 'Window': Failed to read the 'postscriptNames' property from 'QueryOptions': The object must have a callable @@iterator property."
            .to_owned()
    })?;
    Ok(Some(
        values
            .into_iter()
            .map(|value| {
                let value = v8::Local::new(scope, &value);
                crate::webidl::value_to_string(scope, value)
            })
            .collect(),
    ))
}
