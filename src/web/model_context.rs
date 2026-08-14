use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Tool {
    name: String,
    description: String,
    input_schema: v8::Global<v8::Value>,
    execute: v8::Global<v8::Function>,
}
#[derive(Default)]
pub(crate) struct ModelContextStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
    tools: HashMap<i32, Vec<Tool>>,
    handlers: HashMap<i32, v8::Global<v8::Value>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(ModelContextStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "ModelContext", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<ModelContextStore>()
        .and_then(|x| x.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "ModelContext",
        0,
        v8::ConstructorBehavior::Allow,
        super::android_api_support::illegal_constructor,
    )?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "ontoolchange", get_handler, set_handler)?;
    crate::webidl::define_method(s, p, "executeTool", 2, execute_tool)?;
    crate::webidl::define_method(s, p, "getTools", 0, get_tools)?;
    crate::webidl::define_method(s, p, "registerTool", 1, register_tool)?;
    crate::webidl::finish_constructor(s, p, c)?;
    super::android_api_support::set_tag(s, p, "ModelContext")?;
    let stored_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<ModelContextStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create ModelContext".to_owned());
    }
    super::event_target::attach(s, o);
    let id = o.get_identity_hash().get();
    let store = s.get_slot_mut::<ModelContextStore>().unwrap();
    store.instances.insert(id);
    store.tools.insert(id, Vec::new());
    Ok(o)
}
fn valid(
    s: &mut v8::PinScope<'_, '_>,
    a: &v8::FunctionCallbackArguments<'_>,
    op: &str,
) -> Option<i32> {
    let id = a.this().get_identity_hash().get();
    let valid = s
        .get_slot::<ModelContextStore>()
        .unwrap()
        .instances
        .contains(&id);
    super::android_api_support::require_brand(s, valid, "ModelContext", op).then_some(id)
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(s, &a, "ontoolchange") else {
        return;
    };
    if let Some(v) = s
        .get_slot::<ModelContextStore>()
        .and_then(|x| x.handlers.get(&id))
    {
        r.set(v8::Local::new(s, v));
    } else {
        r.set(v8::null(s).into());
    }
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(s, &a, "ontoolchange") else {
        return;
    };
    let value = super::window_event_handler_support::handler_value(s, a.get(0));
    let present = {
        let store = s.get_slot_mut::<ModelContextStore>().unwrap();
        if let Some(v) = value {
            store.handlers.insert(id, v);
        } else {
            store.handlers.remove(&id);
        }
        store.handlers.contains_key(&id)
    };
    super::event_target::set_attribute_handler(s, a.this(), "toolchange", present);
}
fn register_tool(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(s, &a, "registerTool") else {
        return;
    };
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'registerTool' on 'ModelContext': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(o) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "ModelContextTool must be an object");
        return;
    };
    let name = super::android_api_support::string_property(s, o, "name");
    let description = super::android_api_support::string_property(s, o, "description");
    let input_schema = super::android_api_support::property(s, o, "inputSchema")
        .unwrap_or_else(|| v8::undefined(s).into());
    let Some(execute) = super::android_api_support::property(s, o, "execute")
        .and_then(|v| v8::Local::<v8::Function>::try_from(v).ok())
    else {
        crate::webidl::throw_type_error(s, "ModelContextTool.execute must be a function");
        return;
    };
    let input_schema = v8::Global::new(s, input_schema);
    let execute = v8::Global::new(s, execute);
    s.get_slot_mut::<ModelContextStore>()
        .unwrap()
        .tools
        .entry(id)
        .or_default()
        .push(Tool {
            name,
            description,
            input_schema,
            execute,
        });
    if let Some(p) = super::android_api_support::resolved_undefined(s) {
        r.set(p.into());
    }
}
fn tool_object<'s>(s: &mut v8::PinScope<'s, '_>, tool: &Tool) -> v8::Local<'s, v8::Object> {
    let o = v8::Object::new(s);
    for (name, value) in [
        ("name", tool.name.as_str()),
        ("description", tool.description.as_str()),
        ("origin", ""),
    ] {
        if let (Some(k), Some(v)) = (v8::String::new(s, name), v8::String::new(s, value)) {
            let _ = o.set(s, k.into(), v.into());
        }
    }
    if let Some(k) = v8::String::new(s, "inputSchema") {
        let _ = o.set(s, k.into(), v8::Local::new(s, &tool.input_schema));
    }
    o
}
fn get_tools(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(s, &a, "getTools") else {
        return;
    };
    let tools = s
        .get_slot::<ModelContextStore>()
        .and_then(|x| x.tools.get(&id))
        .cloned()
        .unwrap_or_default();
    let values = v8::Array::new(s, tools.len() as i32);
    for (i, tool) in tools.iter().enumerate() {
        let value = tool_object(s, tool);
        let _ = values.set_index(s, i as u32, value.into());
    }
    if let Ok(p) = super::writable_stream::resolved_promise(s, values.into()) {
        r.set(p.into());
    }
}
fn execute_tool(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(id) = valid(s, &a, "executeTool") else {
        return;
    };
    if a.length() < 2 {
        crate::webidl::throw_type_error(
            s,
            &format!(
                "Failed to execute 'executeTool' on 'ModelContext': 2 arguments required, but only {} present.",
                a.length()
            ),
        );
        return;
    }
    let name = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .map(|o| super::android_api_support::string_property(s, o, "name"))
        .unwrap_or_default();
    let input = a.get(1);
    let tool = s
        .get_slot::<ModelContextStore>()
        .and_then(|x| x.tools.get(&id))
        .and_then(|x| x.iter().find(|t| t.name == name))
        .cloned();
    let Some(tool) = tool else {
        let value = v8::null(s);
        if let Ok(p) = super::writable_stream::resolved_promise(s, value.into()) {
            r.set(p.into());
        }
        return;
    };
    let f = v8::Local::new(s, &tool.execute);
    let receiver = v8::undefined(s);
    let value = f
        .call(s, receiver.into(), &[input])
        .unwrap_or_else(|| v8::undefined(s).into());
    if let Ok(p) = super::writable_stream::resolved_promise(s, value) {
        r.set(p.into());
    }
}
