use super::html_table_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createCaption", 0, create_caption)
}

fn create_caption(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(caption) = snapshot.caption {
        result.set(v8::Local::new(scope, caption).into());
        return;
    }
    match super::html_table_caption_element::create(scope) {
        Ok(caption) => {
            let global = v8::Global::new(scope, caption);
            let _ = super::node::insert_child(scope, arguments.this(), caption, 0);
            update_special(scope, arguments.this(), SpecialChild::Caption, Some(global));
            result.set(caption.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
