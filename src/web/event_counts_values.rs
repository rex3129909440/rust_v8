use super::event_counts::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "values", 0, values)
}

fn values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(values) = snapshot(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, values.len() as i32);
    for (index, (_, value)) in values.iter().enumerate() {
        let _ = array.set_index(s, index as u32, v8::Number::new(s, *value as f64).into());
    }
    iterator(s, array, r)
}
