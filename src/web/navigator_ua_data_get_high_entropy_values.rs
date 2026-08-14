pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getHighEntropyValues", 1, get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::navigator_ua_data::record(scope, arguments.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "NavigatorUAData",
            "getHighEntropyValues",
            result,
        );
        return;
    };
    let output = super::navigator_ua_data::to_object(scope, &record);
    let Ok(hints) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The hints must be a sequence");
        return;
    };
    let Some(length_key) = v8::String::new(scope, "length") else {
        return;
    };
    let length = hints
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    for index in 0..length {
        let hint = hints
            .get_index(scope, index)
            .map(|value| crate::webidl::value_to_string(scope, value))
            .unwrap_or_default();
        match hint.as_str() {
            "architecture" => super::navigator_ua_data::define_string(
                scope,
                output,
                "architecture",
                &record.profile.architecture,
            ),
            "bitness" => super::navigator_ua_data::define_string(
                scope,
                output,
                "bitness",
                &record.profile.bitness,
            ),
            "model" => super::navigator_ua_data::define_string(
                scope,
                output,
                "model",
                &record.profile.model,
            ),
            "platformVersion" => super::navigator_ua_data::define_string(
                scope,
                output,
                "platformVersion",
                &record.profile.platform_version,
            ),
            "uaFullVersion" => super::navigator_ua_data::define_string(
                scope,
                output,
                "uaFullVersion",
                &record.profile.ua_full_version,
            ),
            "fullVersionList" => super::navigator_ua_data::define(
                scope,
                output,
                "fullVersionList",
                super::navigator_ua_data::brands_array(scope, &record, true).into(),
            ),
            "wow64" => super::navigator_ua_data::define(
                scope,
                output,
                "wow64",
                v8::Boolean::new(scope, record.profile.wow64).into(),
            ),
            "formFactors" => {
                let array = v8::Array::new(scope, record.profile.form_factors.len() as i32);
                for (index, factor) in record.profile.form_factors.iter().enumerate() {
                    if let Some(value) = v8::String::new(scope, factor) {
                        let _ = array.set_index(scope, index as u32, value.into());
                    }
                }
                super::navigator_ua_data::define(scope, output, "formFactors", array.into());
            }
            _ => {}
        }
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, output.into()) {
        result.set(promise.into());
    }
}
