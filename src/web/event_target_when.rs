use super::event_target::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "when", 1, when)
}

fn when(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let promise = resolver.get_promise(scope);
    let resolver = v8::Global::new(scope, resolver);
    let target_id = target_record_id(scope, arguments.this());
    let Some(record) = scope
        .get_slot_mut::<EventTargetStore>()
        .and_then(|store| store.targets.get_mut(&target_id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.waiters.entry(event_type).or_default().push(resolver);
    result.set(promise.into());
}
