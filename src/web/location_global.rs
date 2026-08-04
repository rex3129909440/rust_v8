#[derive(Default)]
pub(crate) struct LocationGlobalStore {
    value: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LocationGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let location = super::location::create(scope, &crate::page_init::url(scope))?;
    let stored = v8::Global::new(scope, location);
    scope
        .get_slot_mut::<LocationGlobalStore>()
        .ok_or_else(|| "location global state was not prepared".to_owned())?
        .value = Some(stored);
    install_existing(scope)
}

pub(crate) fn install_existing(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get location",
        0,
        v8::ConstructorBehavior::Throw,
        get_location,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set location",
        1,
        v8::ConstructorBehavior::Throw,
        set_location,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(false);
    let key = crate::webidl::string(scope, "location")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.location".to_owned())
    }
}

pub(crate) fn value<'s>(scope: &v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::Object>> {
    if let Some(location) = super::html_i_frame_element::current_location(scope) {
        return Some(location);
    }
    let value = scope.get_slot::<LocationGlobalStore>()?.value.as_ref()?;
    Some(v8::Local::new(scope, value))
}

fn get_location(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(location) = value(scope) {
        result.set(location.into());
    }
}

fn set_location(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(location) = value(scope) else {
        return;
    };
    let Some(href_key) = v8::String::new(scope, "href") else {
        return;
    };
    let _ = location.set(scope, href_key.into(), arguments.get(0));
    let Some(href) = location
        .get(scope, href_key.into())
        .and_then(|value| value.to_string(scope))
    else {
        return;
    };
    let href = href.to_rust_string_lossy(scope);
    crate::page_init::navigate(scope, &href);
    if let Some(document) = super::document_global::value(scope) {
        super::document::set_string_value(scope, document, "URL", &href);
        super::document::set_string_value(scope, document, "documentURI", &href);
    }
}
