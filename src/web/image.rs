#[derive(Default)]
pub(crate) struct ImageFactory;

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ImageFactory);
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let html_image = super::html_image_element::ensure_constructor(scope)?;
    let html_image_prototype = crate::webidl::prototype(scope, html_image)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Image",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    if constructor.define_own_property(
        scope,
        prototype_key.into(),
        html_image_prototype.into(),
        v8::PropertyAttribute::READ_ONLY
            | v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE,
    ) != Some(true)
    {
        return Err("cannot attach Image.prototype".to_owned());
    }
    crate::webidl::define_global(scope, "Image", constructor.into())
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let width = if arguments.get(0).is_undefined() {
        0
    } else {
        arguments.get(0).uint32_value(scope).unwrap_or(0)
    };
    let height = if arguments.get(1).is_undefined() {
        0
    } else {
        arguments.get(1).uint32_value(scope).unwrap_or(0)
    };
    match super::html_image_element::create(scope, width, height) {
        Ok(object) => result.set(object.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
