use super::html_table_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createTHead", 0, create_t_head)
}

fn create_t_head(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let table = v8::Global::new(scope, arguments.this());
    if let Some(section) = snapshot.t_head {
        result.set(v8::Local::new(scope, section).into());
    } else if let Some(section) = create_section(scope, &table, "THEAD", Some(SpecialChild::Head)) {
        result.set(v8::Local::new(scope, section).into());
    }
}
