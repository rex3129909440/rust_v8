pub(crate) fn ensure(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
) -> bool {
    if super::document::is_document(scope, document) {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn hit_test_elements<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'s, v8::Object>,
    x: f64,
    y: f64,
) -> Vec<v8::Local<'s, v8::Object>> {
    hit_test_elements_in_root(scope, document, x, y)
}

pub(crate) fn hit_test_elements_in_root<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
    x: f64,
    y: f64,
) -> Vec<v8::Local<'s, v8::Object>> {
    if !point_in_viewport(scope, x, y) {
        return Vec::new();
    }
    let mut elements = super::dom_selector::descendants(scope, root)
        .into_iter()
        .enumerate()
        .filter_map(|(order, element)| {
            hit_test_element(scope, element, x, y)
                .then(|| (stacking_key(scope, element), order, element))
        })
        .collect::<Vec<_>>();
    elements.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));
    elements
        .into_iter()
        .map(|(_, _, element)| element)
        .collect()
}

pub(crate) fn hit_test_element(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    x: f64,
    y: f64,
) -> bool {
    if !point_in_viewport(scope, x, y) {
        return false;
    }
    let layout = super::element_layout::compute(scope, element);
    let visibility = property(scope, element, "visibility");
    if !layout.rendered
        || visibility.eq_ignore_ascii_case("hidden")
        || visibility.eq_ignore_ascii_case("collapse")
        || property(scope, element, "pointer-events").eq_ignore_ascii_case("none")
    {
        return false;
    }
    let rect = layout.rect();
    rect.width > 0.0
        && rect.height > 0.0
        && x >= rect.x
        && y >= rect.y
        && x < rect.x + rect.width
        && y < rect.y + rect.height
}

fn point_in_viewport(scope: &v8::PinScope<'_, '_>, x: f64, y: f64) -> bool {
    x.is_finite()
        && y.is_finite()
        && x >= 0.0
        && y >= 0.0
        && x < super::window_view_state::inner_width(scope)
        && y < super::window_view_state::inner_height(scope)
}

fn stacking_key(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> Vec<i64> {
    let mut key = Vec::new();
    let mut current = Some(element);
    while let Some(candidate) = current {
        if super::element::record(scope, candidate).is_some() {
            let position = property(scope, candidate, "position");
            let z_index = if matches!(
                position.to_ascii_lowercase().as_str(),
                "absolute" | "fixed" | "relative" | "sticky"
            ) {
                property(scope, candidate, "z-index")
                    .trim()
                    .parse::<i64>()
                    .unwrap_or(0)
            } else {
                0
            };
            key.push(z_index);
        }
        current = super::node::parent(scope, candidate)
            .or_else(|| super::shadow_root::host(scope, candidate));
    }
    key.reverse();
    key
}

fn property(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    super::get_computed_style_global::computed_property_value(scope, element, name)
}

pub(crate) fn resolved<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    super::writable_stream::resolved_promise(scope, value)
}

pub(crate) fn command_supported(command: &str) -> bool {
    matches!(
        command.to_ascii_lowercase().as_str(),
        "backcolor"
            | "bold"
            | "copy"
            | "createlink"
            | "cut"
            | "decreasefontsize"
            | "defaultparagraphseparator"
            | "delete"
            | "fontname"
            | "fontsize"
            | "forecolor"
            | "formatblock"
            | "forwarddelete"
            | "hilitecolor"
            | "increasefontsize"
            | "indent"
            | "insertbrontyping"
            | "inserthorizontalrule"
            | "inserthtml"
            | "insertimage"
            | "insertlinebreak"
            | "insertorderedlist"
            | "insertparagraph"
            | "inserttext"
            | "insertunorderedlist"
            | "italic"
            | "justifycenter"
            | "justifyfull"
            | "justifyleft"
            | "justifyright"
            | "outdent"
            | "paste"
            | "redo"
            | "removeformat"
            | "selectall"
            | "strikethrough"
            | "stylewithcss"
            | "subscript"
            | "superscript"
            | "underline"
            | "undo"
            | "unlink"
            | "usecss"
    )
}
