#[derive(Default)]
pub(crate) struct AudioStore;

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioStore);
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let html_audio = super::html_audio_element::ensure_constructor(scope)?;
    let html_audio_prototype = crate::webidl::prototype(scope, html_audio)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Audio",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    if constructor.define_own_property(
        scope,
        prototype_key.into(),
        html_audio_prototype.into(),
        v8::PropertyAttribute::READ_ONLY
            | v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE,
    ) != Some(true)
    {
        return Err("cannot attach Audio.prototype".to_owned());
    }
    crate::webidl::define_global(scope, "Audio", constructor.into())
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let source = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    match super::html_audio_element::create(scope, source) {
        Ok(object) => result.set(object.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
