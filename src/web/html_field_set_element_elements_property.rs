use super::html_field_set_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "elements", get_elements)
}

fn get_elements(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut items = Vec::new();
    collect_descendants(scope, arguments.this(), &mut items);
    let collection = v8::Local::new(scope, &record.elements);
    super::html_collection::replace(scope, collection, items);
    result.set(collection.into());
}
