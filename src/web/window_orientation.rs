pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let get = crate::webidl::create_function(
        s,
        "get orientation",
        0,
        v8::ConstructorBehavior::Throw,
        get,
    )?;
    let set = v8::undefined(s);
    let mut d = v8::PropertyDescriptor::new_from_get_set(get.into(), set.into());
    d.set_enumerable(true);
    d.set_configurable(true);
    let k = crate::webidl::string(s, "orientation")?;
    if s.get_current_context()
        .global(s)
        .define_property(s, k.into(), &d)
        == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.orientation".to_owned())
    }
}
fn get(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let angle = crate::fingerprint::edge(s).screen.orientation_angle;
    let legacy = match angle % 360 {
        270 => -90,
        value => value as i32,
    };
    r.set(v8::Integer::new(s, legacy).into());
}
