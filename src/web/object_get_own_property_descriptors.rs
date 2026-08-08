pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    crate::webidl::replace_intrinsic_method(
        scope,
        "Object",
        "getOwnPropertyDescriptors",
        1,
        get_own_property_descriptors,
    )
}

fn get_own_property_descriptors(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0))
        && super::html_i_frame_element::is_cross_origin_window_proxy(scope, object)
    {
        let descriptors = v8::Object::new(scope);
        let Some(values) =
            super::html_i_frame_element::cross_origin_window_index_values(scope, object)
        else {
            return;
        };
        for index in 0..values.length() {
            let Some(value) = values.get_index(scope, index) else {
                return;
            };
            let Some(key) = v8::String::new(scope, &index.to_string()) else {
                return;
            };
            let descriptor = data_descriptor(scope, value, false, true, true);
            let _ = descriptors.create_data_property(scope, key.into(), descriptor.into());
        }
        if !copy_named_descriptor(scope, descriptors, object, "window")
            || !copy_named_descriptor(scope, descriptors, object, "self")
            || !copy_named_descriptor(scope, descriptors, object, "location")
            || !copy_named_descriptor(scope, descriptors, object, "closed")
            || !copy_named_descriptor(scope, descriptors, object, "frames")
            || !copy_named_descriptor(scope, descriptors, object, "length")
            || !copy_named_descriptor(scope, descriptors, object, "top")
            || !copy_named_descriptor(scope, descriptors, object, "opener")
            || !copy_named_descriptor(scope, descriptors, object, "parent")
            || !copy_named_descriptor(scope, descriptors, object, "blur")
            || !copy_named_descriptor(scope, descriptors, object, "close")
            || !copy_named_descriptor(scope, descriptors, object, "focus")
            || !copy_named_descriptor(scope, descriptors, object, "postMessage")
            || !copy_named_descriptor(scope, descriptors, object, "then")
        {
            return;
        }
        let symbol_descriptor =
            data_descriptor(scope, v8::undefined(scope).into(), false, false, true);
        let _ = descriptors.create_data_property(
            scope,
            v8::Symbol::get_to_string_tag(scope).into(),
            symbol_descriptor.into(),
        );
        let symbol_descriptor =
            data_descriptor(scope, v8::undefined(scope).into(), false, false, true);
        let _ = descriptors.create_data_property(
            scope,
            v8::Symbol::get_has_instance(scope).into(),
            symbol_descriptor.into(),
        );
        let symbol_descriptor =
            data_descriptor(scope, v8::undefined(scope).into(), false, false, true);
        let _ = descriptors.create_data_property(
            scope,
            v8::Symbol::get_is_concat_spreadable(scope).into(),
            symbol_descriptor.into(),
        );
        result.set(descriptors.into());
        return;
    }
    let Ok(original) =
        v8::Local::<v8::Function>::try_from(crate::trace::native_callback_data(scope, &arguments))
    else {
        return;
    };
    if let Some(value) = original.call(scope, arguments.this().into(), &[arguments.get(0)]) {
        result.set(value);
    }
}

fn copy_named_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    output: v8::Local<'_, v8::Object>,
    window: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    let Some(key) = v8::String::new(scope, name) else {
        return false;
    };
    let Some(descriptor) = window.get_own_property_descriptor(scope, key.into()) else {
        return false;
    };
    output.create_data_property(scope, key.into(), descriptor) == Some(true)
}

fn data_descriptor<'s>(
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
