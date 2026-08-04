pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_target: v8::Local<'s, v8::Function>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let event_target_prototype = crate::webidl::prototype(scope, event_target)?;
    let properties = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, properties, event_target_prototype.into())
        != Some(true)
    {
        return Err("cannot set WindowProperties inheritance".to_owned());
    }
    let tag_key = v8::Symbol::get_to_string_tag(scope);
    let tag = crate::webidl::string(scope, "WindowProperties")?;
    if properties.define_own_property(
        scope,
        tag_key.into(),
        tag.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define WindowProperties toStringTag".to_owned());
    }
    Ok(properties)
}
