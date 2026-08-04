use super::html_table_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "tFoot", get_t_foot, set_t_foot)
}

fn get_t_foot(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_optional(s, a, r, |x| &x.t_foot);
}

fn set_t_foot(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_special_child(
        scope,
        arguments.this(),
        arguments.get(0),
        SpecialChild::Foot,
    );
}
