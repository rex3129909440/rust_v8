use super::html_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "sharedStorageWritable",
        get_shared_storage_writable,
        set_shared_storage_writable,
    )
}

fn get_shared_storage_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    get_reflected_boolean(scope, arguments, result, "sharedstoragewritable");
}

fn set_shared_storage_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_boolean(scope, arguments, "sharedstoragewritable");
}
