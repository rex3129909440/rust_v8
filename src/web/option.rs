#[derive(Default)]
pub(crate) struct OptionFactory;

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OptionFactory);
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let html_option = super::html_option_element::ensure_constructor(scope)?;
    let html_option_prototype = crate::webidl::prototype(scope, html_option)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Option",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    if constructor.define_own_property(
        scope,
        prototype_key.into(),
        html_option_prototype.into(),
        v8::PropertyAttribute::READ_ONLY
            | v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE,
    ) != Some(true)
    {
        return Err("cannot attach Option.prototype".to_owned());
    }
    crate::webidl::define_global(scope, "Option", constructor.into())
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let text = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let value = if arguments.get(1).is_undefined() {
        super::html_option_element::normalize_option_text(&text)
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    let default_selected = arguments.get(2).boolean_value(scope);
    let selected = arguments.get(3).boolean_value(scope);
    match super::html_option_element::create(scope, text, value.clone(), default_selected, selected)
    {
        Ok(object) => {
            if arguments.length() > 1 && !arguments.get(1).is_undefined() {
                super::element::set_attribute_full(scope, object, "value".to_owned(), value, None);
            }
            if default_selected {
                super::element::set_attribute_full(
                    scope,
                    object,
                    "selected".to_owned(),
                    String::new(),
                    None,
                );
            }
            result.set(object.into())
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
