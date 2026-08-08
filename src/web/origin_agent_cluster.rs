pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get originAgentCluster",
        0,
        v8::ConstructorBehavior::Throw,
        get_origin_agent_cluster,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "originAgentCluster")?;
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.originAgentCluster".to_owned())
    }
}

fn get_origin_agent_cluster(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Boolean::new(scope, true).into());
}
