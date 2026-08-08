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
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let animations = super::animation::for_document(scope, arguments.this());
    let array = v8::Array::new(scope, animations.len() as i32);
    for (index, animation) in animations.iter().enumerate() {
        let animation = v8::Local::new(scope, animation);
        let _ = array.set_index(scope, index as u32, animation.into());
    }
    result.set(array.into());
}
