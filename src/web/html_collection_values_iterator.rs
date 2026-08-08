pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "values", 0, v8::ConstructorBehavior::Throw, values)?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, function, &format!("{owner}.values"));
    }
    let key = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define HTMLCollection iterator".to_owned())
    }
}
fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    super::html_collection::refresh_live(scope, arguments.this());
    let Some(record) = super::html_collection::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.len() as i32);
    for (index, item) in record.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, item).into());
    }
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(method) = array.get(scope, key.into()) else {
        return;
    };
    let Ok(method) = v8::Local::<v8::Function>::try_from(method) else {
        return;
    };
    if let Some(iterator) = method.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}
