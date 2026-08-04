use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getEnclosureList", 2, get_enclosure_list)
}

fn get_enclosure_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        return_descendants(scope, arguments.this(), result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
