pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let descriptors = v8::Object::new(scope);

    let getter = super::cross_origin_window_property_window::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "window",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_self::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "self",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_location::create_getter(scope, iframe_id)?;
    let setter = super::cross_origin_window_property_location::create_setter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "location",
        accessor_descriptor(scope, getter, Some(setter)),
    )?;
    let getter = super::cross_origin_window_property_closed::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "closed",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_frames::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "frames",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_length::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "length",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_top::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "top",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_opener::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "opener",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_parent::create_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "parent",
        accessor_descriptor(scope, getter, None),
    )?;

    let blur = super::cross_origin_window_property_blur::create(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "blur",
        data_descriptor(scope, blur.into(), false, false, true),
    )?;
    let close = super::cross_origin_window_property_close::create(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "close",
        data_descriptor(scope, close.into(), false, false, true),
    )?;
    let focus = super::cross_origin_window_property_focus::create(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "focus",
        data_descriptor(scope, focus.into(), false, false, true),
    )?;
    let post_message = super::cross_origin_window_property_post_message::create(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "postMessage",
        data_descriptor(scope, post_message.into(), false, false, true),
    )?;
    let undefined = v8::undefined(scope);
    define(
        scope,
        descriptors,
        "then",
        data_descriptor(scope, undefined.into(), false, false, true),
    )?;
    Ok(descriptors)
}

pub(crate) fn create_ancestor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let descriptors = v8::Object::new(scope);

    let getter =
        super::cross_origin_window_property_window::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "window",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter =
        super::cross_origin_window_property_self::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "self",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter =
        super::cross_origin_window_property_location::create_ancestor_getter(scope, iframe_id)?;
    let setter =
        super::cross_origin_window_property_location::create_ancestor_setter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "location",
        accessor_descriptor(scope, getter, Some(setter)),
    )?;
    let getter =
        super::cross_origin_window_property_closed::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "closed",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter =
        super::cross_origin_window_property_frames::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "frames",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter =
        super::cross_origin_window_property_length::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "length",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter = super::cross_origin_window_property_top::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "top",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter =
        super::cross_origin_window_property_opener::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "opener",
        accessor_descriptor(scope, getter, None),
    )?;
    let getter =
        super::cross_origin_window_property_parent::create_ancestor_getter(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "parent",
        accessor_descriptor(scope, getter, None),
    )?;

    let blur = super::cross_origin_window_property_blur::create(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "blur",
        data_descriptor(scope, blur.into(), false, false, true),
    )?;
    let close = super::cross_origin_window_property_close::create(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "close",
        data_descriptor(scope, close.into(), false, false, true),
    )?;
    let focus = super::cross_origin_window_property_focus::create(scope, iframe_id)?;
    define(
        scope,
        descriptors,
        "focus",
        data_descriptor(scope, focus.into(), false, false, true),
    )?;
    let post_message = super::cross_origin_window_property_post_message::create_ancestor(scope)?;
    define(
        scope,
        descriptors,
        "postMessage",
        data_descriptor(scope, post_message.into(), false, false, true),
    )?;
    let undefined = v8::undefined(scope);
    define(
        scope,
        descriptors,
        "then",
        data_descriptor(scope, undefined.into(), false, false, true),
    )?;
    Ok(descriptors)
}

fn define(
    scope: &v8::PinScope<'_, '_>,
    descriptors: v8::Local<'_, v8::Object>,
    name: &str,
    descriptor: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if descriptors.create_data_property(scope, key.into(), descriptor.into()) == Some(true) {
        Ok(())
    } else {
        Err(format!(
            "cannot cache cross-origin Window.{name} descriptor"
        ))
    }
}

fn accessor_descriptor<'s>(
    scope: &v8::PinScope<'s, '_>,
    getter: v8::Local<'s, v8::Function>,
    setter: Option<v8::Local<'s, v8::Function>>,
) -> v8::Local<'s, v8::Object> {
    let descriptor = v8::Object::new(scope);
    let get_key = v8::String::new(scope, "get").expect("short descriptor key");
    let set_key = v8::String::new(scope, "set").expect("short descriptor key");
    let enumerable_key = v8::String::new(scope, "enumerable").expect("short descriptor key");
    let configurable_key = v8::String::new(scope, "configurable").expect("short descriptor key");
    let _ = descriptor.create_data_property(scope, get_key.into(), getter.into());
    let setter = setter.map_or_else(|| v8::undefined(scope).into(), Into::into);
    let _ = descriptor.create_data_property(scope, set_key.into(), setter);
    let false_value = v8::Boolean::new(scope, false);
    let _ = descriptor.create_data_property(scope, enumerable_key.into(), false_value.into());
    let true_value = v8::Boolean::new(scope, true);
    let _ = descriptor.create_data_property(scope, configurable_key.into(), true_value.into());
    descriptor
}

pub(crate) fn data_descriptor<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) -> v8::Local<'s, v8::Object> {
    let descriptor = v8::Object::new(scope);
    let value_key = v8::String::new(scope, "value").expect("short descriptor key");
    let writable_key = v8::String::new(scope, "writable").expect("short descriptor key");
    let enumerable_key = v8::String::new(scope, "enumerable").expect("short descriptor key");
    let configurable_key = v8::String::new(scope, "configurable").expect("short descriptor key");
    let _ = descriptor.create_data_property(scope, value_key.into(), value);
    let writable = v8::Boolean::new(scope, writable);
    let _ = descriptor.create_data_property(scope, writable_key.into(), writable.into());
    let enumerable = v8::Boolean::new(scope, enumerable);
    let _ = descriptor.create_data_property(scope, enumerable_key.into(), enumerable.into());
    let configurable = v8::Boolean::new(scope, configurable);
    let _ = descriptor.create_data_property(scope, configurable_key.into(), configurable.into());
    descriptor
}
