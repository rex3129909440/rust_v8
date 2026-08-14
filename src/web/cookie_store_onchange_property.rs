use super::cookie_store::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "onchange", get_onchange, set_onchange)
}

fn get_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let handler = scope
        .get_slot::<CookieStoreStore>()
        .and_then(|store| {
            store
                .handlers
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    super::window_event_handler_support::return_handler(scope, handler, result);
}

fn set_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let identity = arguments.this().get_identity_hash().get();
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<CookieStoreStore>() {
        match handler {
            Some(handler) => {
                store.handlers.insert(identity, handler);
            }
            None => {
                store.handlers.remove(&identity);
            }
        }
    }
    let present = scope
        .get_slot::<CookieStoreStore>()
        .is_some_and(|store| store.handlers.contains_key(&identity));
    super::event_target::set_attribute_handler(scope, arguments.this(), "change", present);
}
