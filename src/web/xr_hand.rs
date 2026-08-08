use std::collections::HashMap;

#[derive(Clone)]
struct Joint {
    name: String,
    space: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct XrHandStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<Joint>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrHandStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRHand", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrHandStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRHand",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "entries")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrHandStore>()
        .ok_or_else(|| "XRHand state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRHand".to_owned());
    }
    scope
        .get_slot_mut::<XrHandStore>()
        .ok_or_else(|| "XRHand state missing".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}

fn joints(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Vec<Joint>> {
    scope
        .get_slot::<XrHandStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(joints) = joints(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, joints.len() as i32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(joints) = joints(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(joint) = joints.iter().find(|joint| joint.name == name) {
        result.set(v8::Local::new(scope, &joint.space).into())
    } else {
        result.set(v8::undefined(scope).into())
    }
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(joints) = joints(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, joints.len() as i32);
    for (index, joint) in joints.iter().enumerate() {
        if let Some(name) = v8::String::new(scope, &joint.name) {
            let _ = output.set_index(scope, index as u32, name.into());
        }
    }
    result.set(output.into());
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(joints) = joints(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, joints.len() as i32);
    for (index, joint) in joints.iter().enumerate() {
        let joint = v8::Local::new(scope, &joint.space);
        let _ = output.set_index(scope, index as u32, joint.into());
    }
    result.set(output.into());
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(joints) = joints(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, joints.len() as i32);
    for (index, joint) in joints.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        let name = v8::String::new(scope, &joint.name).expect("valid XR joint name");
        let space = v8::Local::new(scope, &joint.space);
        let _ = pair.set_index(scope, 0, name.into());
        let _ = pair.set_index(scope, 1, space.into());
        let _ = output.set_index(scope, index as u32, pair.into());
    }
    result.set(output.into());
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(joints) = joints(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let this_arg = arguments.get(1);
    for joint in joints {
        let name = v8::String::new(scope, &joint.name).expect("valid XR joint name");
        let space = v8::Local::new(scope, &joint.space);
        let callback_arguments = [space.into(), name.into(), arguments.this().into()];
        let _ = callback.call(scope, this_arg, &callback_arguments);
    }
    result.set(v8::undefined(scope).into());
}
