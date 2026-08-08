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
    empty: bool,
    root: bool,
    checked: bool,
    disabled: bool,
    enabled: bool,
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
    let mut candidates = descendants(scope, root);
    if super::element::record(scope, root).is_some()
        && selectors
            .iter()
            .any(|selector| selector.compounds.iter().any(|compound| compound.scope))
    {
        candidates.insert(0, root);
    }
    Ok(candidates
        .into_iter()
        .filter(|candidate| {
            selectors
                .iter()
                .any(|selector| matches_complex(scope, *candidate, root, selector))
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
    if let Some(tag) = &selector.tag {
        let tag = tag.rsplit('|').next().unwrap_or(tag);
        if tag != "*" && !record.tag_name.eq_ignore_ascii_case(tag) {
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
        let matches = if attribute.case_insensitive {
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
    let is_checked = super::element::attribute_value(scope, element, "checked").is_some()
        || super::element::attribute_value(scope, element, "selected").is_some();
    if selector.checked && !is_checked {
        return false;
    }
    let is_disabled = super::element::attribute_value(scope, element, "disabled").is_some();
    if selector.disabled && !is_disabled {
        return false;
    }
    if selector.enabled && is_disabled {
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
                !nth.of_type
                    || super::element::record(scope, *sibling).is_some_and(|sibling_record| {
                        sibling_record
                            .tag_name
                            .eq_ignore_ascii_case(&record.tag_name)
                            && sibling_record.namespace_uri == record.namespace_uri
                    })
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
    if bytes
        .first()
        .is_some_and(|byte| *byte == b'*' || is_name_start(*byte))
    {
        let end = consume_name(bytes, index);
        selector.tag = Some(source[index..end].to_owned());
        index = end;
    }
    while index < bytes.len() {
        match bytes[index] {
            b'#' => {
                let end = consume_name(bytes, index + 1);
                if end == index + 1 {
                    return Err("Invalid id selector".to_owned());
                }
                selector.id = Some(source[index + 1..end].to_owned());
                index = end;
            }
            b'.' => {
                let end = consume_name(bytes, index + 1);
                if end == index + 1 {
                    return Err("Invalid class selector".to_owned());
                }
                selector.classes.push(source[index + 1..end].to_owned());
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
                let end = consume_name(bytes, index + 1);
                let pseudo = &source[index + 1..end];
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
                    "empty" => selector.empty = true,
                    "root" => selector.root = true,
                    "checked" => selector.checked = true,
                    "disabled" => selector.disabled = true,
                    "enabled" => selector.enabled = true,
                    "not" => selector
                        .not
                        .extend(parse_selector_list(required_argument(pseudo, argument)?)?),
                    "is" | "where" => selector
                        .any
                        .extend(parse_selector_list(required_argument(pseudo, argument)?)?),
                    "has" => selector.has.extend(
                        split_top_level(required_argument(pseudo, argument)?, ',')?
                            .into_iter()
                            .map(str::to_owned),
                    ),
                    "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
                        let (a, b) = parse_nth(required_argument(pseudo, argument)?)?;
                        selector.nth.push(NthSelector {
                            a,
                            b,
                            from_end: lower.contains("last"),
                            of_type: lower.contains("of-type"),
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
    if let Some((body, flag)) = source.rsplit_once(char::is_whitespace)
        && matches!(flag.trim(), "i" | "I" | "s" | "S")
    {
        case_insensitive = flag.trim().eq_ignore_ascii_case("i");
        source = body.trim_end();
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

fn consume_name(source: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < source.len()
        && (source[end].is_ascii_alphanumeric() || matches!(source[end], b'-' | b'_' | b'|' | b'*'))
    {
        end += 1;
    }
    end
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
