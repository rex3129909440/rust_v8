use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "length", get_length, set_length)
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(
            v8::Integer::new_from_unsigned(scope, options_snapshot(scope, a.this()).len() as u32)
                .into(),
        )
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let requested = a.get(0).uint32_value(scope).unwrap_or(0) as usize;
    let current = options_snapshot(scope, a.this());
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if requested < current.len() {
        for o in current[requested..].iter().rev() {
            let _ = super::node::detach(scope, *o);
        }
    } else {
        for _ in current.len()..requested {
            if let Ok(o) = super::html_option_element::create(
                scope,
                String::new(),
                String::new(),
                false,
                false,
            ) {
                let index = super::node::children(scope, a.this()).len();
                let _ = super::node::insert_child(scope, a.this(), o, index);
            }
        }
    }
    refresh(scope, a.this())
}
