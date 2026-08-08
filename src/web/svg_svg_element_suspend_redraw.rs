use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "suspendRedraw", 1, suspend_redraw)
}

fn suspend_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let handle = current.next_redraw_handle;
    update(scope, arguments.this(), |record| {
        record.next_redraw_handle = record.next_redraw_handle.saturating_add(1);
        record.suspended_redraws.insert(handle);
    });
    result.set(v8::Integer::new_from_unsigned(scope, handle).into());
}
