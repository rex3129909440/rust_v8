use super::svg_a_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "download", get_download, set_download)
}

fn get_download(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.download, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_download(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !update(s, a.this(), |v| v.download = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
