pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getAnimations", 0, get_animations)
}

fn get_animations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let subtree = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|options| {
            v8::String::new(scope, "subtree").and_then(|key| options.get(scope, key.into()))
        })
        .is_some_and(|value| value.boolean_value(scope));
    let animations = super::animation::for_element(scope, arguments.this(), subtree);
    let array = v8::Array::new(scope, animations.len() as i32);
    for (index, animation) in animations.iter().enumerate() {
        let animation = v8::Local::new(scope, animation);
        let _ = array.set_index(scope, index as u32, animation.into());
    }
    result.set(array.into());
}
