use super::storage_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "initStorageEvent", 1, init_storage_event)
}

fn init_storage_event(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let key = if a.get(4).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(4)))
    };
    let old_value = if a.get(5).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(5)))
    };
    let new_value = if a.get(6).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(6)))
    };
    let url = crate::webidl::value_to_string(scope, a.get(7));
    let Some(v) = scope
        .get_slot_mut::<StorageEventStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    v.key = key;
    v.old_value = old_value;
    v.new_value = new_value;
    v.url = url;
}
