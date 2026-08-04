use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "offsetTop", get_zero)
}

pub(crate) fn get_zero(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let layout = super::element_layout::compute(scope, arguments.this());
        let parent_y = super::element_layout::offset_parent(scope, arguments.this())
            .map(|parent| super::element_layout::compute(scope, parent).y)
            .unwrap_or(0.0);
        result.set(
            v8::Integer::new(scope, super::element_layout::rounded(layout.y - parent_y)).into(),
        );
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
