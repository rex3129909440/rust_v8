use super::svg_m_path_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "href", get_href)
}

fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let href = scope
        .get_slot::<SvgMPathElementStore>()
        .and_then(|store| store.hrefs.get(&arguments.this().get_identity_hash().get()))
        .cloned();
    if let Some(href) = href {
        result.set(v8::Local::new(scope, &href).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
