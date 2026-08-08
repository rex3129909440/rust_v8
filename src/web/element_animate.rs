pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "animate", 1, animate)
}

fn animate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let global = scope.get_current_context().global(scope);
    let Some(effect_key) = v8::String::new(scope, "KeyframeEffect") else {
        return;
    };
    let Some(effect_value) = global.get(scope, effect_key.into()) else {
        return;
    };
    let Ok(effect_constructor) = v8::Local::<v8::Function>::try_from(effect_value) else {
        return;
    };
    let keyframes = arguments.get(0);
    let options = arguments.get(1);
    let Some(effect) =
        effect_constructor.new_instance(scope, &[arguments.this().into(), keyframes, options])
    else {
        return;
    };
    let Some(animation_key) = v8::String::new(scope, "Animation") else {
        return;
    };
    let Some(animation_value) = global.get(scope, animation_key.into()) else {
        return;
    };
    let Ok(animation_constructor) = v8::Local::<v8::Function>::try_from(animation_value) else {
        return;
    };
    let Some(animation) = animation_constructor.new_instance(scope, &[effect.into()]) else {
        return;
    };
    if let Some(play_key) = v8::String::new(scope, "play")
        && let Some(play) = animation.get(scope, play_key.into())
        && let Ok(play) = v8::Local::<v8::Function>::try_from(play)
    {
        let _ = play.call(scope, animation.into(), &[]);
    }
    result.set(animation.into());
}
