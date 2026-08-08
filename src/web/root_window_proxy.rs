pub(crate) fn global_template<'s>(
    scope: &v8::PinScope<'s, '_, ()>,
) -> v8::Local<'s, v8::ObjectTemplate> {
    let template = v8::ObjectTemplate::new(scope);
    let indexed = v8::IndexedPropertyHandlerConfiguration::new()
        .getter(indexed_getter)
        .setter(indexed_setter)
        .query(indexed_query)
        .deleter(indexed_deleter)
        .enumerator(indexed_enumerator)
        .definer(indexed_definer)
        .descriptor(indexed_descriptor);
    let indexed = match crate::trace::interceptor_data(scope) {
        Some(data) => indexed.data(data),
        None => indexed,
    };
    template.set_indexed_property_handler(indexed);
    let named = v8::NamedPropertyHandlerConfiguration::new()
        .getter(named_getter)
        .setter(named_setter)
        .query(named_query)
        .deleter(named_deleter)
        .enumerator(named_enumerator)
        .definer(named_definer)
        .descriptor(named_descriptor);
    let named = match crate::trace::interceptor_data(scope) {
        Some(data) => named.data(data),
        None => named,
    };
    template.set_named_property_handler(named);
    template
}

fn cross_origin_iframe_id(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
) -> Option<i32> {
    super::html_i_frame_element::cross_origin_ancestor_iframe_id_for_target(
        scope,
        arguments.holder(),
    )
}

fn named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_intercept(scope, &arguments, "get", None, key, None);
    let Some(iframe_id) = cross_origin_iframe_id(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    let Some(_guard) = HandlerGuard::enter() else {
        return v8::Intercepted::kNo;
    };
    if key.is_symbol() {
        if is_allowed_symbol(scope, key) {
            result.set(v8::undefined(scope).into());
        } else {
            super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        }
        return v8::Intercepted::kYes;
    }
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    match super::html_i_frame_element::cross_origin_ancestor_property_value_for_iframe(
        scope, iframe_id, &name,
    ) {
        Some(value) => result.set(value),
        None => result.set(v8::undefined(scope).into()),
    }
    v8::Intercepted::kYes
}

fn named_setter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_named_intercept(scope, &arguments, "set", None, key, Some(value));
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    if property_name(scope, key).as_deref() == Some("location") {
        super::cross_origin_ancestor_location::navigate(
            scope,
            crate::webidl::value_to_string(scope, value),
        );
        result.set_bool(true);
        return v8::Intercepted::kYes;
    }
    super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn named_query(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_named_intercept(scope, &arguments, "has", None, key, None);
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    if key.is_symbol() {
        if is_allowed_symbol(scope, key) {
            result.set_int32(1);
        } else {
            super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        }
        return v8::Intercepted::kYes;
    }
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if is_allowed_name(&name) {
        result.set_int32(1);
    } else {
        super::cross_origin_location::throw_security_error(scope, &name, "Window");
    }
    v8::Intercepted::kYes
}

fn named_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_named_intercept(scope, &arguments, "delete", None, key, None);
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_enumerate_intercept(scope, &arguments, None);
    if handler_is_active() {
        result.set(v8::Array::new(scope, 0));
        return;
    }
    if cross_origin_iframe_id(scope, &arguments).is_some()
        && let Some(keys) =
            super::html_i_frame_element::cross_origin_window_string_keys(scope, arguments.holder())
    {
        result.set(keys);
        return;
    }
    result.set(v8::Array::new(scope, 0));
}

fn named_definer(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    _: &v8::PropertyDescriptor,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_named_intercept(scope, &arguments, "defineProperty", None, key, None);
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn named_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_intercept(
        scope,
        &arguments,
        "getOwnPropertyDescriptor",
        None,
        key,
        None,
    );
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    let Some(iframe_id) = cross_origin_iframe_id(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if key.is_symbol() {
        if is_allowed_symbol(scope, key) {
            let undefined = v8::undefined(scope);
            let descriptor = super::cross_origin_window_descriptors::data_descriptor(
                scope,
                undefined.into(),
                false,
                false,
                true,
            );
            result.set(descriptor.into());
        } else {
            super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        }
        return v8::Intercepted::kYes;
    }
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if !is_allowed_name(&name) {
        super::cross_origin_location::throw_security_error(scope, &name, "Window");
        return v8::Intercepted::kYes;
    }
    if let Some(descriptor) =
        super::html_i_frame_element::cross_origin_ancestor_descriptor_for_iframe(
            scope, iframe_id, key,
        )
    {
        result.set(descriptor);
    }
    v8::Intercepted::kYes
}

fn indexed_getter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_intercept(scope, &arguments, "get", None, index, None);
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    let Some(window) = super::html_i_frame_element::indexed_window_for_target(
        scope,
        arguments.holder(),
        index as usize,
    ) else {
        return v8::Intercepted::kNo;
    };
    result.set(window.into());
    v8::Intercepted::kYes
}

fn indexed_setter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_indexed_intercept(scope, &arguments, "set", None, index, Some(value));
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn indexed_query(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_indexed_intercept(scope, &arguments, "has", None, index, None);
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    if super::html_i_frame_element::indexed_window_for_target(
        scope,
        arguments.holder(),
        index as usize,
    )
    .is_some()
    {
        result.set_int32(1);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn indexed_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_indexed_intercept(scope, &arguments, "delete", None, index, None);
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn indexed_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_enumerate_intercept(scope, &arguments, None);
    if handler_is_active() {
        result.set(v8::Array::new(scope, 0));
        return;
    }
    if cross_origin_iframe_id(scope, &arguments).is_some()
        && let Some(keys) =
            super::html_i_frame_element::cross_origin_window_index_keys(scope, arguments.holder())
    {
        result.set(keys);
        return;
    }
    result.set(v8::Array::new(scope, 0));
}

fn indexed_definer(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    _: &v8::PropertyDescriptor,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_indexed_intercept(scope, &arguments, "defineProperty", None, index, None);
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn indexed_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_intercept(
        scope,
        &arguments,
        "getOwnPropertyDescriptor",
        None,
        index,
        None,
    );
    if handler_is_active() {
        return v8::Intercepted::kNo;
    }
    if cross_origin_iframe_id(scope, &arguments).is_none() {
        return v8::Intercepted::kNo;
    }
    let Some(window) = super::html_i_frame_element::indexed_window_for_target(
        scope,
        arguments.holder(),
        index as usize,
    ) else {
        return v8::Intercepted::kNo;
    };
    let descriptor = super::cross_origin_window_descriptors::data_descriptor(
        scope,
        window.into(),
        false,
        true,
        true,
    );
    result.set(descriptor.into());
    v8::Intercepted::kYes
}

fn property_name(scope: &v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    key.to_string(scope)
        .map(|name| name.to_rust_string_lossy(scope))
}

fn is_allowed_name(name: &str) -> bool {
    matches!(
        name,
        "window"
            | "self"
            | "location"
            | "closed"
            | "frames"
            | "length"
            | "top"
            | "opener"
            | "parent"
            | "blur"
            | "close"
            | "focus"
            | "postMessage"
            | "then"
    )
}

fn is_allowed_symbol(scope: &v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> bool {
    key.strict_equals(v8::Symbol::get_to_string_tag(scope).into())
        || key.strict_equals(v8::Symbol::get_has_instance(scope).into())
        || key.strict_equals(v8::Symbol::get_is_concat_spreadable(scope).into())
}
use std::cell::Cell;

thread_local! {
    static HANDLER_DEPTH: Cell<u32> = const { Cell::new(0) };
}

struct HandlerGuard;

impl HandlerGuard {
    fn enter() -> Option<Self> {
        HANDLER_DEPTH.with(|depth| {
            if depth.get() != 0 {
                None
            } else {
                depth.set(1);
                Some(Self)
            }
        })
    }
}

impl Drop for HandlerGuard {
    fn drop(&mut self) {
        HANDLER_DEPTH.with(|depth| depth.set(0));
    }
}

fn handler_is_active() -> bool {
    HANDLER_DEPTH.with(|depth| depth.get() != 0)
}
