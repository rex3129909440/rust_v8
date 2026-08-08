use super::html_template_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "shadowRootCustomElementRegistry",
        get_shadow_root_custom_element_registry,
        set_shadow_root_custom_element_registry,
    )
}

fn get_shadow_root_custom_element_registry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.shadow_root_custom_element_registry {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_shadow_root_custom_element_registry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let value = value.map(|value| v8::Global::new(scope, value));
    if let Some(record) = scope
        .get_slot_mut::<HtmlTemplateElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.shadow_root_custom_element_registry = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
