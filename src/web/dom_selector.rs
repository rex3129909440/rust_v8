#[derive(Clone, Copy)]
enum Combinator {
    Descendant,
    Child,
    Adjacent,
    Sibling,
}

#[derive(Default)]
struct Compound {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    attributes: Vec<AttributeSelector>,
    scope: bool,
    first_child: bool,
    last_child: bool,
    only_child: bool,
    first_of_type: bool,
    last_of_type: bool,
    only_of_type: bool,
    empty: bool,
    root: bool,
    checked: bool,
    disabled: bool,
    enabled: bool,
    required: bool,
    optional: bool,
    read_only: bool,
    read_write: bool,
    link: bool,
    lang: Option<String>,
    direction: Option<String>,
    default_state: bool,
    indeterminate: bool,
    valid: bool,
    invalid: bool,
    pseudo_element: bool,
    not: Vec<ComplexSelector>,
    any: Vec<ComplexSelector>,
    has: Vec<String>,
    nth: Vec<NthSelector>,
}

#[derive(Clone, Copy)]
enum AttributeOperator {
    Present,
    Equals,
    Includes,
    DashMatch,
    Prefix,
    Suffix,
    Substring,
}

struct AttributeSelector {
    name: String,
    operator: AttributeOperator,
    value: String,
    case_insensitive: bool,
}

struct NthSelector {
    a: i32,
    b: i32,
    from_end: bool,
    of_type: bool,
    of: Vec<ComplexSelector>,
}

struct ComplexSelector {
    compounds: Vec<Compound>,
    combinators: Vec<Combinator>,
}

pub(crate) fn descendants<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let mut output = Vec::new();
    collect_descendants(scope, root, &mut output);
    output
}

fn collect_descendants<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'_, v8::Object>,
    output: &mut Vec<v8::Local<'s, v8::Object>>,
) {
    for child in super::node::children(scope, root) {
        if super::element::record(scope, child).is_some() {
            output.push(child);
        }
        collect_descendants(scope, child, output);
    }
}

pub(crate) fn query_selector_all<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
    selector: &str,
) -> Result<Vec<v8::Local<'s, v8::Object>>, String> {
    let selectors = parse_selector_list(selector)?;
    let candidates = descendants(scope, root);
    let scope_root = if super::document::is_document(scope, root) {
        super::document::document_child_elements(scope, root)
            .into_iter()
            .next()
            .unwrap_or(root)
    } else {
        root
    };
    Ok(candidates
        .into_iter()
        .filter(|candidate| {
            selectors
                .iter()
                .any(|selector| matches_complex(scope, *candidate, scope_root, selector))
        })
        .collect())
}

pub(crate) fn matches_selector(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    selector: &str,
    scope_root: v8::Local<'_, v8::Object>,
) -> Result<bool, String> {
    let selectors = parse_selector_list(selector)?;
    Ok(selectors
        .iter()
        .any(|selector| matches_complex(scope, element, scope_root, selector)))
}

pub(crate) fn throw_api_error(
    scope: &mut v8::PinScope<'_, '_>,
    method: &str,
    interface: &str,
    selector: &str,
) {
    let prefix = format!("Failed to execute '{method}' on '{interface}'");
    let message = if selector.is_empty() {
        format!("{prefix}: The provided selector is empty.")
    } else {
        format!("{prefix}: '{selector}' is not a valid selector.")
    };
    super::node::throw_dom_exception(scope, "SyntaxError", &message);
}

fn matches_complex(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    scope_root: v8::Local<'_, v8::Object>,
    selector: &ComplexSelector,
) -> bool {
    if selector.compounds.is_empty() {
        return false;
    }
    matches_at(
        scope,
        element,
        scope_root,
        selector,
        selector.compounds.len() - 1,
    )
}

fn matches_at(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    scope_root: v8::Local<'_, v8::Object>,
    selector: &ComplexSelector,
    index: usize,
) -> bool {
    if !matches_compound(scope, element, scope_root, &selector.compounds[index]) {
        return false;
    }
    if index == 0 {
        return true;
    }
    match selector.combinators[index - 1] {
        Combinator::Child => super::node::parent(scope, element)
            .is_some_and(|parent| matches_at(scope, parent, scope_root, selector, index - 1)),
        Combinator::Descendant => {
            let mut parent = super::node::parent(scope, element);
            while let Some(candidate) = parent {
                if matches_at(scope, candidate, scope_root, selector, index - 1) {
                    return true;
                }
                parent = super::node::parent(scope, candidate);
            }
            false
        }
        Combinator::Adjacent => previous_element_sibling(scope, element)
            .is_some_and(|sibling| matches_at(scope, sibling, scope_root, selector, index - 1)),
        Combinator::Sibling => {
            let Some(parent) = super::node::parent(scope, element) else {
                return false;
            };
            for sibling in super::node::children(scope, parent) {
                if sibling.get_identity_hash().get() == element.get_identity_hash().get() {
                    break;
                }
                if super::element::record(scope, sibling).is_some()
                    && matches_at(scope, sibling, scope_root, selector, index - 1)
                {
                    return true;
                }
            }
            false
        }
    }
}

fn matches_compound(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    scope_root: v8::Local<'_, v8::Object>,
    selector: &Compound,
) -> bool {
    let Some(record) = super::element::record(scope, element) else {
        return false;
    };
    if selector.scope && element.get_identity_hash().get() != scope_root.get_identity_hash().get() {
        return false;
    }
    if selector.pseudo_element {
        return false;
    }
    if let Some(tag) = &selector.tag {
        let tag = tag.rsplit('|').next().unwrap_or(tag);
        let case_insensitive = record.namespace_uri.as_deref()
            == Some("http://www.w3.org/1999/xhtml")
            && super::node::owner_document(scope, element).is_some_and(|document| {
                super::document::content_type_value(scope, document).as_deref() == Some("text/html")
            });
        if tag != "*"
            && if case_insensitive {
                !record.tag_name.eq_ignore_ascii_case(tag)
            } else {
                record.tag_name != *tag
            }
        {
            return false;
        }
    }
    if selector.id.as_ref().is_some_and(|id| {
        super::element::attribute_value(scope, element, "id").as_deref() != Some(id)
    }) {
        return false;
    }
    let classes = super::element::attribute_value(scope, element, "class").unwrap_or_default();
    if !selector.classes.iter().all(|wanted| {
        classes
            .split_ascii_whitespace()
            .any(|candidate| candidate == wanted)
    }) {
        return false;
    }
    for attribute in &selector.attributes {
        let actual = super::element::attribute_value(scope, element, &attribute.name);
        let Some(actual) = actual else {
            return false;
        };
        if matches!(attribute.operator, AttributeOperator::Present) {
            continue;
        }
        let expected = &attribute.value;
        let matches = if attribute.case_insensitive
            || html_attribute_value_is_ascii_case_insensitive(scope, element, &attribute.name)
        {
            attribute_matches(
                attribute.operator,
                &actual.to_ascii_lowercase(),
                &expected.to_ascii_lowercase(),
            )
        } else {
            attribute_matches(attribute.operator, &actual, expected)
        };
        if !matches {
            return false;
        }
    }
    let element_siblings = super::node::parent(scope, element)
        .map(|parent| {
            super::node::children(scope, parent)
                .into_iter()
                .filter(|child| super::element::record(scope, *child).is_some())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if selector.first_child
        && element_siblings.first().is_none_or(|first| {
            first.get_identity_hash().get() != element.get_identity_hash().get()
        })
    {
        return false;
    }
    if selector.last_child
        && element_siblings
            .last()
            .is_none_or(|last| last.get_identity_hash().get() != element.get_identity_hash().get())
    {
        return false;
    }
    if selector.only_child && element_siblings.len() != 1 {
        return false;
    }
    let same_type_siblings = element_siblings
        .iter()
        .copied()
        .filter(|sibling| {
            super::element::record(scope, *sibling).is_some_and(|sibling_record| {
                sibling_record.tag_name == record.tag_name
                    && sibling_record.namespace_uri == record.namespace_uri
            })
        })
        .collect::<Vec<_>>();
    if selector.first_of_type
        && same_type_siblings
            .first()
            .is_none_or(|first| !first.strict_equals(element.into()))
    {
        return false;
    }
    if selector.last_of_type
        && same_type_siblings
            .last()
            .is_none_or(|last| !last.strict_equals(element.into()))
    {
        return false;
    }
    if selector.only_of_type && same_type_siblings.len() != 1 {
        return false;
    }
    if selector.empty
        && super::node::children(scope, element)
            .into_iter()
            .any(|child| {
                super::node::record(scope, child)
                    .is_some_and(|record| matches!(record.node_type, 1 | 3 | 4))
            })
    {
        return false;
    }
    if selector.root
        && super::node::parent(scope, element).is_none_or(|parent| {
            super::node::record(scope, parent).is_none_or(|record| record.node_type != 9)
        })
    {
        return false;
    }
    let is_checked = super::html_input_element::record(scope, element)
        .is_some_and(|record| record.checked)
        || super::html_option_element::record(scope, element).is_some_and(|record| record.selected);
    if selector.checked && !is_checked {
        return false;
    }
    let disableable = matches!(
        record.tag_name.to_ascii_uppercase().as_str(),
        "BUTTON" | "FIELDSET" | "INPUT" | "OPTGROUP" | "OPTION" | "SELECT" | "TEXTAREA"
    );
    let disabled_by_optgroup = record.tag_name.eq_ignore_ascii_case("OPTION")
        && super::node::parent(scope, element).is_some_and(|parent| {
            super::element::record(scope, parent).is_some_and(|parent_record| {
                parent_record.tag_name.eq_ignore_ascii_case("OPTGROUP")
                    && super::element::attribute_value(scope, parent, "disabled").is_some()
            })
        });
    let is_disabled = disableable
        && (super::element::attribute_value(scope, element, "disabled").is_some()
            || disabled_by_optgroup
            || super::html_element::disabled_by_fieldset(scope, element, &record.tag_name));
    if selector.disabled && !is_disabled {
        return false;
    }
    if selector.enabled && (!disableable || is_disabled) {
        return false;
    }
    let requireable = matches!(
        record.tag_name.to_ascii_uppercase().as_str(),
        "BUTTON" | "INPUT" | "SELECT" | "TEXTAREA"
    );
    let is_required = requireable
        && !record.tag_name.eq_ignore_ascii_case("BUTTON")
        && super::element::attribute_value(scope, element, "required").is_some();
    if selector.required && !is_required {
        return false;
    }
    if selector.optional && (!requireable || is_required) {
        return false;
    }
    let read_write = is_read_write(scope, element, &record.tag_name, is_disabled);
    let read_only = is_read_only(scope, element, &record.tag_name, is_disabled);
    if selector.read_write && !read_write {
        return false;
    }
    if selector.read_only && !read_only {
        return false;
    }
    let is_link = matches!(record.tag_name.to_ascii_uppercase().as_str(), "A" | "AREA")
        && super::element::attribute_value(scope, element, "href").is_some();
    if selector.link && !is_link {
        return false;
    }
    if let Some(language) = &selector.lang
        && !language_matches(scope, element, language)
    {
        return false;
    }
    if let Some(direction) = &selector.direction
        && inherited_attribute(scope, element, "dir")
            .unwrap_or_else(|| "ltr".to_owned())
            .to_ascii_lowercase()
            != direction.to_ascii_lowercase()
    {
        return false;
    }
    let is_default = super::html_input_element::record(scope, element)
        .is_some_and(|record| record.default_checked)
        || super::html_option_element::record(scope, element)
            .is_some_and(|record| record.default_selected);
    if selector.default_state && !is_default {
        return false;
    }
    let is_indeterminate = super::html_input_element::record(scope, element)
        .is_some_and(|record| record.indeterminate);
    if selector.indeterminate && !is_indeterminate {
        return false;
    }
    let validity = selector_validity(scope, element);
    if selector.valid && validity != Some(true) {
        return false;
    }
    if selector.invalid && validity != Some(false) {
        return false;
    }
    if selector
        .not
        .iter()
        .any(|nested| matches_complex(scope, element, element, nested))
    {
        return false;
    }
    if !selector.any.is_empty()
        && !selector
            .any
            .iter()
            .any(|nested| matches_complex(scope, element, element, nested))
    {
        return false;
    }
    if !selector.has.is_empty()
        && !selector
            .has
            .iter()
            .any(|relative| matches_has(scope, element, relative))
    {
        return false;
    }
    for nth in &selector.nth {
        let mut siblings = element_siblings
            .iter()
            .copied()
            .filter(|sibling| {
                (!nth.of_type
                    || super::element::record(scope, *sibling).is_some_and(|sibling_record| {
                        sibling_record.tag_name == record.tag_name
                            && sibling_record.namespace_uri == record.namespace_uri
                    }))
                    && (nth.of.is_empty()
                        || nth
                            .of
                            .iter()
                            .any(|selector| matches_complex(scope, *sibling, *sibling, selector)))
            })
            .collect::<Vec<_>>();
        if nth.from_end {
            siblings.reverse();
        }
        let Some(position) = siblings
            .iter()
            .position(|sibling| sibling.strict_equals(element.into()))
            .map(|position| position as i32 + 1)
        else {
            return false;
        };
        if !matches_nth(position, nth.a, nth.b) {
            return false;
        }
    }
    true
}

fn html_attribute_value_is_ascii_case_insensitive(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    if !name.eq_ignore_ascii_case("type") {
        return false;
    }
    let Some(record) = super::element::record(scope, element) else {
        return false;
    };
    if record.namespace_uri.as_deref() != Some("http://www.w3.org/1999/xhtml") {
        return false;
    }
    let Some(document) = super::node::owner_document(scope, element) else {
        return false;
    };
    if super::document::content_type_value(scope, document).as_deref() != Some("text/html") {
        return false;
    }
    matches!(
        record.tag_name.to_ascii_uppercase().as_str(),
        "BUTTON" | "INPUT" | "LI" | "OL" | "UL"
    )
}

fn inherited_attribute(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let mut current = Some(element);
    while let Some(candidate) = current {
        if let Some(value) = super::element::attribute_value(scope, candidate, name)
            && !value.is_empty()
        {
            return Some(value);
        }
        current = super::node::parent(scope, candidate)
            .filter(|parent| super::element::record(scope, *parent).is_some());
    }
    None
}

fn language_matches(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    requested: &str,
) -> bool {
    let Some(language) = inherited_attribute(scope, element, "lang") else {
        return false;
    };
    language.eq_ignore_ascii_case(requested)
        || language
            .get(..requested.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(requested))
            && language.as_bytes().get(requested.len()) == Some(&b'-')
}

fn is_read_write(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    tag_name: &str,
    disabled: bool,
) -> bool {
    if let Some(record) = super::html_input_element::record(scope, element) {
        let text_like = matches!(
            record.input_type.as_str(),
            "email" | "number" | "password" | "search" | "tel" | "text" | "url"
        );
        return text_like && !record.read_only && !disabled;
    }
    if let Some(record) = super::html_text_area_element::record(scope, element) {
        return !record.booleans.get("readOnly").copied().unwrap_or(false) && !disabled;
    }
    if tag_name.eq_ignore_ascii_case("DIV") {
        return super::element::attribute_value(scope, element, "contenteditable")
            .is_some_and(|value| value.is_empty() || value.eq_ignore_ascii_case("true"));
    }
    false
}

fn is_read_only(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    _tag_name: &str,
    disabled: bool,
) -> bool {
    if let Some(record) = super::html_input_element::record(scope, element) {
        let text_like = matches!(
            record.input_type.as_str(),
            "email" | "number" | "password" | "search" | "tel" | "text" | "url"
        );
        return !text_like || record.read_only || disabled;
    }
    if let Some(record) = super::html_text_area_element::record(scope, element) {
        return record.booleans.get("readOnly").copied().unwrap_or(false) || disabled;
    }
    false
}

fn selector_validity(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<bool> {
    if let Some(record) = super::html_input_element::record(scope, element) {
        return super::html_input_element::will_validate(&record)
            .then(|| !super::html_input_element::invalid(&record));
    }
    if let Some(record) = super::html_select_element::record(scope, element) {
        return (!record.disabled).then(|| !super::html_select_element::invalid(scope, element));
    }
    if let Some(record) = super::html_text_area_element::record(scope, element) {
        return super::html_text_area_element::is_candidate(&record)
            .then(|| super::html_text_area_element::is_valid(&record));
    }
    if super::html_form_element::record(scope, element).is_some() {
        return Some(
            super::html_form_element::collect_controls(scope, element)
                .into_iter()
                .all(|control| selector_validity(scope, control) != Some(false)),
        );
    }
    None
}

fn attribute_matches(operator: AttributeOperator, actual: &str, expected: &str) -> bool {
    match operator {
        AttributeOperator::Present => true,
        AttributeOperator::Equals => actual == expected,
        AttributeOperator::Includes => {
            !expected.is_empty()
                && actual
                    .split_ascii_whitespace()
                    .any(|token| token == expected)
        }
        AttributeOperator::DashMatch => {
            actual == expected
                || actual
                    .strip_prefix(expected)
                    .is_some_and(|rest| rest.starts_with('-'))
        }
        AttributeOperator::Prefix => !expected.is_empty() && actual.starts_with(expected),
        AttributeOperator::Suffix => !expected.is_empty() && actual.ends_with(expected),
        AttributeOperator::Substring => !expected.is_empty() && actual.contains(expected),
    }
}

fn matches_nth(position: i32, a: i32, b: i32) -> bool {
    if a == 0 {
        return position == b;
    }
    let difference = position - b;
    difference % a == 0 && difference / a >= 0
}

fn matches_has(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    relative: &str,
) -> bool {
    let relative = relative.trim();
    if let Some(selector) = relative.strip_prefix('>') {
        return super::node::children(scope, element)
            .into_iter()
            .any(|child| {
                super::element::record(scope, child).is_some()
                    && matches_selector(scope, child, selector.trim(), element).unwrap_or(false)
            });
    }
    if let Some(selector) = relative.strip_prefix('+') {
        return next_element_sibling(scope, element).is_some_and(|sibling| {
            matches_selector(scope, sibling, selector.trim(), element).unwrap_or(false)
        });
    }
    if let Some(selector) = relative.strip_prefix('~') {
        let Some(parent) = super::node::parent(scope, element) else {
            return false;
        };
        let mut after = false;
        return super::node::children(scope, parent)
            .into_iter()
            .any(|sibling| {
                if sibling.strict_equals(element.into()) {
                    after = true;
                    return false;
                }
                after
                    && super::element::record(scope, sibling).is_some()
                    && matches_selector(scope, sibling, selector.trim(), element).unwrap_or(false)
            });
    }
    query_selector_all(scope, element, relative).is_ok_and(|matches| !matches.is_empty())
}

fn next_element_sibling<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let parent = super::node::parent(scope, element)?;
    let mut after = false;
    for child in super::node::children(scope, parent) {
        if child.strict_equals(element.into()) {
            after = true;
        } else if after && super::element::record(scope, child).is_some() {
            return Some(child);
        }
    }
    None
}

fn previous_element_sibling<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let parent = super::node::parent(scope, element)?;
    let mut previous = None;
    for child in super::node::children(scope, parent) {
        if child.get_identity_hash().get() == element.get_identity_hash().get() {
            return previous;
        }
        if super::element::record(scope, child).is_some() {
            previous = Some(child);
        }
    }
    None
}

fn parse_selector_list(source: &str) -> Result<Vec<ComplexSelector>, String> {
    let groups = split_top_level(source, ',')?;
    if groups.is_empty() {
        return Err("The provided selector is empty".to_owned());
    }
    groups
        .into_iter()
        .map(|group| parse_complex(group.trim()))
        .collect()
}

fn parse_complex(source: &str) -> Result<ComplexSelector, String> {
    if source.is_empty() {
        return Err("The provided selector is empty".to_owned());
    }
    if source.starts_with(['>', '+', '~']) || source.ends_with(['>', '+', '~']) {
        return Err("Invalid selector combinator".to_owned());
    }
    let mut compounds = Vec::new();
    let mut combinators = Vec::new();
    let mut start = 0;
    let mut depth = 0_i32;
    let mut quote = None;
    let chars = source.char_indices().collect::<Vec<_>>();
    let mut index = 0;
    while index < chars.len() {
        let (offset, character) = chars[index];
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if character == '\\' {
            let mut next = index + 1;
            if next >= chars.len() {
                return Err("Invalid escape sequence".to_owned());
            }
            if chars[next].1.is_ascii_hexdigit() {
                let mut digits = 0;
                while next < chars.len() && digits < 6 && chars[next].1.is_ascii_hexdigit() {
                    next += 1;
                    digits += 1;
                }
                if next < chars.len() && chars[next].1.is_ascii_whitespace() {
                    next += 1;
                }
            } else {
                next += 1;
            }
            index = next;
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            '>' | '+' | '~' if depth == 0 => {
                push_compound(source, start, offset, &mut compounds)?;
                combinators.push(match character {
                    '>' => Combinator::Child,
                    '+' => Combinator::Adjacent,
                    _ => Combinator::Sibling,
                });
                start = offset + character.len_utf8();
            }
            character if character.is_ascii_whitespace() && depth == 0 => {
                let end = offset;
                let mut next = index + 1;
                while next < chars.len() && chars[next].1.is_ascii_whitespace() {
                    next += 1;
                }
                if source[start..end].trim().is_empty() {
                    start = chars.get(next).map(|entry| entry.0).unwrap_or(source.len());
                    index = next;
                    continue;
                }
                push_compound(source, start, end, &mut compounds)?;
                if chars
                    .get(next)
                    .is_none_or(|(_, character)| !matches!(character, '>' | '+' | '~'))
                {
                    combinators.push(Combinator::Descendant);
                }
                start = chars.get(next).map(|entry| entry.0).unwrap_or(source.len());
                index = next;
                continue;
            }
            _ => {}
        }
        if depth < 0 {
            return Err("Unbalanced selector".to_owned());
        }
        index += 1;
    }
    if quote.is_some() || depth != 0 {
        return Err("Unbalanced selector".to_owned());
    }
    push_compound(source, start, source.len(), &mut compounds)?;
    while combinators.len() >= compounds.len() {
        combinators.pop();
    }
    if compounds.is_empty() || combinators.len() + 1 != compounds.len() {
        return Err("Invalid selector combinator".to_owned());
    }
    Ok(ComplexSelector {
        compounds,
        combinators,
    })
}

fn push_compound(
    source: &str,
    start: usize,
    end: usize,
    compounds: &mut Vec<Compound>,
) -> Result<(), String> {
    let value = source[start..end].trim();
    if !value.is_empty() {
        compounds.push(parse_compound(value)?);
    }
    Ok(())
}

fn parse_compound(source: &str) -> Result<Compound, String> {
    let mut selector = Compound::default();
    let bytes = source.as_bytes();
    let mut index = 0;
    if bytes.first().is_some_and(|byte| {
        *byte == b'*' || *byte == b'\\' || is_name_start(*byte) || !byte.is_ascii()
    }) {
        let (name, end) = consume_identifier(source, index)?;
        selector.tag = Some(name);
        index = end;
    }
    while index < bytes.len() {
        match bytes[index] {
            b'#' => {
                let (name, end) = consume_identifier(source, index + 1)?;
                if end == index + 1 {
                    return Err("Invalid id selector".to_owned());
                }
                selector.id = Some(name);
                index = end;
            }
            b'.' => {
                let (name, end) = consume_identifier(source, index + 1)?;
                if end == index + 1 {
                    return Err("Invalid class selector".to_owned());
                }
                selector.classes.push(name);
                index = end;
            }
            b'[' => {
                let end = find_closing(bytes, index, b'[', b']')?;
                selector
                    .attributes
                    .push(parse_attribute(&source[index + 1..end])?);
                index = end + 1;
            }
            b':' => {
                if bytes.get(index + 1) == Some(&b':') {
                    let (name, end) = consume_identifier(source, index + 2)?;
                    if !matches!(
                        name.to_ascii_lowercase().as_str(),
                        "after"
                            | "backdrop"
                            | "before"
                            | "file-selector-button"
                            | "first-letter"
                            | "first-line"
                            | "marker"
                            | "placeholder"
                            | "selection"
                    ) {
                        return Err("Unsupported pseudo-element".to_owned());
                    }
                    selector.pseudo_element = true;
                    index = end;
                    continue;
                }
                let (pseudo_name, end) = consume_identifier(source, index + 1)?;
                let pseudo = pseudo_name.as_str();
                let lower = pseudo.to_ascii_lowercase();
                let argument = if bytes.get(end) == Some(&b'(') {
                    let close = find_closing(bytes, end, b'(', b')')?;
                    index = close + 1;
                    Some(source[end + 1..close].trim())
                } else {
                    index = end;
                    None
                };
                match lower.as_str() {
                    "scope" => selector.scope = true,
                    "first-child" => selector.first_child = true,
                    "last-child" => selector.last_child = true,
                    "only-child" => selector.only_child = true,
                    "first-of-type" => selector.first_of_type = true,
                    "last-of-type" => selector.last_of_type = true,
                    "only-of-type" => selector.only_of_type = true,
                    "empty" => selector.empty = true,
                    "root" => selector.root = true,
                    "checked" => selector.checked = true,
                    "disabled" => selector.disabled = true,
                    "enabled" => selector.enabled = true,
                    "required" => selector.required = true,
                    "optional" => selector.optional = true,
                    "read-only" => selector.read_only = true,
                    "read-write" => selector.read_write = true,
                    "link" | "any-link" => selector.link = true,
                    "lang" => {
                        selector.lang = Some(
                            required_argument(pseudo, argument)?
                                .trim_matches(['\'', '"'])
                                .to_owned(),
                        )
                    }
                    "dir" => {
                        let direction = required_argument(pseudo, argument)?.to_ascii_lowercase();
                        if !matches!(direction.as_str(), "ltr" | "rtl") {
                            return Err("Invalid :dir argument".to_owned());
                        }
                        selector.direction = Some(direction)
                    }
                    "default" => selector.default_state = true,
                    "indeterminate" => selector.indeterminate = true,
                    "valid" => selector.valid = true,
                    "invalid" => selector.invalid = true,
                    "not" => selector
                        .not
                        .extend(parse_selector_list(required_argument(pseudo, argument)?)?),
                    "is" | "where" => {
                        selector
                            .any
                            .extend(parse_forgiving_selector_list(required_argument(
                                pseudo, argument,
                            )?)?)
                    }
                    "has" => selector
                        .has
                        .extend(parse_relative_selector_list(required_argument(
                            pseudo, argument,
                        )?)?),
                    "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
                        let argument = required_argument(pseudo, argument)?;
                        let (expression, of) = split_nth_of(argument);
                        let (a, b) = parse_nth(expression)?;
                        let of = match of {
                            Some(source) if lower.contains("of-type") => {
                                return Err("Invalid nth expression".to_owned());
                            }
                            Some(source) => parse_selector_list(source)?,
                            None => Vec::new(),
                        };
                        selector.nth.push(NthSelector {
                            a,
                            b,
                            from_end: lower.contains("last"),
                            of_type: lower.contains("of-type"),
                            of,
                        });
                    }
                    _ => return Err(format!("Unsupported pseudo-class :{pseudo}")),
                }
            }
            _ => return Err("Invalid compound selector".to_owned()),
        }
    }
    Ok(selector)
}

fn parse_attribute(source: &str) -> Result<AttributeSelector, String> {
    let mut source = source.trim();
    if source.is_empty() {
        return Err("Empty attribute selector".to_owned());
    }
    let mut case_insensitive = false;
    if let Some((body, flag)) = source.rsplit_once(char::is_whitespace) {
        if matches!(flag.trim(), "s" | "S") {
            return Err("Unsupported attribute selector flag".to_owned());
        }
        if matches!(flag.trim(), "i" | "I") {
            case_insensitive = true;
            source = body.trim_end();
        }
    }
    let operators = [
        ("~=", AttributeOperator::Includes),
        ("|=", AttributeOperator::DashMatch),
        ("^=", AttributeOperator::Prefix),
        ("$=", AttributeOperator::Suffix),
        ("*=", AttributeOperator::Substring),
        ("=", AttributeOperator::Equals),
    ];
    for (token, operator) in operators {
        if let Some(index) = source.find(token) {
            let name = source[..index].trim();
            let value = source[index + token.len()..].trim();
            if name.is_empty() || value.is_empty() {
                return Err("Invalid attribute selector".to_owned());
            }
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                &value[1..value.len() - 1]
            } else {
                value
            };
            return Ok(AttributeSelector {
                name: name.rsplit('|').next().unwrap_or(name).to_owned(),
                operator,
                value: value.to_owned(),
                case_insensitive,
            });
        }
    }
    Ok(AttributeSelector {
        name: source.rsplit('|').next().unwrap_or(source).to_owned(),
        operator: AttributeOperator::Present,
        value: String::new(),
        case_insensitive,
    })
}

fn required_argument<'a>(pseudo: &str, argument: Option<&'a str>) -> Result<&'a str, String> {
    argument
        .filter(|argument| !argument.is_empty())
        .ok_or_else(|| format!("Pseudo-class :{pseudo} requires an argument"))
}

fn parse_nth(source: &str) -> Result<(i32, i32), String> {
    let normalized = source
        .split_ascii_whitespace()
        .collect::<String>()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "odd" => return Ok((2, 1)),
        "even" => return Ok((2, 0)),
        _ => {}
    }
    if let Some(index) = normalized.find('n') {
        let coefficient = &normalized[..index];
        let a = match coefficient {
            "" | "+" => 1,
            "-" => -1,
            value => value
                .parse::<i32>()
                .map_err(|_| "Invalid nth expression".to_owned())?,
        };
        let remainder = &normalized[index + 1..];
        let b = if remainder.is_empty() {
            0
        } else {
            remainder
                .parse::<i32>()
                .map_err(|_| "Invalid nth expression".to_owned())?
        };
        Ok((a, b))
    } else {
        normalized
            .parse::<i32>()
            .map(|value| (0, value))
            .map_err(|_| "Invalid nth expression".to_owned())
    }
}

fn split_nth_of(source: &str) -> (&str, Option<&str>) {
    let lower = source.to_ascii_lowercase();
    for marker in [" of ", "\tof ", " of\t", "\nof ", " of\n"] {
        if let Some(index) = lower.find(marker) {
            let offset = marker.find("of").unwrap_or(1);
            return (
                source[..index].trim(),
                Some(source[index + offset + 2..].trim()),
            );
        }
    }
    (source.trim(), None)
}

fn parse_forgiving_selector_list(source: &str) -> Result<Vec<ComplexSelector>, String> {
    let selectors = split_top_level(source, ',')?
        .into_iter()
        .filter_map(|selector| parse_complex(selector.trim()).ok())
        .collect::<Vec<_>>();
    if selectors.is_empty() {
        Err("Invalid forgiving selector list".to_owned())
    } else {
        Ok(selectors)
    }
}

fn parse_relative_selector_list(source: &str) -> Result<Vec<String>, String> {
    let mut output = Vec::new();
    for selector in split_top_level(source, ',')? {
        let selector = selector.trim();
        let body = selector
            .strip_prefix(['>', '+', '~'])
            .unwrap_or(selector)
            .trim();
        parse_complex(body)?;
        output.push(selector.to_owned());
    }
    Ok(output)
}

fn split_top_level(source: &str, separator: char) -> Result<Vec<&str>, String> {
    let mut output = Vec::new();
    let mut start = 0;
    let mut depth = 0_i32;
    let mut quote = None;
    for (offset, character) in source.char_indices() {
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            _ if character == separator && depth == 0 => {
                output.push(source[start..offset].trim());
                start = offset + character.len_utf8();
            }
            _ => {}
        }
        if depth < 0 {
            return Err("Unbalanced selector".to_owned());
        }
    }
    if depth != 0 || quote.is_some() {
        return Err("Unbalanced selector".to_owned());
    }
    output.push(source[start..].trim());
    if output.iter().any(|value| value.is_empty()) {
        return Err("Invalid selector list".to_owned());
    }
    Ok(output)
}

fn consume_identifier(source: &str, start: usize) -> Result<(String, usize), String> {
    let bytes = source.as_bytes();
    let mut end = start;
    let mut output = String::new();
    while end < bytes.len() {
        let byte = bytes[end];
        if byte == b'\\' {
            end += 1;
            if end >= bytes.len() {
                return Err("Invalid escape sequence".to_owned());
            }
            if bytes[end].is_ascii_hexdigit() {
                let hex_start = end;
                while end < bytes.len() && end - hex_start < 6 && bytes[end].is_ascii_hexdigit() {
                    end += 1;
                }
                let value = u32::from_str_radix(&source[hex_start..end], 16)
                    .map_err(|_| "Invalid escape sequence".to_owned())?;
                output.push(char::from_u32(value).unwrap_or('\u{fffd}'));
                if end < bytes.len() && bytes[end].is_ascii_whitespace() {
                    end += 1;
                }
                continue;
            }
            let character = source[end..]
                .chars()
                .next()
                .ok_or_else(|| "Invalid escape sequence".to_owned())?;
            output.push(character);
            end += character.len_utf8();
            continue;
        }
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'|' | b'*') {
            output.push(byte as char);
            end += 1;
            continue;
        }
        if !byte.is_ascii() {
            let character = source[end..]
                .chars()
                .next()
                .ok_or_else(|| "Invalid identifier".to_owned())?;
            output.push(character);
            end += character.len_utf8();
            continue;
        }
        break;
    }
    Ok((output, end))
}

fn is_name_start(value: u8) -> bool {
    value.is_ascii_alphabetic() || matches!(value, b'_' | b'-')
}

fn find_closing(source: &[u8], start: usize, open: u8, close: u8) -> Result<usize, String> {
    let mut depth = 0;
    let mut quote = None;
    for (index, byte) in source.iter().copied().enumerate().skip(start) {
        if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
            continue;
        }
        match byte {
            b'"' | b'\'' => quote = Some(byte),
            value if value == open => depth += 1,
            value if value == close => {
                depth -= 1;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    Err("Unbalanced selector".to_owned())
}
