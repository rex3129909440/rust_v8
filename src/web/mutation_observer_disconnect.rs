use super::mutation_observer::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "disconnect", 0, disconnect)
}

fn disconnect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(observer) = scope
        .get_slot_mut::<MutationObserverStore>()
        .and_then(|store| store.observers.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    observer.observed_targets.clear();
    observer.pending.clear();
    observer.microtask_scheduled = false;
    observer.transient_observed_targets.clear();
}
