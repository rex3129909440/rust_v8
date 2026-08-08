pub(crate) fn install<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let parent = super::worklet_global_scope::install(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "AudioWorkletGlobalScope",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    if crate::webidl::set_platform_prototype(scope, prototype, parent.into()) != Some(true) {
        return Err("cannot inherit AudioWorkletGlobalScope".to_owned());
    }
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::register_processor_global::install(scope, prototype)?;
    super::worklet_current_frame_global::install(scope, prototype)?;
    super::worklet_current_time_global::install(scope, prototype)?;
    super::worklet_sample_rate_global::install(scope, prototype)?;
    super::worklet_render_quantum_size_global::install(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_global(scope, "AudioWorkletGlobalScope", constructor.into())?;
    Ok(prototype)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'AudioWorkletGlobalScope': Illegal constructor",
    );
}
