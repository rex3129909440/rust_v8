use super::mutation_observer::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "takeRecords", 0, take_records)
}

fn take_records(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(records) = scope
        .get_slot_mut::<MutationObserverStore>()
        .and_then(|store| store.observers.get_mut(&id))
        .map(|observer| std::mem::take(&mut observer.pending))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(records_array(scope, &records).into());
}
