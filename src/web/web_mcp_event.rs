use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct WebMcpEventStore {
    constructor: crate::webidl::RealmConstructor,
    tools: HashMap<i32, String>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(WebMcpEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "WebMCPEvent", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<WebMcpEventStore>()
        .and_then(|x| x.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "WebMCPEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "toolName", get_tool)?;
    crate::webidl::finish_constructor(s, p, c)?;
    super::android_api_support::set_tag(s, p, "WebMCPEvent")?;
    let stored_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<WebMcpEventStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'WebMCPEvent': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'WebMCPEvent': 1 argument required, but only 0 present.",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(s, a.get(0));
    let tool = v8::Local::<v8::Object>::try_from(a.get(1))
        .ok()
        .map(|o| super::android_api_support::string_property(s, o, "toolName"))
        .unwrap_or_default();
    super::event::attach(s, a.this(), event_type, false, false, false);
    s.get_slot_mut::<WebMcpEventStore>()
        .unwrap()
        .tools
        .insert(a.this().get_identity_hash().get(), tool);
    r.set(a.this().into());
}
fn get_tool(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = s
        .get_slot::<WebMcpEventStore>()
        .and_then(|x| x.tools.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(s, &x) {
        r.set(v.into());
    }
}
