use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "unsuspendRedraw", 1, unsuspend_redraw)
}

fn unsuspend_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handle = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| {
        record.suspended_redraws.remove(&handle);
    });
}
