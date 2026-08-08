use super::mutation_record::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "nextSibling", get_next_sibling)
}

fn get_next_sibling(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_optional_object(scope, arguments, result, |record| {
        record.next_sibling.as_ref()
    });
}
