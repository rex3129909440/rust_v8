use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "canPlayType", 1, can_play_type)
}

fn can_play_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let configured = &crate::fingerprint::edge(scope).media;
    let support = if crate::fingerprint_environment::media_type_matches(
        &configured.can_play_probably_types,
        &media_type,
    ) {
        "probably"
    } else if crate::fingerprint_environment::media_type_matches(
        &configured.can_play_maybe_types,
        &media_type,
    ) {
        "maybe"
    } else {
        ""
    };
    return_string(scope, &mut result, support);
}
