pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    attribute: &str,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::element::attribute_value(scope, arguments.this(), attribute) {
        Some(value) => {
            if let Some(value) = v8::String::new(scope, &value) {
                result.set(value.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}

pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    attribute: &str,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0);
    if value.is_null_or_undefined() {
        super::element::remove_attribute_value(scope, arguments.this(), attribute);
    } else {
        let value = crate::webidl::value_to_string(scope, value);
        super::element::set_attribute_value(scope, arguments.this(), attribute.to_owned(), value);
    }
}

fn root<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let mut current = node;
    while let Some(parent) = super::node::parent(scope, current) {
        current = parent;
    }
    current
}

fn elements_for_ids<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
    ids: &str,
) -> Vec<v8::Local<'s, v8::Object>> {
    let root = root(scope, element);
    let candidates = if super::element::record(scope, root).is_some() {
        let mut values = vec![root];
        values.extend(super::dom_selector::descendants(scope, root));
        values
    } else {
        super::dom_selector::descendants(scope, root)
    };
    ids.split_ascii_whitespace()
        .filter_map(|id| {
            candidates.iter().copied().find(|candidate| {
                super::element::attribute_value(scope, *candidate, "id").as_deref() == Some(id)
            })
        })
        .collect()
}

pub(crate) fn get_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    property: &str,
    attribute: &str,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = super::element::cached_reflected_value(scope, arguments.this(), property) {
        result.set(v8::Local::new(scope, &value));
        return;
    }
    let value =
        super::element::attribute_value(scope, arguments.this(), attribute).and_then(|ids| {
            elements_for_ids(scope, arguments.this(), &ids)
                .into_iter()
                .next()
        });
    match value {
        Some(element) => result.set(element.into()),
        None => result.set(v8::null(scope).into()),
    }
}

pub(crate) fn set_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    property: &str,
    attribute: &str,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0);
    if value.is_null_or_undefined() {
        super::element::set_reflected_value(scope, arguments.this(), property, None);
        super::element::remove_attribute_value(scope, arguments.this(), attribute);
        return;
    }
    let Ok(element) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "The value is not an Element");
        return;
    };
    if super::element::record(scope, element).is_none() {
        crate::webidl::throw_type_error(scope, "The value is not an Element");
        return;
    }
    super::element::set_reflected_value(scope, arguments.this(), property, Some(element.into()));
    match super::element::attribute_value(scope, element, "id") {
        Some(id) if !id.is_empty() => {
            super::element::set_attribute_value(scope, arguments.this(), attribute.to_owned(), id);
        }
        _ => {
            super::element::remove_attribute_value(scope, arguments.this(), attribute);
        }
    }
}

pub(crate) fn get_elements(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    property: &str,
    attribute: &str,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = super::element::cached_reflected_value(scope, arguments.this(), property) {
        result.set(v8::Local::new(scope, &value));
        return;
    }
    let elements = super::element::attribute_value(scope, arguments.this(), attribute)
        .map(|ids| elements_for_ids(scope, arguments.this(), &ids))
        .unwrap_or_default();
    let array = v8::Array::new(scope, elements.len() as i32);
    for (index, element) in elements.into_iter().enumerate() {
        let _ = array.set_index(scope, index as u32, element.into());
    }
    super::element::set_reflected_value(scope, arguments.this(), property, Some(array.into()));
    result.set(array.into());
}

pub(crate) fn set_elements(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    property: &str,
    attribute: &str,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0);
    if value.is_null_or_undefined() {
        super::element::set_reflected_value(scope, arguments.this(), property, None);
        super::element::remove_attribute_value(scope, arguments.this(), attribute);
        return;
    }
    let Ok(sequence) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "The value is not a sequence of Elements");
        return;
    };
    let length = v8::String::new(scope, "length")
        .and_then(|key| sequence.get(scope, key.into()))
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let array = v8::Array::new(scope, length as i32);
    let mut ids = Vec::new();
    for index in 0..length {
        let Some(value) = sequence.get_index(scope, index) else {
            crate::webidl::throw_type_error(scope, "Cannot read the element sequence");
            return;
        };
        let Ok(element) = v8::Local::<v8::Object>::try_from(value) else {
            crate::webidl::throw_type_error(scope, "The sequence contains a non-Element");
            return;
        };
        if super::element::record(scope, element).is_none() {
            crate::webidl::throw_type_error(scope, "The sequence contains a non-Element");
            return;
        }
        let _ = array.set_index(scope, index, element.into());
        if let Some(id) = super::element::attribute_value(scope, element, "id")
            && !id.is_empty()
        {
            ids.push(id);
        }
    }
    super::element::set_reflected_value(scope, arguments.this(), property, Some(array.into()));
    if ids.is_empty() {
        super::element::remove_attribute_value(scope, arguments.this(), attribute);
    } else {
        super::element::set_attribute_value(
            scope,
            arguments.this(),
            attribute.to_owned(),
            ids.join(" "),
        );
    }
}
