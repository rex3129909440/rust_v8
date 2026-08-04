pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "classList", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::element::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(list) = record.class_list {
        result.set(v8::Local::new(scope, &list).into());
        return;
    }
    let value =
        super::element::attribute_value(scope, arguments.this(), "class").unwrap_or_default();
    let Ok(list) = super::dom_token_list::create_bound(scope, &value, arguments.this(), "class")
    else {
        return;
    };
    super::element::cache_class_list(scope, arguments.this(), list);
    result.set(list.into());
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::element::set_attribute_value(scope, arguments.this(), "class".to_owned(), value.clone());
    if let Some(list) =
        super::element::record(scope, arguments.this()).and_then(|record| record.class_list)
    {
        let list = v8::Local::new(scope, &list);
        super::dom_token_list::set_string_value(scope, list, &value);
    }
}
