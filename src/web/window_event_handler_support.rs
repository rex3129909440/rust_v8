pub(crate) fn handler_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    value.is_object().then(|| v8::Global::new(scope, value))
}

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    name: &str,
    getter: impl v8::MapFnTo<v8::FunctionCallback>,
    setter: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        getter,
    )?;
    let hidden_setter = crate::webidl::create_function(
        scope,
        &format!("set {name}"),
        1,
        v8::ConstructorBehavior::Throw,
        setter,
    )?;
    let name_value = crate::webidl::string(scope, name)?;
    let data = v8::Array::new(scope, 2);
    let _ = data.set_index(scope, 0, name_value.into());
    let _ = data.set_index(scope, 1, hidden_setter.into());
    let setter = crate::webidl::create_function_with_data(
        scope,
        &format!("set {name}"),
        1,
        v8::ConstructorBehavior::Throw,
        ordered_setter,
        data.into(),
    )?;
    crate::trace::relabel_native_function(scope, setter, &format!("window.set {name}"));
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, name)?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define window.{name}"))
    }
}

fn ordered_setter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(data) =
        v8::Local::<v8::Object>::try_from(crate::trace::native_callback_data(scope, &arguments))
    else {
        return;
    };
    let Some(name) = data.get_index(scope, 0) else {
        return;
    };
    let name = crate::webidl::value_to_string(scope, name);
    let Some(setter) = data.get_index(scope, 1) else {
        return;
    };
    let Ok(setter) = v8::Local::<v8::Function>::try_from(setter) else {
        return;
    };
    let value = arguments.get(0);
    super::event_target::set_attribute_handler(
        scope,
        arguments.this(),
        name.strip_prefix("on").unwrap_or(&name),
        value.is_object(),
    );
    let _ = setter.call(scope, arguments.this().into(), &[value]);
}

pub(crate) fn is_window(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> bool {
    let global = scope.get_current_context().global(scope);
    target.get_identity_hash().get() == global.get_identity_hash().get()
}

pub(crate) fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    handler: Option<v8::Global<v8::Value>>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(handler) = handler {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn invoke(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
) {
    let Some(handler) = handler else {
        return;
    };
    let local = v8::Local::new(scope, &handler);
    let Ok(function) = v8::Local::<v8::Function>::try_from(local) else {
        return;
    };
    let canceled = {
        v8::tc_scope!(let try_catch, scope);
        function
            .call(try_catch, target.into(), &[event.into()])
            .is_some_and(|value| value.is_boolean() && !value.boolean_value(try_catch))
    };
    if canceled {
        super::event::cancel(scope, event);
    }
}
