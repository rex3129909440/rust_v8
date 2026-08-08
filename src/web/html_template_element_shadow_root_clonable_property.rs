use super::html_template_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "shadowRootClonable",
        get_shadow_root_clonable,
        set_shadow_root_clonable,
    )
}

fn get_shadow_root_clonable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.shadow_root_clonable);
}

fn set_shadow_root_clonable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_bool(s, a, |x, v| x.shadow_root_clonable = v);
}
