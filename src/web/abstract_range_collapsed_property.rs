pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "collapsed", get_collapsed)
}

fn get_collapsed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::abstract_range::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let same = v8::Local::new(scope, &record.start_container)
        .strict_equals(v8::Local::new(scope, &record.end_container).into())
        && record.start_offset == record.end_offset;
    result.set(v8::Boolean::new(scope, same).into());
}
