use super::html_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "height", get_height, set_height)
}

fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let layout = super::element_layout::compute(scope, arguments.this());
    if layout.rendered {
        result.set(
            v8::Integer::new(
                scope,
                super::element_layout::rounded(layout.content_height).max(0),
            )
            .into(),
        );
    } else {
        let height = display_dimensions(scope, arguments.this())
            .map(|dimensions| dimensions.1)
            .unwrap_or(0);
        result.set(v8::Integer::new_from_unsigned(scope, height).into());
    }
}

fn set_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_unsigned(scope, arguments, "height");
}
