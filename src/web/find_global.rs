pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "find", 0, v8::ConstructorBehavior::Throw, find)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "find")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.find".to_owned())
    }
}

fn find(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let query = crate::webidl::value_to_string(scope, arguments.get(0));
    let case_sensitive = arguments.get(1).boolean_value(scope);
    let whole_word = arguments.get(4).boolean_value(scope);
    let text = super::document_global::value(scope)
        .map(|document| {
            let mut text = super::node::text_content(scope, document);
            text.push_str(&super::document::searchable_text(scope, document));
            text
        })
        .unwrap_or_default();
    let found = if query.is_empty() {
        false
    } else if case_sensitive {
        contains(&text, &query, whole_word)
    } else {
        contains(&text.to_lowercase(), &query.to_lowercase(), whole_word)
    };
    result.set(v8::Boolean::new(scope, found).into());
}

fn contains(text: &str, query: &str, whole_word: bool) -> bool {
    if !whole_word {
        return text.contains(query);
    }
    text.match_indices(query).any(|(index, _)| {
        let before = text[..index].chars().next_back();
        let after = text[index + query.len()..].chars().next();
        before.is_none_or(|value| !value.is_alphanumeric() && value != '_')
            && after.is_none_or(|value| !value.is_alphanumeric() && value != '_')
    })
}
