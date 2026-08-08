use super::performance_event_timing::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(base) = super::performance_entry::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = super::performance_entry::to_object(scope, &base);
    define_value(
        scope,
        output,
        "processingStart",
        v8::Number::new(scope, record.processing_start).into(),
    );
    define_value(
        scope,
        output,
        "processingEnd",
        v8::Number::new(scope, record.processing_end).into(),
    );
    define_value(
        scope,
        output,
        "cancelable",
        v8::Boolean::new(scope, record.cancelable).into(),
    );
    if let Some(target) = record.target {
        define_value(scope, output, "target", v8::Local::new(scope, &target));
    } else {
        define_value(scope, output, "target", v8::null(scope).into());
    }
    define_value(
        scope,
        output,
        "interactionId",
        v8::Integer::new(scope, record.interaction_id).into(),
    );
    result.set(output.into());
}
