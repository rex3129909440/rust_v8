thread_local! {
    static PENDING_CONVERSION_TYPE_ERROR: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

#[derive(Default)]
pub(crate) struct RealmConstructor {
    values: std::collections::HashMap<i32, v8::Global<v8::Function>>,
}

impl RealmConstructor {
    pub(crate) fn get(&self, realm_id: i32) -> Option<&v8::Global<v8::Function>> {
        self.values.get(&realm_id)
    }

    pub(crate) fn insert(&mut self, realm_id: i32, constructor: v8::Global<v8::Function>) {
        self.values.insert(realm_id, constructor);
    }

    pub(crate) fn remove(&mut self, realm_id: i32) {
        self.values.remove(&realm_id);
    }
}

pub(crate) fn string<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: &str,
) -> Result<v8::Local<'s, v8::String>, String> {
    v8::String::new(scope, value).ok_or_else(|| "string exceeds V8 limits".to_owned())
}

pub(crate) fn value_to_string(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> String {
    if value.is_symbol() {
        let message = crate::trace::current_constructor_name()
            .map(|name| {
                format!("Failed to construct '{name}': Cannot convert a Symbol value to a string")
            })
            .unwrap_or_else(|| "Cannot convert a Symbol value to a string".to_owned());
        PENDING_CONVERSION_TYPE_ERROR.with(|pending| {
            *pending.borrow_mut() = Some(message.clone());
        });
        return "Symbol(audit)".to_owned();
    }
    value
        .to_string(scope)
        .map(|text| text.to_rust_string_lossy(scope))
        .unwrap_or_default()
}

/// Performs the Web IDL DOMString conversion at the observable point where an
/// API consumes its argument. Unlike `value_to_string`, this form does not
/// defer a Symbol conversion failure until the native callback returns, so a
/// constructor cannot inspect later arguments after the conversion has
/// already failed.
pub(crate) fn dom_string(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<String> {
    if value.is_symbol() {
        let message = crate::trace::current_constructor_name()
            .map(|name| {
                format!("Failed to construct '{name}': Cannot convert a Symbol value to a string")
            })
            .unwrap_or_else(|| "Cannot convert a Symbol value to a string".to_owned());
        let message = v8::String::new(scope, &message)?;
        scope.throw_exception(v8::Exception::type_error(scope, message));
        return None;
    }
    value
        .to_string(scope)
        .map(|text| text.to_rust_string_lossy(scope))
}

pub(crate) fn dom_string_with_context(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    context: &str,
) -> Option<String> {
    if value.is_symbol() {
        let message = v8::String::new(
            scope,
            &format!("{context}: Cannot convert a Symbol value to a string"),
        )?;
        scope.throw_exception(v8::Exception::type_error(scope, message));
        return None;
    }
    dom_string(scope, value)
}

/// Returns the exact V8/Web IDL error for primitive values that cannot take
/// part in a Number conversion. Callers retain their own range and finiteness
/// rules after this primitive conversion step.
pub(crate) fn number_conversion_error(value: v8::Local<'_, v8::Value>) -> Option<String> {
    let kind = if value.is_symbol() {
        "Symbol"
    } else if value.is_big_int() {
        "BigInt"
    } else {
        return None;
    };
    Some(
        crate::trace::current_constructor_name()
            .map(|name| {
                format!("Failed to construct '{name}': Cannot convert a {kind} value to a number")
            })
            .unwrap_or_else(|| format!("Cannot convert a {kind} value to a number")),
    )
}

pub(crate) fn clear_pending_conversion_type_error() {
    PENDING_CONVERSION_TYPE_ERROR.with(|pending| pending.borrow_mut().take());
}

pub(crate) fn take_pending_conversion_type_error() -> Option<String> {
    PENDING_CONVERSION_TYPE_ERROR.with(|pending| pending.borrow_mut().take())
}

/// Converts a Web IDL `sequence<T>` by using the JavaScript iterator protocol.
/// A sequence is not array-like: observable conversion must start by reading
/// `@@iterator`, and must not inspect `length` or indexed properties directly.
pub(crate) fn sequence_values(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Vec<v8::Global<v8::Value>>, String> {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return Err("The provided value cannot be converted to a sequence".to_owned());
    };
    let iterator_key = v8::Symbol::get_iterator(scope);
    let Some(iterator_method) = object.get(scope, iterator_key.into()) else {
        return Err("The provided value cannot be converted to a sequence".to_owned());
    };
    let Ok(iterator_method) = v8::Local::<v8::Function>::try_from(iterator_method) else {
        return Err("The provided value is not iterable".to_owned());
    };
    let Some(iterator_value) = iterator_method.call(scope, object.into(), &[]) else {
        return Err("The sequence iterator could not be created".to_owned());
    };
    let Ok(iterator) = v8::Local::<v8::Object>::try_from(iterator_value) else {
        return Err("The sequence iterator is not an object".to_owned());
    };
    let next_key = v8::String::new(scope, "next")
        .ok_or_else(|| "The sequence iterator is invalid".to_owned())?;
    let Some(next) = iterator
        .get(scope, next_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return Err("The sequence iterator has no callable next method".to_owned());
    };
    let done_key = v8::String::new(scope, "done")
        .ok_or_else(|| "The sequence iterator is invalid".to_owned())?;
    let value_key = v8::String::new(scope, "value")
        .ok_or_else(|| "The sequence iterator is invalid".to_owned())?;
    let mut values = Vec::new();
    loop {
        let Some(step_value) = next.call(scope, iterator.into(), &[]) else {
            return Err("The sequence iterator failed".to_owned());
        };
        let Ok(step) = v8::Local::<v8::Object>::try_from(step_value) else {
            return Err("The sequence iterator result is not an object".to_owned());
        };
        if step
            .get(scope, done_key.into())
            .is_some_and(|value| value.boolean_value(scope))
        {
            break;
        }
        let Some(item) = step.get(scope, value_key.into()) else {
            return Err("The sequence iterator result has no value".to_owned());
        };
        values.push(v8::Global::new(scope, item));
        if values.len() >= 65_536 {
            return Err("The sequence is too large".to_owned());
        }
    }
    Ok(values)
}

pub(crate) fn realm_id(scope: &v8::PinScope<'_, '_>) -> i32 {
    let global = scope.get_current_context().global(scope);
    let object = v8::String::new(scope, "Object")
        .and_then(|key| global.get(scope, key.into()))
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok());
    object
        .map(|object| object.get_identity_hash().get())
        .unwrap_or_else(|| global.get_identity_hash().get())
}

pub(crate) fn create_function<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    length: i32,
    constructor_behavior: v8::ConstructorBehavior,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let kind = callback_kind(name, &constructor_behavior);
    crate::trace::create_native_function(
        scope,
        name,
        length,
        constructor_behavior,
        callback.map_fn_to(),
        kind,
    )
}

pub(crate) fn create_function_with_data<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    length: i32,
    constructor_behavior: v8::ConstructorBehavior,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
    data: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let kind = callback_kind(name, &constructor_behavior);
    crate::trace::create_native_function_with_data(
        scope,
        name,
        length,
        constructor_behavior,
        callback.map_fn_to(),
        kind,
        data,
    )
}

fn callback_kind(
    name: &str,
    constructor_behavior: &v8::ConstructorBehavior,
) -> crate::trace::NativeCallbackKind {
    match constructor_behavior {
        v8::ConstructorBehavior::Allow => crate::trace::NativeCallbackKind::Constructor,
        v8::ConstructorBehavior::Throw if name.starts_with("get ") => {
            crate::trace::NativeCallbackKind::Getter
        }
        v8::ConstructorBehavior::Throw if name.starts_with("set ") => {
            crate::trace::NativeCallbackKind::Setter
        }
        v8::ConstructorBehavior::Throw => crate::trace::NativeCallbackKind::Function,
    }
}

pub(crate) fn define_global(
    scope: &mut v8::PinScope<'_, '_>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let context = scope.get_current_context();
    let global = context.global(scope);
    crate::trace::label_native_value_once(scope, global.into(), "window");
    crate::trace::label_native_value_once(scope, value, name);
    if let Ok(function) = v8::Local::<v8::Function>::try_from(value) {
        crate::trace::relabel_native_function(scope, function, name);
    }
    let key = string(scope, name)?;
    match global.define_own_property(scope, key.into(), value, v8::PropertyAttribute::DONT_ENUM) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define window.{name}")),
    }
}

pub(crate) fn define_method(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
    length: i32,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let method = create_function(
        scope,
        name,
        length,
        v8::ConstructorBehavior::Throw,
        callback,
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, method, &format!("{owner}.{name}"));
    }
    let key = string(scope, name)?;
    match prototype.define_own_property(
        scope,
        key.into(),
        method.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define prototype method {name}")),
    }
}

pub(crate) fn replace_intrinsic_method(
    scope: &mut v8::PinScope<'_, '_>,
    intrinsic_name: &str,
    method_name: &str,
    length: i32,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let intrinsic_key = string(scope, intrinsic_name)?;
    let intrinsic = global
        .get(scope, intrinsic_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| format!("{intrinsic_name} intrinsic is unavailable"))?;
    let method_key = string(scope, method_name)?;
    let original = intrinsic
        .get(scope, method_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| format!("{intrinsic_name}.{method_name} is unavailable"))?;
    let replacement = create_function_with_data(
        scope,
        method_name,
        length,
        v8::ConstructorBehavior::Throw,
        callback,
        original.into(),
    )?;
    crate::trace::relabel_native_function(
        scope,
        replacement,
        &format!("{intrinsic_name}.{method_name}"),
    );
    if intrinsic.define_own_property(
        scope,
        method_key.into(),
        replacement.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!(
            "cannot replace native {intrinsic_name}.{method_name}"
        ))
    }
}

pub(crate) fn define_accessor(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
    getter: impl v8::MapFnTo<v8::FunctionCallback>,
    setter: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let getter_name = format!("get {name}");
    let setter_name = format!("set {name}");
    let getter = create_function(
        scope,
        &getter_name,
        0,
        v8::ConstructorBehavior::Throw,
        getter,
    )?;
    let setter = create_function(
        scope,
        &setter_name,
        1,
        v8::ConstructorBehavior::Throw,
        setter,
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, getter, &format!("{owner}.{getter_name}"));
        crate::trace::relabel_native_function(scope, setter, &format!("{owner}.{setter_name}"));
    }
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = string(scope, name)?;
    match prototype.define_property(scope, key.into(), &descriptor) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define prototype accessor {name}")),
    }
}

pub(crate) fn define_accessor_with_data(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
    getter: impl v8::MapFnTo<v8::FunctionCallback>,
    setter: impl v8::MapFnTo<v8::FunctionCallback>,
    data: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let getter_name = format!("get {name}");
    let setter_name = format!("set {name}");
    let getter = create_function_with_data(
        scope,
        &getter_name,
        0,
        v8::ConstructorBehavior::Throw,
        getter,
        data,
    )?;
    let setter = create_function_with_data(
        scope,
        &setter_name,
        1,
        v8::ConstructorBehavior::Throw,
        setter,
        data,
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, getter, &format!("{owner}.{getter_name}"));
        crate::trace::relabel_native_function(scope, setter, &format!("{owner}.{setter_name}"));
    }
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = string(scope, name)?;
    match prototype.define_property(scope, key.into(), &descriptor) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define prototype accessor {name}")),
    }
}

pub(crate) fn define_readonly_accessor(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
    getter: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let getter_name = format!("get {name}");
    let getter = create_function(
        scope,
        &getter_name,
        0,
        v8::ConstructorBehavior::Throw,
        getter,
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, getter, &format!("{owner}.{getter_name}"));
    }
    let setter = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = string(scope, name)?;
    match prototype.define_property(scope, key.into(), &descriptor) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define prototype accessor {name}")),
    }
}

pub(crate) fn define_constant(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: i32,
) -> Result<(), String> {
    let key = string(scope, name)?;
    let value = v8::Integer::new(scope, value);
    match object.define_own_property(
        scope,
        key.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
    ) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define constant {name}")),
    }
}

pub(crate) fn prototype<'s>(
    scope: &v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let key = string(scope, "prototype")?;
    let value = constructor
        .get(scope, key.into())
        .ok_or_else(|| "constructor has no prototype".to_owned())?;
    v8::Local::<v8::Object>::try_from(value)
        .map_err(|_| "constructor prototype is not an object".to_owned())
}

pub(crate) fn reset_constructor_order(
    scope: &v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let key = string(scope, "constructor")?;
    match prototype.delete(scope, key.into()) {
        Some(true) => Ok(()),
        _ => Err("cannot reorder prototype constructor".to_owned()),
    }
}

pub(crate) fn finish_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    constructor: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    let key = string(scope, "constructor")?;
    if prototype.define_own_property(
        scope,
        key.into(),
        constructor.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define prototype constructor".to_owned());
    }
    let name_key = string(scope, "name")?;
    let interface_name = constructor
        .get(scope, name_key.into())
        .ok_or_else(|| "constructor has no name".to_owned())?;
    let to_string_tag = v8::Symbol::get_to_string_tag(scope);
    if prototype.define_own_property(
        scope,
        to_string_tag.into(),
        interface_name,
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define interface toStringTag".to_owned());
    }
    let interface_name_text = value_to_string(scope, interface_name);
    crate::web::structured_clone::register_platform_prototype(
        scope,
        prototype,
        &interface_name_text,
    );
    let prototype_key = string(scope, "prototype")?;
    let prototype_value: v8::Local<v8::Value> = prototype.into();
    match constructor.define_own_property(
        scope,
        prototype_key.into(),
        prototype_value,
        v8::PropertyAttribute::READ_ONLY
            | v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot finalize constructor prototype".to_owned()),
    }
}

pub(crate) fn lock_constructor_prototype(
    scope: &mut v8::PinScope<'_, '_>,
    constructor: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    let prototype_key = string(scope, "prototype")?;
    let prototype_value = constructor
        .get(scope, prototype_key.into())
        .ok_or_else(|| "constructor has no prototype".to_owned())?;
    match constructor.define_own_property(
        scope,
        prototype_key.into(),
        prototype_value,
        v8::PropertyAttribute::READ_ONLY
            | v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot lock constructor prototype".to_owned()),
    }
}

pub(crate) fn set_platform_prototype(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    prototype: v8::Local<'_, v8::Value>,
) -> Option<bool> {
    let updated = object.set_prototype(scope, prototype);
    if updated == Some(true) {
        crate::web::structured_clone::register_platform_object_from_prototype(
            scope, object, prototype,
        );
    }
    updated
}

pub(crate) fn define_to_string_tag(
    scope: &v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    interface_name: &str,
) -> Result<(), String> {
    let tag = v8::Symbol::get_to_string_tag(scope);
    let value = string(scope, interface_name)?;
    if prototype.define_own_property(
        scope,
        tag.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define {interface_name} toStringTag"))
    }
}

pub(crate) fn new_unscopables<'s>(
    scope: &v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = v8::Object::new(scope);
    let null = v8::null(scope);
    if object.set_prototype(scope, null.into()) == Some(true) {
        Ok(object)
    } else {
        Err("cannot create unscopables object".to_owned())
    }
}

pub(crate) fn define_unscopable(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let key = string(scope, name)?;
    let value = v8::Boolean::new(scope, true);
    if object.define_own_property(scope, key.into(), value.into(), v8::PropertyAttribute::NONE)
        == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define unscopable {name}"))
    }
}

pub(crate) fn attach_unscopables(
    scope: &v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let key = v8::Symbol::get_unscopables(scope);
    if prototype.define_own_property(
        scope,
        key.into(),
        object.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot attach unscopables object".to_owned())
    }
}

pub(crate) fn define_iterator_alias(
    scope: &v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    method_name: &str,
) -> Result<(), String> {
    let method_key = string(scope, method_name)?;
    let method = prototype
        .get(scope, method_key.into())
        .ok_or_else(|| format!("{method_name} method is missing"))?;
    let iterator = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator.into(),
        method,
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define iterator alias for {method_name}"))
    }
}

pub(crate) fn define_async_iterator_alias(
    scope: &v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    method_name: &str,
) -> Result<(), String> {
    let method_key = string(scope, method_name)?;
    let method = prototype
        .get(scope, method_key.into())
        .ok_or_else(|| format!("{method_name} method is missing"))?;
    let iterator = v8::Symbol::get_async_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator.into(),
        method,
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!(
            "cannot define async iterator alias for {method_name}"
        ))
    }
}

pub(crate) fn define_indexed_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let iterator = create_function(
        scope,
        "values",
        0,
        v8::ConstructorBehavior::Throw,
        indexed_values,
    )?;
    let iterator_key = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator_key.into(),
        iterator.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define indexed iterator".to_owned())
    }
}

pub(crate) fn move_iterator_to_end(
    scope: &v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let iterator_key = v8::Symbol::get_iterator(scope);
    let iterator = prototype
        .get(scope, iterator_key.into())
        .ok_or_else(|| "iterator is missing".to_owned())?;
    if prototype.delete(scope, iterator_key.into()) != Some(true) {
        return Err("cannot reorder iterator".to_owned());
    }
    if prototype.define_own_property(
        scope,
        iterator_key.into(),
        iterator,
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot restore iterator".to_owned())
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ArrayLikeIteratorKind {
    Keys,
    Values,
    Entries,
}

pub(crate) fn return_array_like_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    kind: ArrayLikeIteratorKind,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(length_key) = v8::String::new(scope, "length") else {
        return;
    };
    let length = object
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let values = v8::Array::new(scope, length as i32);
    for index in 0..length {
        let direct = object
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let value: v8::Local<v8::Value> = match kind {
            ArrayLikeIteratorKind::Keys => v8::Integer::new_from_unsigned(scope, index).into(),
            ArrayLikeIteratorKind::Values => direct,
            ArrayLikeIteratorKind::Entries => {
                let pair = v8::Array::new(scope, 2);
                let _ = pair.set_index(
                    scope,
                    0,
                    v8::Integer::new_from_unsigned(scope, index).into(),
                );
                let _ = pair.set_index(scope, 1, direct);
                pair.into()
            }
        };
        let _ = values.set_index(scope, index, value);
    }
    let iterator_key = v8::Symbol::get_iterator(scope);
    let Some(iterator) = values.get(scope, iterator_key.into()) else {
        return;
    };
    let Ok(iterator) = v8::Local::<v8::Function>::try_from(iterator) else {
        return;
    };
    if let Some(value) = iterator.call(scope, values.into(), &[]) {
        result.set(value);
    }
}

pub(crate) fn array_like_for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        let value = arguments.get(0);
        let description = if value.is_null() {
            "object null".to_owned()
        } else if value.is_undefined() {
            "undefined".to_owned()
        } else {
            format!(
                "{} {}",
                value.type_of(scope).to_rust_string_lossy(scope),
                value_to_string(scope, value)
            )
        };
        throw_type_error(scope, &format!("{description} is not a function"));
        return;
    };
    let Some(length_key) = v8::String::new(scope, "length") else {
        return;
    };
    let length = arguments
        .this()
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let receiver = arguments.get(1);
    for index in 0..length {
        let value = arguments
            .this()
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let key = v8::Integer::new_from_unsigned(scope, index);
        if callback
            .call(
                scope,
                receiver,
                &[value, key.into(), arguments.this().into()],
            )
            .is_none()
        {
            return;
        }
    }
}

fn indexed_values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_array_like_iterator(
        scope,
        arguments.this(),
        ArrayLikeIteratorKind::Values,
        result,
    )
}

pub(crate) fn inherit(
    scope: &v8::PinScope<'_, '_>,
    constructor: v8::Local<'_, v8::Function>,
    parent: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    let child_prototype = prototype(scope, constructor)?;
    let parent_prototype = prototype(scope, parent)?;
    if child_prototype.set_prototype(scope, parent_prototype.into()) != Some(true) {
        return Err("cannot set interface prototype inheritance".to_owned());
    }
    let constructor_object: v8::Local<v8::Object> = constructor.into();
    if constructor_object.set_prototype(scope, parent.into()) != Some(true) {
        return Err("cannot set constructor inheritance".to_owned());
    }
    Ok(())
}

pub(crate) fn throw_type_error(scope: &v8::PinScope<'_, '_>, message: &str) {
    let expanded;
    let pending_conversion = PENDING_CONVERSION_TYPE_ERROR.with(|pending| pending.borrow().clone());
    let message = if let Some(message) = pending_conversion.as_deref() {
        message
    } else if let Some((name, required, present)) =
        crate::trace::current_constructor_missing_arguments()
    {
        let noun = if required == 1 {
            "argument"
        } else {
            "arguments"
        };
        expanded = Some(format!(
            "Failed to construct '{name}': {required} {noun} required, but only {present} present."
        ));
        expanded.as_deref().unwrap_or(message)
    } else if message == "Illegal constructor" {
        expanded = crate::trace::current_constructor_name()
            .map(|name| format!("Failed to construct '{name}': Illegal constructor"));
        expanded.as_deref().unwrap_or(message)
    } else {
        message
    };
    if let Some(message) = v8::String::new(scope, message) {
        let exception = v8::Exception::type_error(scope, message);
        scope.throw_exception(exception);
    }
}

pub(crate) fn rejected_type_error_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    message: &str,
) -> Option<v8::Local<'s, v8::Promise>> {
    let message = v8::String::new(scope, message)?;
    let exception = v8::Exception::type_error(scope, message);
    let resolver = v8::PromiseResolver::new(scope)?;
    let promise = resolver.get_promise(scope);
    let _ = resolver.reject(scope, exception);
    Some(promise)
}

pub(crate) fn reject_illegal_invocation_promise(
    scope: &mut v8::PinScope<'_, '_>,
    interface_name: &str,
    method_name: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let message =
        format!("Failed to execute '{method_name}' on '{interface_name}': Illegal invocation");
    if let Some(promise) = rejected_type_error_promise(scope, &message) {
        result.set(promise.into());
    }
}
