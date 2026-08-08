use super::html_data_list_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "options", get_options)
}

fn get_options(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let options = scope
        .get_slot::<HtmlDataListElementStore>()
        .and_then(|store| {
            store
                .options
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    let Some(options) = options else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let options = v8::Local::new(scope, &options);
    super::html_collection::refresh_live(scope, options);
    result.set(options.into());
}
