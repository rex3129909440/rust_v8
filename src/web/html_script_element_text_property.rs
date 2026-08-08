use super::html_script_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "text", get_text, set_text)
}

fn get_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        if let Some(v) = v8::String::new(scope, &super::node::text_content(scope, a.this())) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    for child in super::node::children(scope, a.this()) {
        super::node::detach(scope, child);
    }
    if !v.is_empty()
        && let Ok(text) = super::text::create(scope, v)
    {
        let _ = super::node::insert_node(scope, a.this(), text, 0);
    }
}
