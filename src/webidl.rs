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
    value
        .to_string(scope)
        .map(|text| text.to_rust_string_lossy(scope))
        .unwrap_or_default()
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

fn indexed_values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let object = arguments.this();
    let Some(length_key) = v8::String::new(scope, "length") else {
        return;
    };
    let length = object
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let item_key = v8::String::new(scope, "item").expect("short indexed iterator key");
    let item = object
        .get(scope, item_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok());
    let values = v8::Array::new(scope, length as i32);
    for index in 0..length {
        let direct = object.get_index(scope, index);
        let value = if direct.is_some_and(|value| !value.is_undefined()) {
            direct
        } else if let Some(item) = item {
            let index_value = v8::Integer::new_from_unsigned(scope, index);
            item.call(scope, object.into(), &[index_value.into()])
        } else {
            direct
        };
        if let Some(value) = value {
            let _ = values.set_index(scope, index, value);
        }
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
    if let Some(message) = v8::String::new(scope, message) {
        let exception = v8::Exception::type_error(scope, message);
        scope.throw_exception(exception);
    }
}
