use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct StyleMediaGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
    objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StyleMediaGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let prototype = v8::Object::new(scope);
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_method(scope, prototype, "matchMedium", 0, match_medium)?;
    let tag_key = v8::Symbol::get_to_string_tag(scope);
    let tag_value = crate::webidl::string(scope, "StyleMedia")?;
    if prototype.define_own_property(
        scope,
        tag_key.into(),
        tag_value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define StyleMedia toStringTag".to_owned());
    }

    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create StyleMedia".to_owned());
    }
    let object_id = object.get_identity_hash().get();
    let stored = v8::Global::new(scope, object);
    let realm_id = crate::webidl::realm_id(scope);
    let store = scope
        .get_slot_mut::<StyleMediaGlobalStore>()
        .ok_or_else(|| "styleMedia state was not prepared".to_owned())?;
    store.objects.insert(object_id);
    store.values.insert(realm_id, stored);

    let getter = crate::webidl::create_function(
        scope,
        "get styleMedia",
        0,
        v8::ConstructorBehavior::Throw,
        get_style_media,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "styleMedia")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.styleMedia".to_owned())
    }
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<StyleMediaGlobalStore>()
        .is_some_and(|store| store.objects.contains(&object.get_identity_hash().get()))
}

fn get_style_media(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<StyleMediaGlobalStore>()
        .and_then(|store| store.values.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        result.set(v8::Local::new(scope, &value).into());
    }
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = v8::String::new(scope, "screen") {
        result.set(value.into());
    }
}

fn match_medium(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let query = crate::webidl::value_to_string(scope, arguments.get(0));
    let width = super::window_view_state::inner_width(scope);
    let height = super::window_view_state::inner_height(scope);
    result.set(v8::Boolean::new(scope, evaluate_query(&query, width, height)).into());
}

fn evaluate_query(query: &str, width: f64, height: f64) -> bool {
    let normalized = query.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "all" || normalized == "screen" {
        return true;
    }
    if normalized == "undefined" || normalized == "not all" || normalized == "print" {
        return false;
    }
    if normalized.contains("orientation: landscape") {
        return width >= height;
    }
    if normalized.contains("orientation: portrait") {
        return height > width;
    }
    if let Some(value) = pixels_after(&normalized, "min-width:") {
        return width >= value;
    }
    if let Some(value) = pixels_after(&normalized, "max-width:") {
        return width <= value;
    }
    if let Some(value) = pixels_after(&normalized, "min-height:") {
        return height >= value;
    }
    if let Some(value) = pixels_after(&normalized, "max-height:") {
        return height <= value;
    }
    false
}

fn pixels_after(query: &str, marker: &str) -> Option<f64> {
    let start = query.find(marker)? + marker.len();
    let tail = query[start..].trim_start();
    let end = tail.find("px")?;
    tail[..end].trim().parse().ok()
}
