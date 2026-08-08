use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "addTextTrack", 1, add_text_track)
}

fn add_text_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let kind = crate::webidl::value_to_string(scope, arguments.get(0));
    let label = if arguments.length() > 1 {
        crate::webidl::value_to_string(scope, arguments.get(1))
    } else {
        String::new()
    };
    let language = if arguments.length() > 2 {
        crate::webidl::value_to_string(scope, arguments.get(2))
    } else {
        String::new()
    };
    let Ok(track) = super::text_track::create(scope, kind, label, language, String::new()) else {
        return;
    };
    if let Some(list) = record.text_tracks {
        let list = v8::Local::new(scope, &list);
        let _ = super::text_track_list::append(scope, list, track);
    }
    result.set(track.into());
}
