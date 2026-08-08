use super::math_ml_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "focus", 0, focus)
}

pub(crate) fn focus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let valid = scope.get_slot::<MathMlElementStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&arguments.this().get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::html_element::clear_focus(scope);
    super::svg_element::clear_focus(scope);
    clear_focus(scope);
    if let Some(record) = scope
        .get_slot_mut::<MathMlElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.focused = true;
    }
    super::element::update_document_focus(scope, arguments.this(), true);
}
