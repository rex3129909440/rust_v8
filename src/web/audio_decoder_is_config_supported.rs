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
            "Failed to execute 'isConfigSupported' on 'AudioDecoder': Failed to read the 'codec' property from 'AudioDecoderConfig': Required member is undefined.",
            result,
        );
        return;
    };
    let Some(number_of_channels) =
        super::webcodecs_config_support::number_member(scope, input, "numberOfChannels")
    else {
        super::webcodecs_config_support::reject_type_error(
            scope,
            "Failed to execute 'isConfigSupported' on 'AudioDecoder': Failed to read the 'numberOfChannels' property from 'AudioDecoderConfig': Required member is undefined.",
            result,
        );
        return;
    };
    let Some(sample_rate) =
        super::webcodecs_config_support::number_member(scope, input, "sampleRate")
    else {
        super::webcodecs_config_support::reject_type_error(
            scope,
            "Failed to execute 'isConfigSupported' on 'AudioDecoder': Failed to read the 'sampleRate' property from 'AudioDecoderConfig': Required member is undefined.",
            result,
        );
        return;
    };
    let config = v8::Object::new(scope);
    super::webcodecs_config_support::define_string(scope, config, "codec", &codec);
    super::webcodecs_config_support::define_number(
        scope,
        config,
        "numberOfChannels",
        number_of_channels,
    );
    super::webcodecs_config_support::define_number(scope, config, "sampleRate", sample_rate);
    let supported = !codec.is_empty()
        && number_of_channels.is_finite()
        && (1.0..=32.0).contains(&number_of_channels)
        && sample_rate.is_finite()
        && sample_rate > 0.0
        && super::webcodecs_config_support::codec_supported(
            &crate::fingerprint::edge(scope).media.audio_decoder_codecs,
            &codec,
        );
    super::webcodecs_config_support::resolve_support(scope, config, supported, result);
}
