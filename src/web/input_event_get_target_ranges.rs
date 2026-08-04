use super::input_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getTargetRanges", 0, get_target_ranges)
}

fn get_target_ranges(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, record.target_ranges.len() as i32);
    for (index, value) in record.target_ranges.iter().enumerate() {
        let _ = output.set_index(scope, index as u32, v8::Local::new(scope, value));
    }
    result.set(output.into());
}
