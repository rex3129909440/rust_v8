pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    constructor: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "isConfigSupported",
        1,
        is_config_supported,
    )
}

fn is_config_supported(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let input = super::webcodecs_config_support::dictionary(scope, arguments.get(0));
    let Some(codec) = super::webcodecs_config_support::string_member(scope, input, "codec") else {
        super::webcodecs_config_support::reject_type_error(
            scope,
            "Failed to execute 'isConfigSupported' on 'VideoDecoder': Failed to read the 'codec' property from 'VideoDecoderConfig': Required member is undefined.",
            result,
        );
        return;
    };
    let coded_height = super::webcodecs_config_support::number_member(scope, input, "codedHeight");
    let coded_width = super::webcodecs_config_support::number_member(scope, input, "codedWidth");
    let flip = super::webcodecs_config_support::boolean_member(scope, input, "flip", false);
    let hardware_acceleration =
        super::webcodecs_config_support::string_member(scope, input, "hardwareAcceleration")
            .unwrap_or_else(|| "no-preference".to_owned());
    let rotation =
        super::webcodecs_config_support::number_member(scope, input, "rotation").unwrap_or(0.0);
    let config = v8::Object::new(scope);
    super::webcodecs_config_support::define_string(scope, config, "codec", &codec);
    if let Some(value) = coded_height {
        super::webcodecs_config_support::define_number(scope, config, "codedHeight", value);
    }
    if let Some(value) = coded_width {
        super::webcodecs_config_support::define_number(scope, config, "codedWidth", value);
    }
    super::webcodecs_config_support::define_boolean(scope, config, "flip", flip);
    super::webcodecs_config_support::define_string(
        scope,
        config,
        "hardwareAcceleration",
        &hardware_acceleration,
    );
    super::webcodecs_config_support::define_number(scope, config, "rotation", rotation);
    let supported = !codec.is_empty()
        && coded_height.is_none_or(|value| value.is_finite() && value > 0.0)
        && coded_width.is_none_or(|value| value.is_finite() && value > 0.0)
        && matches!(
            hardware_acceleration.as_str(),
            "no-preference" | "prefer-hardware" | "prefer-software"
        )
        && matches!(rotation as i32, 0 | 90 | 180 | 270)
        && super::webcodecs_config_support::codec_supported(
            &crate::fingerprint::edge(scope).media.video_decoder_codecs,
            &codec,
        );
    super::webcodecs_config_support::resolve_support(scope, config, supported, result);
}
