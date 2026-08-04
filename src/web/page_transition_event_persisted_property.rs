use super::page_transition_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "persisted", get_persisted)
}

fn get_persisted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(persisted) = scope
        .get_slot::<PageTransitionEventStore>()
        .and_then(|store| {
            store
                .persisted
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Boolean::new(scope, *persisted).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
