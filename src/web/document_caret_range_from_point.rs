pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "caretRangeFromPoint",
        0,
        caret_range_from_point,
    )
}

fn caret_range_from_point(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let x = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let y = arguments.get(1).number_value(scope).unwrap_or(f64::NAN);
    let Some(node) =
        super::document_method_support::hit_test_elements(scope, arguments.this(), x, y)
            .into_iter()
            .next()
    else {
        result.set(v8::null(scope).into());
        return;
    };
    match super::range::create(scope, arguments.this()) {
        Ok(range) => {
            let node = v8::Global::new(scope, node);
            super::abstract_range::update(scope, range, |record| {
                record.start_container = node.clone();
                record.end_container = node;
                record.start_offset = 0;
                record.end_offset = 0;
            });
            result.set(range.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
