use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgImageElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) preserve_aspect_ratio: v8::Global<v8::Object>,
    pub(crate) href: v8::Global<v8::Object>,
    pub(crate) decoding: String,
    pub(crate) cross_origin: Option<String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgImageElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGImageElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgImageElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGImageElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_image_element_x_property::define(scope, prototype)?;
    super::svg_image_element_y_property::define(scope, prototype)?;
    super::svg_image_element_width_property::define(scope, prototype)?;
    super::svg_image_element_height_property::define(scope, prototype)?;
    super::svg_image_element_preserve_aspect_ratio_property::define(scope, prototype)?;
    super::svg_image_element_decoding_property::define(scope, prototype)?;
    super::svg_image_element_cross_origin_property::define(scope, prototype)?;
    super::svg_image_element_href_property::define(scope, prototype)?;
    super::svg_image_element_decode::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_graphics_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgImageElementStore>()
        .ok_or_else(|| "SVGImageElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object =
        super::svg_graphics_element::create_with_constructor(scope, constructor, "image", owner)?;
    let x = super::svg_animated_length::create(scope, 0.0)?;
    let y = super::svg_animated_length::create(scope, 0.0)?;
    let width = super::svg_animated_length::create(scope, 0.0)?;
    let height = super::svg_animated_length::create(scope, 0.0)?;
    let preserve_aspect_ratio = super::svg_animated_preserve_aspect_ratio::create(
        scope,
        super::svg_preserve_aspect_ratio::PreserveAspectRatioValue {
            align: 6,
            meet_or_slice: 1,
        },
    )?;
    let href = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        width: v8::Global::new(scope, width),
        height: v8::Global::new(scope, height),
        preserve_aspect_ratio: v8::Global::new(scope, preserve_aspect_ratio),
        href: v8::Global::new(scope, href),
        decoding: "auto".to_owned(),
        cross_origin: None,
    };
    scope
        .get_slot_mut::<SvgImageElementStore>()
        .ok_or_else(|| "SVGImageElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGImageElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgImageElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(record) = scope
        .get_slot_mut::<SvgImageElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_object(
    scope: &v8::PinScope<'_, '_>,
    object: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, object).into());
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

pub(crate) fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.width, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.height, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn get_preserve_aspect_ratio(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        return_object(scope, &value.preserve_aspect_ratio, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        return_string(scope, &value.decoding, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if matches!(value.as_str(), "sync" | "async" | "auto") {
        value
    } else {
        "auto".to_owned()
    };
    update(scope, arguments.this(), |record| record.decoding = value);
}

pub(crate) fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        if let Some(value) = value.cross_origin {
            return_string(scope, &value, result);
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let normalized = if value.is_null() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, value);
        Some(if value == "use-credentials" {
            value
        } else {
            "anonymous".to_owned()
        })
    };
    update(scope, arguments.this(), |record| {
        record.cross_origin = normalized
    });
}

pub(crate) fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        return_object(scope, &value.href, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn decode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        crate::webidl::throw_type_error(scope, "Cannot create decode promise");
        return;
    };
    let promise = resolver.get_promise(scope);
    let _ = resolver.resolve(scope, v8::undefined(scope).into());
    result.set(promise.into());
}
