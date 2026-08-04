use super::event_counts::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)
}

fn for_each(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "The callback must be a function");
        return;
    };
    let Some(values) = snapshot(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    for (key, value) in values {
        let Some(key) = v8::String::new(s, &key) else {
            continue;
        };
        let _ = callback.call(
            s,
            a.get(1),
            &[
                v8::Number::new(s, value as f64).into(),
                key.into(),
                a.this().into(),
            ],
        );
    }
}
