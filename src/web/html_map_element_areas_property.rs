use super::html_map_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "areas", get_areas)
}

fn get_areas(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut areas = Vec::new();
    collect_areas(scope, a.this(), &mut areas);
    let collection = v8::Local::new(scope, &record.areas);
    let _ = super::html_collection::replace(scope, collection, areas);
    r.set(collection.into());
}
