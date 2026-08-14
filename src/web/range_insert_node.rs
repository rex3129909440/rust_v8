pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "insertNode", 1, insert_node_callback)
}

fn insert_node_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::abstract_range::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(new_node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    match insert_node(scope, arguments.this(), new_node) {
        Ok(()) => {}
        Err(("TypeError", message)) => crate::webidl::throw_type_error(scope, &message),
        Err((name, message)) => super::node::throw_dom_exception(scope, name, &message),
    }
}

pub(crate) fn insert_node(
    scope: &mut v8::PinScope<'_, '_>,
    range: v8::Local<'_, v8::Object>,
    new_node: v8::Local<'_, v8::Object>,
) -> Result<(), (&'static str, String)> {
    if super::node::record(scope, new_node).is_none() {
        return Err(("TypeError", "The argument is not a Node".to_owned()));
    }
    let record = super::abstract_range::record(scope, range)
        .ok_or_else(|| ("TypeError", "Illegal invocation".to_owned()))?;
    let start = v8::Local::new(scope, &record.start_container);
    if start.strict_equals(new_node.into()) {
        return Err((
            "HierarchyRequestError",
            "A range cannot insert its own boundary container".to_owned(),
        ));
    }
    let inserted_count =
        if super::node::record(scope, new_node).is_some_and(|value| value.node_type == 11) {
            super::node::children(scope, new_node).len() as u32
        } else {
            1
        };
    let collapsed =
        record.start_container == record.end_container && record.start_offset == record.end_offset;

    if let Some(data) = super::text::data_if_text(scope, start) {
        let Some(parent) = super::node::parent(scope, start) else {
            return Err((
                "HierarchyRequestError",
                "The range starts in a Text node without a parent".to_owned(),
            ));
        };
        let units: Vec<u16> = data.encode_utf16().collect();
        let offset = (record.start_offset as usize).min(units.len());
        let left = String::from_utf16_lossy(&units[..offset]);
        let right = String::from_utf16_lossy(&units[offset..]);
        let _ = super::character_data::set_data_without_range_adjustment(scope, start, left);
        let split = super::text::create(scope, right).map_err(|message| ("TypeError", message))?;
        if let Some(document) = super::node::record(scope, start)
            .and_then(|value| value.owner_document)
            .map(|document| v8::Local::new(scope, &document))
        {
            super::node::set_owner_document(scope, split, document);
        }
        let siblings = super::node::children(scope, parent);
        let index = siblings
            .iter()
            .position(|candidate| candidate.strict_equals(start.into()))
            .unwrap_or(siblings.len());
        super::node::insert_node(scope, parent, split, index + 1)
            .map_err(|(name, message)| (name, message.to_owned()))?;
        super::abstract_range::adjust_for_split_text(scope, start, split, record.start_offset);
        super::node::insert_node(scope, parent, new_node, index + 1)
            .map_err(|(name, message)| (name, message.to_owned()))?;
        let parent_global = v8::Global::new(scope, parent);
        let split_global = v8::Global::new(scope, split);
        super::abstract_range::update(scope, range, |value| {
            if collapsed {
                value.end_container = parent_global;
                value.end_offset = index as u32 + inserted_count + 1;
            } else if value.end_container == record.start_container
                && value.end_offset > record.start_offset
            {
                value.end_container = split_global;
                value.end_offset -= record.start_offset;
            }
        });
        return Ok(());
    }

    let index = record.start_offset as usize;
    super::node::insert_node(scope, start, new_node, index)
        .map_err(|(name, message)| (name, message.to_owned()))?;
    super::abstract_range::update(scope, range, |value| {
        if value.end_container == record.start_container
            && (collapsed || value.end_offset > record.start_offset)
        {
            value.end_offset = value.end_offset.saturating_add(inserted_count);
        }
    });
    Ok(())
}
