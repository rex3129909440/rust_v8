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
            "Failed to execute 'isConfigSupported' on 'VideoEncoder': Failed to read the 'codec' property from 'VideoEncoderConfig': Required member is undefined.",
            result,
        );
        return;
    };
    let Some(height) = super::webcodecs_config_support::number_member(scope, input, "height")
    else {
        super::webcodecs_config_support::reject_type_error(
            scope,
            "Failed to execute 'isConfigSupported' on 'VideoEncoder': Failed to read the 'height' property from 'VideoEncoderConfig': Required member is undefined.",
            result,
        );
        return;
    };
    let Some(width) = super::webcodecs_config_support::number_member(scope, input, "width") else {
        super::webcodecs_config_support::reject_type_error(
            scope,
            "Failed to execute 'isConfigSupported' on 'VideoEncoder': Failed to read the 'width' property from 'VideoEncoderConfig': Required member is undefined.",
            result,
        );
        return;
    };
    let alpha = super::webcodecs_config_support::string_member(scope, input, "alpha")
        .unwrap_or_else(|| "discard".to_owned());
    let bitrate = super::webcodecs_config_support::number_member(scope, input, "bitrate");
    let bitrate_mode = super::webcodecs_config_support::string_member(scope, input, "bitrateMode")
        .unwrap_or_else(|| "variable".to_owned());
    let content_hint = super::webcodecs_config_support::string_member(scope, input, "contentHint")
        .unwrap_or_default();
    let framerate = super::webcodecs_config_support::number_member(scope, input, "framerate");
    let hardware_acceleration =
        super::webcodecs_config_support::string_member(scope, input, "hardwareAcceleration")
            .unwrap_or_else(|| "no-preference".to_owned());
    let latency_mode = super::webcodecs_config_support::string_member(scope, input, "latencyMode")
        .unwrap_or_else(|| "quality".to_owned());
    let config = v8::Object::new(scope);
    super::webcodecs_config_support::define_string(scope, config, "alpha", &alpha);
    if let Some(value) = bitrate {
        super::webcodecs_config_support::define_number(scope, config, "bitrate", value);
    }
    super::webcodecs_config_support::define_string(scope, config, "bitrateMode", &bitrate_mode);
    super::webcodecs_config_support::define_string(scope, config, "codec", &codec);
    super::webcodecs_config_support::define_string(scope, config, "contentHint", &content_hint);
    if let Some(value) = framerate {
        super::webcodecs_config_support::define_number(scope, config, "framerate", value);
    }
    super::webcodecs_config_support::define_string(
        scope,
        config,
        "hardwareAcceleration",
        &hardware_acceleration,
    );
    super::webcodecs_config_support::define_number(scope, config, "height", height);
    super::webcodecs_config_support::define_string(scope, config, "latencyMode", &latency_mode);
    super::webcodecs_config_support::define_number(scope, config, "width", width);
    let supported = !codec.is_empty()
        && height.is_finite()
        && height > 0.0
        && width.is_finite()
        && width > 0.0
        && bitrate.is_none_or(|value| value.is_finite() && value > 0.0)
        && framerate.is_none_or(|value| value.is_finite() && value > 0.0)
        && matches!(alpha.as_str(), "discard" | "keep")
        && matches!(bitrate_mode.as_str(), "constant" | "variable")
        && matches!(
            hardware_acceleration.as_str(),
            "no-preference" | "prefer-hardware" | "prefer-software"
        )
        && matches!(latency_mode.as_str(), "quality" | "realtime")
        && super::webcodecs_config_support::codec_supported(
            &crate::fingerprint::edge(scope).media.video_encoder_codecs,
            &codec,
        );
    super::webcodecs_config_support::resolve_support(scope, config, supported, result);
}
