use super::html_fenced_frame_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "config", get_config, set_config)
}

fn get_config(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.config {
            Some(config) => result.set(v8::Local::new(scope, &config).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_config(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let config = if value.is_null() {
        None
    } else {
        let Ok(config) = v8::Local::<v8::Object>::try_from(value) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to set 'config' on 'HTMLFencedFrameElement': value is not a FencedFrameConfig.",
            );
            return;
        };
        if !super::fenced_frame_config::is_instance(scope, config) {
            crate::webidl::throw_type_error(
                scope,
                "Failed to set 'config' on 'HTMLFencedFrameElement': value is not a FencedFrameConfig.",
            );
            return;
        }
        Some(v8::Global::new(scope, config))
    };
    update(scope, arguments.this(), |record| record.config = config);
}
