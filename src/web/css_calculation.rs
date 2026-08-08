#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Number,
    Length,
    Percentage,
    LengthPercentage,
    Angle,
    Time,
    Frequency,
    Resolution,
    Flex,
}

#[derive(Clone, Debug)]
enum Node {
    Literal {
        value: f64,
        unit: String,
    },
    Constant(f64),
    Size,
    Keyword(String),
    Negate(Box<Node>),
    Binary {
        operator: char,
        left: Box<Node>,
        right: Box<Node>,
    },
    Function {
        name: String,
        arguments: Vec<Node>,
    },
}

#[derive(Clone, Copy, Debug)]
struct TypedValue {
    value: f64,
    kind: Kind,
}

#[derive(Clone, Debug)]
struct SymbolicValue {
    value: f64,
    unit: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EvaluationContext {
    pub(crate) viewport_width: f64,
    pub(crate) viewport_height: f64,
    pub(crate) percentage_basis: Option<f64>,
    pub(crate) font_size: f64,
    pub(crate) root_font_size: f64,
    pub(crate) intrinsic_size: Option<f64>,
}

impl EvaluationContext {
    fn constants_only() -> Self {
        Self {
            viewport_width: f64::NAN,
            viewport_height: f64::NAN,
            percentage_basis: None,
            font_size: f64::NAN,
            root_font_size: f64::NAN,
            intrinsic_size: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64, String),
    Ident(String),
    Percentage(f64, String),
    Dimension(f64, String, String),
    Plus(bool),
    Minus(bool),
    Star,
    Slash,
    LeftParen,
    RightParen,
    Comma,
}

pub(crate) fn contains_math(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "calc(",
        "calc-size(",
        "min(",
        "max(",
        "clamp(",
        "round(",
        "mod(",
        "rem(",
        "progress(",
        "sin(",
        "cos(",
        "tan(",
        "asin(",
        "acos(",
        "atan(",
        "atan2(",
        "pow(",
        "sqrt(",
        "hypot(",
        "log(",
        "exp(",
        "abs(",
        "sign(",
    ]
    .iter()
    .any(|function| lower.contains(function))
}

pub(crate) fn is_root_numeric_math(value: &str) -> bool {
    parse(value.trim()).is_ok()
}

pub(crate) fn normalize_property_value(name: &str, value: &str) -> Option<String> {
    let value = value.trim();
    if is_color_property(name) {
        return normalize_color_value(value);
    }
    if value.is_empty() || name.starts_with("--") || !contains_math(value) {
        return Some(value.to_owned());
    }
    if value.to_ascii_lowercase().contains("var(") || value.to_ascii_lowercase().contains("env(") {
        return Some(value.to_owned());
    }
    if let Ok(node) = parse(value) {
        let kind = infer(&node).ok()?;
        if property_kind_allowed(name, kind) == Some(false) {
            return None;
        }
        return Some(normalize_node(&node));
    }
    if property_has_numeric_constraint(name) {
        return None;
    }
    normalize_embedded_math(value)
}

fn is_color_property(name: &str) -> bool {
    name == "color"
        || name.ends_with("-color")
        || matches!(
            name,
            "fill" | "stroke" | "flood-color" | "lighting-color" | "stop-color"
        )
}

fn canonical_system_color(value: &str) -> Option<&'static str> {
    let value = value.trim();
    SYSTEM_COLORS
        .iter()
        .find(|(name, _)| value.eq_ignore_ascii_case(name))
        .map(|(name, _)| *name)
}

fn normalize_color_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return Some(String::new());
    }
    if let Some(value) = canonical_system_color(value) {
        return Some(value.to_owned());
    }
    let lower = value.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "transparent"
            | "currentcolor"
            | "inherit"
            | "initial"
            | "unset"
            | "revert"
            | "revert-layer"
    ) || named_color_hex(&lower).is_some()
    {
        return Some(lower);
    }
    if let Some(hex) = lower.strip_prefix('#') {
        if matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return computed_hex_color(hex);
        }
        return None;
    }
    valid_color_function(&lower)
        .then(|| canonical_color_function(&lower))
        .flatten()
}

#[derive(Clone, Copy)]
struct SrgbColor {
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64,
}

fn canonical_color_function(value: &str) -> Option<String> {
    let open = value.find('(')?;
    let name = &value[..open];
    let body = &value[open + 1..value.len() - 1];
    match name {
        "rgb" | "rgba" => parse_rgb_color(name, body).map(serialize_srgb),
        "hsl" | "hsla" => parse_hsl_color(name, body).map(serialize_srgb),
        "hwb" => parse_hwb_color(body).map(serialize_srgb),
        "lab" | "lch" | "oklab" | "oklch" | "color" => {
            Some(canonical_absolute_function(name, body))
        }
        "color-mix" | "light-dark" | "var" => Some(value.to_owned()),
        _ => None,
    }
}

fn parse_rgb_color(name: &str, body: &str) -> Option<SrgbColor> {
    let commas = split_top_level(body, ',');
    let (channels, alpha) = if commas.len() > 1 {
        let expected = if name == "rgba" { 4 } else { 3 };
        if commas.len() != expected {
            return None;
        }
        (&commas[..3], commas.get(3).copied())
    } else {
        let slash = split_top_level(body, '/');
        if slash.len() > 2 {
            return None;
        }
        let channels = split_ascii_whitespace_top_level(slash[0]);
        if channels.len() != 3 {
            return None;
        }
        return Some(SrgbColor {
            red: parse_rgb_channel(channels[0])?,
            green: parse_rgb_channel(channels[1])?,
            blue: parse_rgb_channel(channels[2])?,
            alpha: slash.get(1).map_or(Some(1.0), |value| parse_alpha(value))?,
        });
    };
    Some(SrgbColor {
        red: parse_rgb_channel(channels[0])?,
        green: parse_rgb_channel(channels[1])?,
        blue: parse_rgb_channel(channels[2])?,
        alpha: alpha.map_or(Some(1.0), parse_alpha)?,
    })
}

fn parse_hsl_color(name: &str, body: &str) -> Option<SrgbColor> {
    let commas = split_top_level(body, ',');
    let (channels, alpha) = if commas.len() > 1 {
        let expected = if name == "hsla" { 4 } else { 3 };
        if commas.len() != expected {
            return None;
        }
        (commas[..3].to_vec(), commas.get(3).copied())
    } else {
        let slash = split_top_level(body, '/');
        if slash.len() > 2 {
            return None;
        }
        let channels = split_ascii_whitespace_top_level(slash[0]);
        if channels.len() != 3 {
            return None;
        }
        (channels, slash.get(1).copied())
    };
    let hue = parse_hue(channels[0])?;
    let saturation = parse_percentage(channels[1])?.clamp(0.0, 1.0);
    let lightness = parse_percentage(channels[2])?.clamp(0.0, 1.0);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = (hue.rem_euclid(360.0)) / 60.0;
    let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match sector.floor() as i32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let offset = lightness - chroma / 2.0;
    Some(SrgbColor {
        red: (red + offset) * 255.0,
        green: (green + offset) * 255.0,
        blue: (blue + offset) * 255.0,
        alpha: alpha.map_or(Some(1.0), parse_alpha)?,
    })
}

fn parse_hwb_color(body: &str) -> Option<SrgbColor> {
    let slash = split_top_level(body, '/');
    if slash.len() > 2 {
        return None;
    }
    let channels = split_ascii_whitespace_top_level(slash[0]);
    if channels.len() != 3 {
        return None;
    }
    let hue = parse_hue(channels[0])?;
    let mut white = parse_percentage(channels[1])?.max(0.0);
    let mut black = parse_percentage(channels[2])?.max(0.0);
    if white + black >= 1.0 {
        let gray = white / (white + black);
        return Some(SrgbColor {
            red: gray * 255.0,
            green: gray * 255.0,
            blue: gray * 255.0,
            alpha: slash.get(1).map_or(Some(1.0), |value| parse_alpha(value))?,
        });
    }
    white = white.clamp(0.0, 1.0);
    black = black.clamp(0.0, 1.0);
    let pure = parse_hsl_color("hsl", &format!("{hue} 100% 50%"))?;
    let factor = 1.0 - white - black;
    Some(SrgbColor {
        red: (pure.red / 255.0 * factor + white) * 255.0,
        green: (pure.green / 255.0 * factor + white) * 255.0,
        blue: (pure.blue / 255.0 * factor + white) * 255.0,
        alpha: slash.get(1).map_or(Some(1.0), |value| parse_alpha(value))?,
    })
}

fn parse_rgb_channel(value: &str) -> Option<f64> {
    let value = value.trim();
    if value == "none" {
        return Some(0.0);
    }
    if let Some(value) = value.strip_suffix('%') {
        return Some(value.parse::<f64>().ok()?.clamp(0.0, 100.0) * 2.55);
    }
    Some(value.parse::<f64>().ok()?.clamp(0.0, 255.0))
}

fn parse_alpha(value: &str) -> Option<f64> {
    let value = value.trim();
    if value == "none" {
        return Some(0.0);
    }
    if let Some(value) = value.strip_suffix('%') {
        return Some((value.parse::<f64>().ok()? / 100.0).clamp(0.0, 1.0));
    }
    Some(value.parse::<f64>().ok()?.clamp(0.0, 1.0))
}

fn parse_percentage(value: &str) -> Option<f64> {
    Some(value.trim().strip_suffix('%')?.parse::<f64>().ok()? / 100.0)
}

fn parse_hue(value: &str) -> Option<f64> {
    let value = value.trim();
    if value == "none" {
        return Some(0.0);
    }
    for (suffix, scale) in [
        ("deg", 1.0),
        ("grad", 0.9),
        ("rad", 180.0 / std::f64::consts::PI),
        ("turn", 360.0),
    ] {
        if let Some(number) = value.strip_suffix(suffix) {
            return Some(number.parse::<f64>().ok()? * scale);
        }
    }
    value.parse::<f64>().ok()
}

fn serialize_srgb(color: SrgbColor) -> String {
    let red = color.red.clamp(0.0, 255.0).round() as u8;
    let green = color.green.clamp(0.0, 255.0).round() as u8;
    let blue = color.blue.clamp(0.0, 255.0).round() as u8;
    if color.alpha < 1.0 {
        let alpha = (color.alpha.clamp(0.0, 1.0) * 1_000.0).round() / 1_000.0;
        format!("rgba({red}, {green}, {blue}, {})", format_number(alpha))
    } else {
        format!("rgb({red}, {green}, {blue})")
    }
}

fn canonical_absolute_function(name: &str, body: &str) -> String {
    let slash = split_top_level(body, '/');
    let mut components = split_ascii_whitespace_top_level(slash[0])
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if matches!(name, "lab" | "lch") {
        if let Some(lightness) = components.first_mut() {
            if let Some(number) = lightness.strip_suffix('%') {
                *lightness = number.to_owned();
            }
        }
    }
    let mut output = format!("{name}({})", components.join(" "));
    if let Some(alpha) = slash.get(1) {
        output.pop();
        output.push_str(" / ");
        output.push_str(alpha.trim());
        output.push(')');
    }
    output
}

fn valid_color_function(value: &str) -> bool {
    let Some(open) = value.find('(') else {
        return false;
    };
    let Some(close) = matching_parenthesis(value, open) else {
        return false;
    };
    if close + 1 != value.len() {
        return false;
    }
    let name = value[..open].trim();
    let body = value[open + 1..close].trim();
    if body.is_empty() {
        return false;
    }
    match name {
        "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch" => {
            valid_component_color_function(name, body)
        }
        "color" => valid_color_space_function(body),
        "color-mix" => valid_color_mix_function(body),
        "light-dark" => split_top_level(body, ',').len() == 2,
        "var" => body.starts_with("--"),
        _ => false,
    }
}

fn valid_component_color_function(name: &str, body: &str) -> bool {
    let comma_parts = split_top_level(body, ',');
    if comma_parts.len() > 1 {
        let expected = if matches!(name, "rgba" | "hsla") {
            4
        } else {
            3
        };
        return comma_parts.len() == expected
            && comma_parts.iter().all(|part| valid_color_component(part));
    }
    let slash_parts = split_top_level(body, '/');
    if slash_parts.len() > 2 || slash_parts.iter().any(|part| part.trim().is_empty()) {
        return false;
    }
    let components = split_ascii_whitespace_top_level(slash_parts[0]);
    components.len() == 3
        && components.iter().all(|part| valid_color_component(part))
        && slash_parts
            .get(1)
            .is_none_or(|alpha| valid_color_component(alpha))
}

fn valid_color_space_function(body: &str) -> bool {
    let slash_parts = split_top_level(body, '/');
    if slash_parts.len() > 2 {
        return false;
    }
    let components = split_ascii_whitespace_top_level(slash_parts[0]);
    let Some(space) = components.first().copied() else {
        return false;
    };
    let known_space = matches!(
        space,
        "srgb"
            | "srgb-linear"
            | "display-p3"
            | "a98-rgb"
            | "prophoto-rgb"
            | "rec2020"
            | "xyz"
            | "xyz-d50"
            | "xyz-d65"
    );
    known_space
        && components.len() == 4
        && components[1..]
            .iter()
            .all(|part| valid_color_component(part))
        && slash_parts
            .get(1)
            .is_none_or(|alpha| valid_color_component(alpha))
}

fn valid_color_mix_function(body: &str) -> bool {
    let parts = split_top_level(body, ',');
    parts.len() == 3
        && parts[0].trim_start().starts_with("in ")
        && parts[1..].iter().all(|part| !part.trim().is_empty())
}

fn valid_color_component(value: &str) -> bool {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") || value.starts_with("var(") || value.starts_with("calc(")
    {
        return true;
    }
    let number = value
        .strip_suffix('%')
        .or_else(|| value.strip_suffix("deg"))
        .or_else(|| value.strip_suffix("grad"))
        .or_else(|| value.strip_suffix("rad"))
        .or_else(|| value.strip_suffix("turn"))
        .unwrap_or(value);
    !number.is_empty() && number.parse::<f64>().is_ok_and(f64::is_finite)
}

fn split_top_level(value: &str, delimiter: char) -> Vec<&str> {
    let mut output = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if character == delimiter && depth == 0 => {
                output.push(&value[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    output.push(&value[start..]);
    output
}

fn split_ascii_whitespace_top_level(value: &str) -> Vec<&str> {
    let mut output = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, character) in value.char_indices() {
        if character == '(' {
            depth += 1;
        } else if character == ')' {
            depth = depth.saturating_sub(1);
        }
        if character.is_ascii_whitespace() && depth == 0 {
            if let Some(begin) = start.take() {
                output.push(&value[begin..index]);
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(begin) = start {
        output.push(&value[begin..]);
    }
    output
}

pub(crate) fn computed_system_color(name: &str, value: &str) -> Option<&'static str> {
    if !is_color_property(name) {
        return None;
    }
    let value = canonical_system_color(value)?;
    SYSTEM_COLORS
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, computed)| *computed)
}

pub(crate) fn computed_color(name: &str, value: &str) -> Option<String> {
    if !is_color_property(name) {
        return None;
    }
    if let Some(value) = computed_system_color(name, value) {
        return Some(value.to_owned());
    }
    let lower = value.trim().to_ascii_lowercase();
    if lower == "transparent" {
        return Some("rgba(0, 0, 0, 0)".to_owned());
    }
    let hex = lower
        .strip_prefix('#')
        .or_else(|| named_color_hex(&lower))?;
    computed_hex_color(hex)
}

fn computed_hex_color(hex: &str) -> Option<String> {
    let expanded;
    let hex = match hex.len() {
        3 | 4 => {
            expanded = hex
                .chars()
                .flat_map(|character| [character, character])
                .collect::<String>();
            expanded.as_str()
        }
        6 | 8 => hex,
        _ => return None,
    };
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    if hex.len() == 8 {
        let alpha = u8::from_str_radix(&hex[6..8], 16).ok()?;
        if alpha < 255 {
            return Some(format!(
                "rgba({red}, {green}, {blue}, {})",
                format_number(f64::from(alpha) / 255.0)
            ));
        }
    }
    Some(format!("rgb({red}, {green}, {blue})"))
}

// Microsoft Edge/Chromium resolves CSS system colors to used RGB values in
// getComputedStyle().  The deprecated CSS2 names remain accepted for web
// compatibility; their specified serialization is the lowercase keyword.
// These values are the normal (non-forced-colors) Edge palette.
const SYSTEM_COLORS: &[(&str, &str)] = &[
    ("activeborder", "rgb(0, 0, 0)"),
    ("activecaption", "rgb(255, 255, 255)"),
    ("activetext", "rgb(0, 102, 204)"),
    ("appworkspace", "rgb(255, 255, 255)"),
    ("background", "rgb(255, 255, 255)"),
    ("buttonborder", "rgb(0, 0, 0)"),
    ("buttonface", "rgb(240, 240, 240)"),
    ("buttonhighlight", "rgb(240, 240, 240)"),
    ("buttonshadow", "rgb(240, 240, 240)"),
    ("buttontext", "rgb(0, 0, 0)"),
    ("canvas", "rgb(255, 255, 255)"),
    ("canvastext", "rgb(0, 0, 0)"),
    ("captiontext", "rgb(0, 0, 0)"),
    ("field", "rgb(255, 255, 255)"),
    ("fieldtext", "rgb(0, 0, 0)"),
    ("graytext", "rgb(109, 109, 109)"),
    ("highlight", "rgb(0, 120, 215)"),
    ("highlighttext", "rgb(255, 255, 255)"),
    ("inactiveborder", "rgb(0, 0, 0)"),
    ("inactivecaption", "rgb(255, 255, 255)"),
    ("inactivecaptiontext", "rgb(128, 128, 128)"),
    ("infobackground", "rgb(255, 255, 255)"),
    ("infotext", "rgb(0, 0, 0)"),
    ("linktext", "rgb(0, 102, 204)"),
    ("mark", "rgb(255, 255, 0)"),
    ("marktext", "rgb(0, 0, 0)"),
    ("menu", "rgb(255, 255, 255)"),
    ("menutext", "rgb(0, 0, 0)"),
    ("scrollbar", "rgb(255, 255, 255)"),
    ("selecteditem", "rgb(25, 103, 210)"),
    ("selecteditemtext", "rgb(255, 255, 255)"),
    ("threeddarkshadow", "rgb(0, 0, 0)"),
    ("threedface", "rgb(240, 240, 240)"),
    ("threedhighlight", "rgb(0, 0, 0)"),
    ("threedlightshadow", "rgb(0, 0, 0)"),
    ("threedshadow", "rgb(0, 0, 0)"),
    ("visitedtext", "rgb(0, 102, 204)"),
    ("window", "rgb(255, 255, 255)"),
    ("windowframe", "rgb(0, 0, 0)"),
    ("windowtext", "rgb(0, 0, 0)"),
    ("accentcolor", "rgb(0, 117, 255)"),
    ("accentcolortext", "rgb(255, 255, 255)"),
];

fn named_color_hex(value: &str) -> Option<&'static str> {
    NAMED_COLORS
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, hex)| *hex)
}

const NAMED_COLORS: &[(&str, &str)] = &[
    ("aliceblue", "F0F8FF"),
    ("antiquewhite", "FAEBD7"),
    ("aqua", "00FFFF"),
    ("aquamarine", "7FFFD4"),
    ("azure", "F0FFFF"),
    ("beige", "F5F5DC"),
    ("bisque", "FFE4C4"),
    ("black", "000000"),
    ("blanchedalmond", "FFEBCD"),
    ("blue", "0000FF"),
    ("blueviolet", "8A2BE2"),
    ("brown", "A52A2A"),
    ("burlywood", "DEB887"),
    ("cadetblue", "5F9EA0"),
    ("chartreuse", "7FFF00"),
    ("chocolate", "D2691E"),
    ("coral", "FF7F50"),
    ("cornflowerblue", "6495ED"),
    ("cornsilk", "FFF8DC"),
    ("crimson", "DC143C"),
    ("cyan", "00FFFF"),
    ("darkblue", "00008B"),
    ("darkcyan", "008B8B"),
    ("darkgoldenrod", "B8860B"),
    ("darkgray", "A9A9A9"),
    ("darkgreen", "006400"),
    ("darkgrey", "A9A9A9"),
    ("darkkhaki", "BDB76B"),
    ("darkmagenta", "8B008B"),
    ("darkolivegreen", "556B2F"),
    ("darkorange", "FF8C00"),
    ("darkorchid", "9932CC"),
    ("darkred", "8B0000"),
    ("darksalmon", "E9967A"),
    ("darkseagreen", "8FBC8F"),
    ("darkslateblue", "483D8B"),
    ("darkslategray", "2F4F4F"),
    ("darkslategrey", "2F4F4F"),
    ("darkturquoise", "00CED1"),
    ("darkviolet", "9400D3"),
    ("deeppink", "FF1493"),
    ("deepskyblue", "00BFFF"),
    ("dimgray", "696969"),
    ("dimgrey", "696969"),
    ("dodgerblue", "1E90FF"),
    ("firebrick", "B22222"),
    ("floralwhite", "FFFAF0"),
    ("forestgreen", "228B22"),
    ("fuchsia", "FF00FF"),
    ("gainsboro", "DCDCDC"),
    ("ghostwhite", "F8F8FF"),
    ("gold", "FFD700"),
    ("goldenrod", "DAA520"),
    ("gray", "808080"),
    ("green", "008000"),
    ("greenyellow", "ADFF2F"),
    ("grey", "808080"),
    ("honeydew", "F0FFF0"),
    ("hotpink", "FF69B4"),
    ("indianred", "CD5C5C"),
    ("indigo", "4B0082"),
    ("ivory", "FFFFF0"),
    ("khaki", "F0E68C"),
    ("lavender", "E6E6FA"),
    ("lavenderblush", "FFF0F5"),
    ("lawngreen", "7CFC00"),
    ("lemonchiffon", "FFFACD"),
    ("lightblue", "ADD8E6"),
    ("lightcoral", "F08080"),
    ("lightcyan", "E0FFFF"),
    ("lightgoldenrodyellow", "FAFAD2"),
    ("lightgray", "D3D3D3"),
    ("lightgreen", "90EE90"),
    ("lightgrey", "D3D3D3"),
    ("lightpink", "FFB6C1"),
    ("lightsalmon", "FFA07A"),
    ("lightseagreen", "20B2AA"),
    ("lightskyblue", "87CEFA"),
    ("lightslategray", "778899"),
    ("lightslategrey", "778899"),
    ("lightsteelblue", "B0C4DE"),
    ("lightyellow", "FFFFE0"),
    ("lime", "00FF00"),
    ("limegreen", "32CD32"),
    ("linen", "FAF0E6"),
    ("magenta", "FF00FF"),
    ("maroon", "800000"),
    ("mediumaquamarine", "66CDAA"),
    ("mediumblue", "0000CD"),
    ("mediumorchid", "BA55D3"),
    ("mediumpurple", "9370DB"),
    ("mediumseagreen", "3CB371"),
    ("mediumslateblue", "7B68EE"),
    ("mediumspringgreen", "00FA9A"),
    ("mediumturquoise", "48D1CC"),
    ("mediumvioletred", "C71585"),
    ("midnightblue", "191970"),
    ("mintcream", "F5FFFA"),
    ("mistyrose", "FFE4E1"),
    ("moccasin", "FFE4B5"),
    ("navajowhite", "FFDEAD"),
    ("navy", "000080"),
    ("oldlace", "FDF5E6"),
    ("olive", "808000"),
    ("olivedrab", "6B8E23"),
    ("orange", "FFA500"),
    ("orangered", "FF4500"),
    ("orchid", "DA70D6"),
    ("palegoldenrod", "EEE8AA"),
    ("palegreen", "98FB98"),
    ("paleturquoise", "AFEEEE"),
    ("palevioletred", "DB7093"),
    ("papayawhip", "FFEFD5"),
    ("peachpuff", "FFDAB9"),
    ("peru", "CD853F"),
    ("pink", "FFC0CB"),
    ("plum", "DDA0DD"),
    ("powderblue", "B0E0E6"),
    ("purple", "800080"),
    ("rebeccapurple", "663399"),
    ("red", "FF0000"),
    ("rosybrown", "BC8F8F"),
    ("royalblue", "4169E1"),
    ("saddlebrown", "8B4513"),
    ("salmon", "FA8072"),
    ("sandybrown", "F4A460"),
    ("seagreen", "2E8B57"),
    ("seashell", "FFF5EE"),
    ("sienna", "A0522D"),
    ("silver", "C0C0C0"),
    ("skyblue", "87CEEB"),
    ("slateblue", "6A5ACD"),
    ("slategray", "708090"),
    ("slategrey", "708090"),
    ("snow", "FFFAFA"),
    ("springgreen", "00FF7F"),
    ("steelblue", "4682B4"),
    ("tan", "D2B48C"),
    ("teal", "008080"),
    ("thistle", "D8BFD8"),
    ("tomato", "FF6347"),
    ("turquoise", "40E0D0"),
    ("violet", "EE82EE"),
    ("wheat", "F5DEB3"),
    ("white", "FFFFFF"),
    ("whitesmoke", "F5F5F5"),
    ("yellow", "FFFF00"),
    ("yellowgreen", "9ACD32"),
];

fn normalize_node(node: &Node) -> String {
    if let Ok(result) = evaluate(&node, EvaluationContext::constants_only()) {
        if result.value.is_finite() {
            return format!(
                "calc({}{})",
                format_number(result.value),
                canonical_suffix(result.kind)
            );
        }
        let constant = if result.value.is_nan() {
            "NaN"
        } else if result.value.is_sign_negative() {
            "-infinity"
        } else {
            "infinity"
        };
        let suffix = canonical_suffix(result.kind);
        return if suffix.is_empty() {
            format!("calc({constant})")
        } else {
            format!("calc({constant} * 1{suffix})")
        };
    }
    if let Ok(result) = evaluate_symbolic(node) {
        if result.value.is_finite() {
            return format!("calc({}{})", format_number(result.value), result.unit);
        }
    }
    serialize_root(node)
}

pub(crate) fn normalize_numeric_value(value: &str) -> Option<String> {
    let node = parse(value.trim()).ok()?;
    infer(&node).ok()?;
    Some(normalize_node(&node))
}

fn normalize_embedded_math(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    let mut found = false;
    while cursor < bytes.len() {
        let Some((start, open)) = next_math_function(value, cursor) else {
            output.push_str(&value[cursor..]);
            break;
        };
        output.push_str(&value[cursor..start]);
        let end = matching_parenthesis(value, open)?;
        let node = parse(&value[start..=end]).ok()?;
        infer(&node).ok()?;
        output.push_str(&normalize_node(&node));
        cursor = end + 1;
        found = true;
    }
    found.then_some(output)
}

fn next_math_function(value: &str, from: usize) -> Option<(usize, usize)> {
    let bytes = value.as_bytes();
    let mut cursor = from;
    while cursor < bytes.len() {
        if bytes[cursor].is_ascii_alphabetic()
            && (cursor == 0
                || !matches!(bytes[cursor - 1], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-'))
        {
            let start = cursor;
            cursor += 1;
            while bytes
                .get(cursor)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'))
            {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b'(')
                && is_math_function(&value[start..cursor].to_ascii_lowercase())
            {
                return Some((start, cursor));
            }
        } else {
            cursor += 1;
        }
    }
    None
}

fn matching_parenthesis(value: &str, open: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (offset, byte) in bytes.iter().copied().enumerate().skip(open) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if let Some(current) = quote {
            if byte == current {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(offset);
            }
        }
    }
    None
}

fn is_math_function(name: &str) -> bool {
    matches!(
        name,
        "calc"
            | "calc-size"
            | "min"
            | "max"
            | "clamp"
            | "round"
            | "mod"
            | "rem"
            | "progress"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "atan2"
            | "pow"
            | "sqrt"
            | "hypot"
            | "log"
            | "exp"
            | "abs"
            | "sign"
    )
}

pub(crate) fn supports_property(name: &str, value: &str) -> bool {
    if name.trim().is_empty() || value.trim().is_empty() {
        return false;
    }
    if is_color_property(&name.trim().to_ascii_lowercase()) {
        return normalize_color_value(value).is_some_and(|value| !value.is_empty());
    }
    if !contains_math(value) {
        return true;
    }
    normalize_property_value(name, value).is_some()
}

pub(crate) fn resolve_length(value: &str, context: EvaluationContext) -> Option<f64> {
    let node = parse(value).ok()?;
    let kind = infer(&node).ok()?;
    if !kind_allowed(kind, Kind::LengthPercentage) {
        return None;
    }
    let result = evaluate(&node, context).ok()?;
    (result.kind == Kind::Length).then(|| layout_unit(result.value))
}

pub(crate) fn resolve_line_height(value: &str, context: EvaluationContext) -> Option<f64> {
    let node = parse(value).ok()?;
    let kind = infer(&node).ok()?;
    if !matches!(
        kind,
        Kind::Number | Kind::Length | Kind::Percentage | Kind::LengthPercentage
    ) {
        return None;
    }
    let result = evaluate(&node, context).ok()?;
    let value = match result.kind {
        Kind::Number => result.value * context.font_size,
        Kind::Length => result.value,
        _ => return None,
    };
    Some(layout_unit(value))
}

pub(crate) fn computed_absolute_length(value: &str) -> Option<String> {
    let node = parse(value).ok()?;
    let result = evaluate(&node, EvaluationContext::constants_only()).ok()?;
    (result.kind == Kind::Length && result.value.is_finite())
        .then(|| format!("{}px", format_number(result.value.max(0.0))))
}

pub(crate) fn is_length_property(name: &str) -> bool {
    expected_kind(name) == Some(Kind::LengthPercentage) || name.eq_ignore_ascii_case("line-height")
}

pub(crate) fn needs_computed_length_resolution(name: &str, value: &str) -> bool {
    if matches!(name, "font-size" | "line-height") {
        return true;
    }
    let value = value.trim();
    if value == "0"
        || value
            .strip_suffix("px")
            .is_some_and(|number| number.trim().parse::<f64>().is_ok())
    {
        return false;
    }
    let Ok(node) = parse(value) else {
        return false;
    };
    infer(&node).is_ok_and(|kind| kind_allowed(kind, Kind::LengthPercentage))
}

pub(crate) fn serialize_computed_length(name: &str, value: f64) -> String {
    let value = if nonnegative_length_property(name) {
        value.max(0.0)
    } else {
        value
    };
    format!("{}px", format_number(value))
}

pub(crate) fn computed_non_length(name: &str, value: &str) -> Option<String> {
    let name = name.trim().to_ascii_lowercase();
    let expected = expected_kind(&name);
    if expected == Some(Kind::LengthPercentage)
        || expected.is_none() && !matches!(name.as_str(), "opacity" | "scale")
    {
        return None;
    }
    let node = parse(value).ok()?;
    let result = evaluate(&node, EvaluationContext::constants_only()).ok()?;
    if matches!(name.as_str(), "opacity" | "scale") {
        let mut number = match result.kind {
            Kind::Number => result.value,
            Kind::Percentage => result.value / 100.0,
            _ => return None,
        };
        if name == "opacity" {
            number = number.clamp(0.0, 1.0);
        }
        return number.is_finite().then(|| format_number(number));
    }
    (Some(result.kind) == expected && result.value.is_finite()).then(|| {
        format!(
            "{}{}",
            format_number(result.value),
            canonical_suffix(result.kind)
        )
    })
}

pub(crate) fn layout_unit(value: f64) -> f64 {
    const MAX: f64 = (i32::MAX as f64) / 64.0;
    if value.is_nan() {
        0.0
    } else {
        (value.clamp(-MAX, MAX) * 64.0).floor() / 64.0
    }
}

fn nonnegative_length_property(name: &str) -> bool {
    matches!(
        name,
        "width"
            | "height"
            | "min-width"
            | "min-height"
            | "max-width"
            | "max-height"
            | "padding-top"
            | "padding-right"
            | "padding-bottom"
            | "padding-left"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "border-width"
            | "border-top-width"
            | "border-right-width"
            | "border-bottom-width"
            | "border-left-width"
            | "outline-width"
            | "font-size"
            | "line-height"
            | "perspective"
    )
}

fn expected_kind(name: &str) -> Option<Kind> {
    let name = name.trim().to_ascii_lowercase();
    if matches!(
        name.as_str(),
        "width"
            | "height"
            | "min-width"
            | "min-height"
            | "max-width"
            | "max-height"
            | "top"
            | "right"
            | "bottom"
            | "left"
            | "margin-top"
            | "margin-right"
            | "margin-bottom"
            | "margin-left"
            | "padding-top"
            | "padding-right"
            | "padding-bottom"
            | "padding-left"
            | "row-gap"
            | "column-gap"
            | "flex-basis"
            | "font-size"
            | "letter-spacing"
            | "word-spacing"
            | "text-indent"
            | "border-top-width"
            | "border-right-width"
            | "border-bottom-width"
            | "border-left-width"
            | "outline-width"
            | "perspective"
    ) {
        Some(Kind::LengthPercentage)
    } else if matches!(name.as_str(), "rotate" | "offset-rotate") {
        Some(Kind::Angle)
    } else if matches!(
        name.as_str(),
        "animation-duration" | "animation-delay" | "transition-duration" | "transition-delay"
    ) {
        Some(Kind::Time)
    } else if matches!(
        name.as_str(),
        "opacity" | "order" | "z-index" | "flex-grow" | "flex-shrink" | "scale"
    ) {
        Some(Kind::Number)
    } else {
        None
    }
}

fn property_kind_allowed(name: &str, actual: Kind) -> Option<bool> {
    let name = name.trim().to_ascii_lowercase();
    if name == "line-height" {
        Some(matches!(
            actual,
            Kind::Number | Kind::Length | Kind::Percentage | Kind::LengthPercentage
        ))
    } else if matches!(name.as_str(), "opacity" | "scale") {
        Some(matches!(actual, Kind::Number | Kind::Percentage))
    } else {
        expected_kind(&name).map(|expected| kind_allowed(actual, expected))
    }
}

fn property_has_numeric_constraint(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase();
    expected_kind(&name).is_some() || matches!(name.as_str(), "line-height" | "opacity" | "scale")
}

fn kind_allowed(actual: Kind, expected: Kind) -> bool {
    actual == expected
        || expected == Kind::LengthPercentage
            && matches!(
                actual,
                Kind::Length | Kind::Percentage | Kind::LengthPercentage
            )
}

fn canonical_suffix(kind: Kind) -> &'static str {
    match kind {
        Kind::Number => "",
        Kind::Length => "px",
        Kind::Percentage => "%",
        Kind::Angle => "deg",
        Kind::Time => "s",
        Kind::Frequency => "hz",
        Kind::Resolution => "dppx",
        Kind::Flex => "fr",
        Kind::LengthPercentage => "",
    }
}

fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let absolute = value.abs();
    if absolute >= 1_000_000.0 || absolute < 0.000001 {
        let scientific = format!("{value:.5e}");
        let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
        let exponent = exponent.parse::<i32>().unwrap_or(0);
        return format!("{mantissa}e{exponent:+03}");
    }
    let digits_before_decimal = absolute.log10().floor() as i32 + 1;
    let decimals = (6 - digits_before_decimal).clamp(0, 12) as usize;
    let mut output = format!("{value:.decimals$}");
    if output.contains('.') {
        while output.ends_with('0') {
            output.pop();
        }
        if output.ends_with('.') {
            output.pop();
        }
    }
    if output == "-0" {
        "0".to_owned()
    } else {
        output
    }
}

fn parse(input: &str) -> Result<Node, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, offset: 0 };
    let node = parser.sum()?;
    if parser.offset != parser.tokens.len() {
        return Err("unexpected CSS math token".to_owned());
    }
    Ok(node)
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let bytes = input.as_bytes();
    let mut offset = 0;
    let mut tokens = Vec::new();
    let mut separated_before = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if byte.is_ascii_whitespace() {
            offset += 1;
            separated_before = true;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            let Some(relative_end) = input[offset + 2..].find("*/") else {
                return Err("unclosed CSS comment".to_owned());
            };
            offset += relative_end + 4;
            separated_before = true;
            continue;
        }
        let single = match byte {
            b'+' => Some(Token::Plus(
                separated_before && separated_after_operator(input, offset + 1),
            )),
            b'-' => Some(Token::Minus(
                separated_before && separated_after_operator(input, offset + 1),
            )),
            b'*' => Some(Token::Star),
            b'/' => Some(Token::Slash),
            b'(' => Some(Token::LeftParen),
            b')' => Some(Token::RightParen),
            b',' => Some(Token::Comma),
            _ => None,
        };
        if let Some(token) = single {
            tokens.push(token);
            offset += 1;
            separated_before = false;
            continue;
        }
        if byte.is_ascii_digit()
            || byte == b'.' && bytes.get(offset + 1).is_some_and(u8::is_ascii_digit)
        {
            let start = offset;
            if byte == b'.' {
                offset += 1;
            }
            while bytes.get(offset).is_some_and(u8::is_ascii_digit) {
                offset += 1;
            }
            if bytes.get(offset) == Some(&b'.') {
                offset += 1;
                while bytes.get(offset).is_some_and(u8::is_ascii_digit) {
                    offset += 1;
                }
            }
            if matches!(bytes.get(offset), Some(b'e' | b'E')) {
                let exponent = offset;
                offset += 1;
                if matches!(bytes.get(offset), Some(b'+' | b'-')) {
                    offset += 1;
                }
                let digits = offset;
                while bytes.get(offset).is_some_and(u8::is_ascii_digit) {
                    offset += 1;
                }
                if digits == offset {
                    offset = exponent;
                }
            }
            let text = &input[start..offset];
            let value = text.parse::<f64>().map_err(|_| "invalid CSS number")?;
            if bytes.get(offset) == Some(&b'%') {
                offset += 1;
                tokens.push(Token::Percentage(value, text.to_owned()));
                separated_before = false;
                continue;
            }
            let unit_start = offset;
            while bytes
                .get(offset)
                .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'-')
            {
                offset += 1;
            }
            if unit_start != offset {
                tokens.push(Token::Dimension(
                    value,
                    text.to_owned(),
                    input[unit_start..offset].to_ascii_lowercase(),
                ));
            } else {
                tokens.push(Token::Number(value, text.to_owned()));
            }
            separated_before = false;
            continue;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = offset;
            offset += 1;
            while bytes
                .get(offset)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_'))
            {
                offset += 1;
            }
            tokens.push(Token::Ident(input[start..offset].to_ascii_lowercase()));
            separated_before = false;
            continue;
        }
        return Err("invalid CSS math character".to_owned());
    }
    Ok(tokens)
}

fn separated_after_operator(input: &str, mut offset: usize) -> bool {
    let bytes = input.as_bytes();
    let mut separated = false;
    while offset < bytes.len() {
        if bytes[offset].is_ascii_whitespace() {
            separated = true;
            offset += 1;
        } else if bytes.get(offset..offset + 2) == Some(b"/*") {
            let Some(relative_end) = input[offset + 2..].find("*/") else {
                return false;
            };
            separated = true;
            offset += relative_end + 4;
        } else {
            break;
        }
    }
    separated
}

struct Parser {
    tokens: Vec<Token>,
    offset: usize,
}

impl Parser {
    fn sum(&mut self) -> Result<Node, String> {
        let mut node = self.product()?;
        loop {
            let operator = match self.tokens.get(self.offset) {
                Some(Token::Plus(true)) => '+',
                Some(Token::Minus(true)) => '-',
                Some(Token::Plus(false) | Token::Minus(false)) => {
                    return Err("binary + and - require surrounding whitespace".to_owned());
                }
                _ => break,
            };
            self.offset += 1;
            let right = self.product()?;
            node = Node::Binary {
                operator,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn product(&mut self) -> Result<Node, String> {
        let mut node = self.unary()?;
        loop {
            let operator = match self.tokens.get(self.offset) {
                Some(Token::Star) => '*',
                Some(Token::Slash) => '/',
                _ => break,
            };
            self.offset += 1;
            let right = self.unary()?;
            node = Node::Binary {
                operator,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn unary(&mut self) -> Result<Node, String> {
        match self.tokens.get(self.offset) {
            Some(Token::Plus(_)) => {
                self.offset += 1;
                self.unary()
            }
            Some(Token::Minus(_)) => {
                self.offset += 1;
                Ok(Node::Negate(Box::new(self.unary()?)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Node, String> {
        let token = self
            .tokens
            .get(self.offset)
            .cloned()
            .ok_or_else(|| "CSS math value ended early".to_owned())?;
        self.offset += 1;
        match token {
            Token::Number(value, text) => Ok(Node::Literal {
                value,
                unit: text
                    .chars()
                    .all(|character| character.is_ascii_digit())
                    .then_some("")
                    .unwrap_or("")
                    .to_owned(),
            }),
            Token::Percentage(value, _) => Ok(Node::Literal {
                value,
                unit: "%".to_owned(),
            }),
            Token::Dimension(value, _, unit) => Ok(Node::Literal { value, unit }),
            Token::Ident(name) if self.tokens.get(self.offset) == Some(&Token::LeftParen) => {
                self.offset += 1;
                let mut arguments = Vec::new();
                if self.tokens.get(self.offset) != Some(&Token::RightParen) {
                    loop {
                        if name == "round" && arguments.is_empty() {
                            if let Some(Token::Ident(strategy)) =
                                self.tokens.get(self.offset).cloned()
                            {
                                if self.tokens.get(self.offset + 1) == Some(&Token::Comma) {
                                    self.offset += 1;
                                    arguments.push(Node::Keyword(strategy));
                                } else {
                                    arguments.push(self.sum()?);
                                }
                            } else {
                                arguments.push(self.sum()?);
                            }
                        } else {
                            arguments.push(self.sum()?);
                        }
                        if self.tokens.get(self.offset) != Some(&Token::Comma) {
                            break;
                        }
                        self.offset += 1;
                    }
                }
                if self.tokens.get(self.offset) != Some(&Token::RightParen) {
                    return Err("unclosed CSS math function".to_owned());
                }
                self.offset += 1;
                Ok(Node::Function { name, arguments })
            }
            Token::Ident(name) => match name.as_str() {
                "pi" => Ok(Node::Constant(std::f64::consts::PI)),
                "e" => Ok(Node::Constant(std::f64::consts::E)),
                "infinity" => Ok(Node::Constant(f64::INFINITY)),
                "nan" => Ok(Node::Constant(f64::NAN)),
                "size" => Ok(Node::Size),
                "auto" | "min-content" | "max-content" | "fit-content" | "stretch" | "any" => {
                    Ok(Node::Keyword(name))
                }
                _ => Err("unknown CSS math identifier".to_owned()),
            },
            Token::LeftParen => {
                let node = self.sum()?;
                if self.tokens.get(self.offset) != Some(&Token::RightParen) {
                    return Err("unclosed CSS math parenthesis".to_owned());
                }
                self.offset += 1;
                Ok(node)
            }
            _ => Err("unexpected CSS math token".to_owned()),
        }
    }
}

fn infer(node: &Node) -> Result<Kind, String> {
    match node {
        Node::Literal { unit, .. } => unit_kind(unit),
        Node::Constant(_) => Ok(Kind::Number),
        Node::Size => Ok(Kind::Length),
        Node::Keyword(_) => Err("CSS keyword is not a numeric value".to_owned()),
        Node::Negate(value) => infer(value),
        Node::Binary {
            operator,
            left,
            right,
        } => {
            let left = infer(left)?;
            let right = infer(right)?;
            match operator {
                '+' | '-' => compatible_sum_kind(left, right),
                '*' if left == Kind::Number => Ok(right),
                '*' if right == Kind::Number => Ok(left),
                '/' if right == Kind::Number => Ok(left),
                '/' if left == right => Ok(Kind::Number),
                _ => Err("incompatible CSS math dimensions".to_owned()),
            }
        }
        Node::Function { name, arguments } => infer_function(name, arguments),
    }
}

fn compatible_sum_kind(left: Kind, right: Kind) -> Result<Kind, String> {
    if left == right {
        Ok(left)
    } else if matches!(
        left,
        Kind::Length | Kind::Percentage | Kind::LengthPercentage
    ) && matches!(
        right,
        Kind::Length | Kind::Percentage | Kind::LengthPercentage
    ) {
        Ok(Kind::LengthPercentage)
    } else {
        Err("incompatible CSS math sum dimensions".to_owned())
    }
}

fn infer_function(name: &str, arguments: &[Node]) -> Result<Kind, String> {
    match name {
        "calc" | "abs" => one(arguments).and_then(infer),
        "calc-size" if arguments.len() == 2 && matches!(&arguments[0], Node::Keyword(_)) => {
            infer(&arguments[1])
        }
        "sign" => {
            one(arguments)?;
            Ok(Kind::Number)
        }
        "min" | "max" | "hypot" => common_kind(arguments),
        "clamp" if arguments.len() == 3 => common_kind(arguments),
        "sin" | "cos" | "tan" => {
            let kind = infer(one(arguments)?)?;
            matches!(kind, Kind::Number | Kind::Angle)
                .then_some(Kind::Number)
                .ok_or_else(|| "CSS trigonometry requires a number or angle".to_owned())
        }
        "asin" | "acos" | "atan" => (infer(one(arguments)?)? == Kind::Number)
            .then_some(Kind::Angle)
            .ok_or_else(|| "inverse CSS trigonometry requires a number".to_owned()),
        "atan2" if arguments.len() == 2 => {
            let left = infer(&arguments[0])?;
            let right = infer(&arguments[1])?;
            (compatible_sum_kind(left, right).is_ok())
                .then_some(Kind::Angle)
                .ok_or_else(|| "atan2 arguments must have compatible dimensions".to_owned())
        }
        "pow" if arguments.len() == 2 => numbers(arguments).map(|_| Kind::Number),
        "sqrt" | "exp" => numbers(arguments).and_then(|_| {
            (arguments.len() == 1)
                .then_some(Kind::Number)
                .ok_or_else(|| "CSS math function requires one argument".to_owned())
        }),
        "log" if matches!(arguments.len(), 1 | 2) => numbers(arguments).map(|_| Kind::Number),
        "round" => {
            let values = if arguments
                .first()
                .is_some_and(|value| matches!(value, Node::Keyword(_)))
            {
                &arguments[1..]
            } else {
                arguments
            };
            if !matches!(values.len(), 1 | 2) {
                return Err("round() requires one or two numeric values".to_owned());
            }
            common_kind(values)
        }
        "mod" | "rem" if arguments.len() == 2 => common_kind(arguments),
        "progress" if arguments.len() == 3 => {
            common_kind(arguments)?;
            Ok(Kind::Number)
        }
        _ => Err("unsupported CSS math function".to_owned()),
    }
}

fn one(arguments: &[Node]) -> Result<&Node, String> {
    (arguments.len() == 1)
        .then(|| &arguments[0])
        .ok_or_else(|| "CSS math function requires one argument".to_owned())
}

fn numbers(arguments: &[Node]) -> Result<(), String> {
    arguments.iter().try_for_each(|value| {
        (infer(value)? == Kind::Number)
            .then_some(())
            .ok_or_else(|| "CSS math function requires numbers".to_owned())
    })
}

fn common_kind(arguments: &[Node]) -> Result<Kind, String> {
    let (first, rest) = arguments
        .split_first()
        .ok_or_else(|| "CSS math function requires arguments".to_owned())?;
    rest.iter().try_fold(infer(first)?, |kind, value| {
        compatible_sum_kind(kind, infer(value)?)
    })
}

fn unit_kind(unit: &str) -> Result<Kind, String> {
    match unit {
        "" => Ok(Kind::Number),
        "%" => Ok(Kind::Percentage),
        "px" | "cm" | "mm" | "q" | "in" | "pc" | "pt" | "em" | "rem" | "ex" | "rex" | "cap"
        | "rcap" | "ch" | "rch" | "ic" | "ric" | "lh" | "rlh" | "vw" | "vh" | "vi" | "vb"
        | "vmin" | "vmax" | "svw" | "svh" | "svi" | "svb" | "svmin" | "svmax" | "lvw" | "lvh"
        | "lvi" | "lvb" | "lvmin" | "lvmax" | "dvw" | "dvh" | "dvi" | "dvb" | "dvmin" | "dvmax"
        | "cqw" | "cqh" | "cqi" | "cqb" | "cqmin" | "cqmax" => Ok(Kind::Length),
        "deg" | "grad" | "rad" | "turn" => Ok(Kind::Angle),
        "s" | "ms" => Ok(Kind::Time),
        "hz" | "khz" => Ok(Kind::Frequency),
        "dppx" | "dpi" | "dpcm" | "x" => Ok(Kind::Resolution),
        "fr" => Ok(Kind::Flex),
        _ => Err("unknown CSS unit".to_owned()),
    }
}

fn evaluate_symbolic(node: &Node) -> Result<SymbolicValue, String> {
    match node {
        Node::Literal { value, unit } => Ok(SymbolicValue {
            value: *value,
            unit: unit.clone(),
        }),
        Node::Constant(value) => Ok(SymbolicValue {
            value: *value,
            unit: String::new(),
        }),
        Node::Size => Err("size needs an intrinsic-size context".to_owned()),
        Node::Keyword(_) => Err("CSS keyword is not numeric".to_owned()),
        Node::Negate(value) => {
            let mut value = evaluate_symbolic(value)?;
            value.value = -value.value;
            Ok(value)
        }
        Node::Binary {
            operator,
            left,
            right,
        } => {
            let left = evaluate_symbolic(left)?;
            let right = evaluate_symbolic(right)?;
            match operator {
                '+' | '-' if left.unit == right.unit => Ok(SymbolicValue {
                    value: if *operator == '+' {
                        left.value + right.value
                    } else {
                        left.value - right.value
                    },
                    unit: left.unit,
                }),
                '*' if left.unit.is_empty() => Ok(SymbolicValue {
                    value: left.value * right.value,
                    unit: right.unit,
                }),
                '*' if right.unit.is_empty() => Ok(SymbolicValue {
                    value: left.value * right.value,
                    unit: left.unit,
                }),
                '/' if right.unit.is_empty() => Ok(SymbolicValue {
                    value: left.value / right.value,
                    unit: left.unit,
                }),
                '/' if left.unit == right.unit => Ok(SymbolicValue {
                    value: left.value / right.value,
                    unit: String::new(),
                }),
                _ => Err("CSS symbolic units cannot be combined".to_owned()),
            }
        }
        Node::Function { name, arguments } if name == "calc" => evaluate_symbolic(one(arguments)?),
        Node::Function { name, arguments } if name == "calc-size" && arguments.len() == 2 => {
            evaluate_symbolic(&arguments[1])
        }
        Node::Function { name, arguments } if matches!(name.as_str(), "abs" | "sign") => {
            let mut value = evaluate_symbolic(one(arguments)?)?;
            value.value = if name == "abs" {
                value.value.abs()
            } else {
                value.unit.clear();
                value.value.signum()
            };
            Ok(value)
        }
        Node::Function { name, arguments }
            if matches!(name.as_str(), "min" | "max" | "clamp" | "mod" | "rem") =>
        {
            let values = arguments
                .iter()
                .map(evaluate_symbolic)
                .collect::<Result<Vec<_>, _>>()?;
            let first = values
                .first()
                .ok_or_else(|| "CSS math function requires values".to_owned())?;
            if !values.iter().all(|value| value.unit == first.unit) {
                return Err("CSS symbolic units differ".to_owned());
            }
            let result = match name.as_str() {
                "min" => values
                    .iter()
                    .skip(1)
                    .fold(first.value, |value, next| value.min(next.value)),
                "max" => values
                    .iter()
                    .skip(1)
                    .fold(first.value, |value, next| value.max(next.value)),
                "clamp" if values.len() == 3 => {
                    values[1].value.max(values[0].value).min(values[2].value)
                }
                "mod" if values.len() == 2 => {
                    values[0].value - values[1].value * (values[0].value / values[1].value).floor()
                }
                "rem" if values.len() == 2 => values[0].value % values[1].value,
                _ => return Err("invalid CSS symbolic function".to_owned()),
            };
            Ok(SymbolicValue {
                value: result,
                unit: first.unit.clone(),
            })
        }
        _ => Err("CSS expression needs a layout context".to_owned()),
    }
}

fn evaluate(node: &Node, context: EvaluationContext) -> Result<TypedValue, String> {
    match node {
        Node::Literal { value, unit } => evaluate_literal(*value, unit, context),
        Node::Constant(value) => Ok(TypedValue {
            value: *value,
            kind: Kind::Number,
        }),
        Node::Size => Ok(TypedValue {
            value: context
                .intrinsic_size
                .ok_or_else(|| "size needs an intrinsic-size context".to_owned())?,
            kind: Kind::Length,
        }),
        Node::Keyword(_) => Err("CSS keyword is not directly evaluable".to_owned()),
        Node::Negate(value) => {
            let mut value = evaluate(value, context)?;
            value.value = -value.value;
            Ok(value)
        }
        Node::Binary {
            operator,
            left,
            right,
        } => {
            let left = evaluate(left, context)?;
            let right = evaluate(right, context)?;
            evaluate_binary(*operator, left, right)
        }
        Node::Function { name, arguments } => evaluate_function(name, arguments, context),
    }
}

fn evaluate_literal(
    value: f64,
    unit: &str,
    context: EvaluationContext,
) -> Result<TypedValue, String> {
    let (value, kind) = match unit {
        "" => (value, Kind::Number),
        "%" => match context.percentage_basis {
            Some(basis) => (basis * value / 100.0, Kind::Length),
            None => (value, Kind::Percentage),
        },
        "px" => (value, Kind::Length),
        "in" => (value * 96.0, Kind::Length),
        "cm" => (value * 96.0 / 2.54, Kind::Length),
        "mm" => (value * 96.0 / 25.4, Kind::Length),
        "q" => (value * 96.0 / 101.6, Kind::Length),
        "pc" => (value * 16.0, Kind::Length),
        "pt" => (value * 96.0 / 72.0, Kind::Length),
        "em" => (value * finite_context(context.font_size)?, Kind::Length),
        "rem" => (
            value * finite_context(context.root_font_size)?,
            Kind::Length,
        ),
        "ex" | "rex" | "cap" | "rcap" | "ch" | "rch" | "ic" | "ric" => (
            value * finite_context(context.font_size)? * 0.5,
            Kind::Length,
        ),
        "lh" | "rlh" => (
            value * finite_context(context.font_size)? * 1.2,
            Kind::Length,
        ),
        unit if matches!(
            unit,
            "vw" | "vi" | "svw" | "svi" | "lvw" | "lvi" | "dvw" | "dvi" | "cqw" | "cqi"
        ) =>
        {
            (
                value * finite_context(context.viewport_width)? / 100.0,
                Kind::Length,
            )
        }
        unit if matches!(
            unit,
            "vh" | "vb" | "svh" | "svb" | "lvh" | "lvb" | "dvh" | "dvb" | "cqh" | "cqb"
        ) =>
        {
            (
                value * finite_context(context.viewport_height)? / 100.0,
                Kind::Length,
            )
        }
        unit if matches!(unit, "vmin" | "svmin" | "lvmin" | "dvmin" | "cqmin") => (
            value * finite_context(context.viewport_width.min(context.viewport_height))? / 100.0,
            Kind::Length,
        ),
        unit if matches!(unit, "vmax" | "svmax" | "lvmax" | "dvmax" | "cqmax") => (
            value * finite_context(context.viewport_width.max(context.viewport_height))? / 100.0,
            Kind::Length,
        ),
        "deg" => (value, Kind::Angle),
        "grad" => (value * 0.9, Kind::Angle),
        "rad" => (value.to_degrees(), Kind::Angle),
        "turn" => (value * 360.0, Kind::Angle),
        "s" => (value, Kind::Time),
        "ms" => (value / 1000.0, Kind::Time),
        "hz" => (value, Kind::Frequency),
        "khz" => (value * 1000.0, Kind::Frequency),
        "dppx" | "x" => (value, Kind::Resolution),
        "dpi" => (value / 96.0, Kind::Resolution),
        "dpcm" => (value * 2.54 / 96.0, Kind::Resolution),
        "fr" => (value, Kind::Flex),
        _ => return Err("unknown CSS unit".to_owned()),
    };
    Ok(TypedValue { value, kind })
}

fn finite_context(value: f64) -> Result<f64, String> {
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| "CSS value needs layout context".to_owned())
}

fn evaluate_binary(
    operator: char,
    left: TypedValue,
    right: TypedValue,
) -> Result<TypedValue, String> {
    match operator {
        '+' | '-' => {
            let kind = compatible_sum_kind(left.kind, right.kind)?;
            if kind == Kind::LengthPercentage {
                return Err("CSS percentage needs layout context".to_owned());
            }
            Ok(TypedValue {
                value: if operator == '+' {
                    left.value + right.value
                } else {
                    left.value - right.value
                },
                kind,
            })
        }
        '*' if left.kind == Kind::Number => Ok(TypedValue {
            value: left.value * right.value,
            kind: right.kind,
        }),
        '*' if right.kind == Kind::Number => Ok(TypedValue {
            value: left.value * right.value,
            kind: left.kind,
        }),
        '/' if right.kind == Kind::Number => Ok(TypedValue {
            value: left.value / right.value,
            kind: left.kind,
        }),
        '/' if left.kind == right.kind => Ok(TypedValue {
            value: left.value / right.value,
            kind: Kind::Number,
        }),
        _ => Err("incompatible CSS math dimensions".to_owned()),
    }
}

fn evaluate_function(
    name: &str,
    arguments: &[Node],
    context: EvaluationContext,
) -> Result<TypedValue, String> {
    if name == "calc" {
        return evaluate(one(arguments)?, context);
    }
    if name == "calc-size" && arguments.len() == 2 {
        return evaluate(&arguments[1], context);
    }
    let values = arguments
        .iter()
        .filter(|argument| !matches!(argument, Node::Keyword(_)))
        .map(|argument| evaluate(argument, context))
        .collect::<Result<Vec<_>, _>>()?;
    match name {
        "abs" => unary_value(&values, |value| value.abs()),
        "sign" => unary_value(&values, |value| value.signum()).map(|mut value| {
            value.kind = Kind::Number;
            value
        }),
        "min" => extremum(&values, f64::min),
        "max" => extremum(&values, f64::max),
        "clamp" if values.len() == 3 => {
            same_value_kinds(&values)?;
            Ok(TypedValue {
                value: values[1].value.max(values[0].value).min(values[2].value),
                kind: values[0].kind,
            })
        }
        "sin" => trig(&values, libm::sin),
        "cos" => trig(&values, libm::cos),
        "tan" => trig(&values, libm::tan),
        "asin" => inverse_trig(&values, libm::asin),
        "acos" => inverse_trig(&values, libm::acos),
        "atan" => inverse_trig(&values, libm::atan),
        "atan2" if values.len() == 2 => {
            same_value_kinds(&values)?;
            Ok(TypedValue {
                value: libm::atan2(values[0].value, values[1].value).to_degrees(),
                kind: Kind::Angle,
            })
        }
        "pow" if values.len() == 2 => Ok(TypedValue {
            value: libm::pow(values[0].value, values[1].value),
            kind: Kind::Number,
        }),
        "sqrt" => unary_number(&values, libm::sqrt),
        "exp" => unary_number(&values, libm::exp),
        "log" if values.len() == 1 => Ok(TypedValue {
            value: libm::log(values[0].value),
            kind: Kind::Number,
        }),
        "log" if values.len() == 2 => Ok(TypedValue {
            value: libm::log(values[0].value) / libm::log(values[1].value),
            kind: Kind::Number,
        }),
        "hypot" => {
            same_value_kinds(&values)?;
            Ok(TypedValue {
                value: values
                    .iter()
                    .map(|value| value.value.powi(2))
                    .sum::<f64>()
                    .sqrt(),
                kind: values[0].kind,
            })
        }
        "round" => evaluate_round(arguments, &values),
        "mod" if values.len() == 2 => {
            same_value_kinds(&values)?;
            Ok(TypedValue {
                value: values[0].value
                    - values[1].value * (values[0].value / values[1].value).floor(),
                kind: values[0].kind,
            })
        }
        "rem" if values.len() == 2 => {
            same_value_kinds(&values)?;
            Ok(TypedValue {
                value: values[0].value % values[1].value,
                kind: values[0].kind,
            })
        }
        "progress" if values.len() == 3 => {
            same_value_kinds(&values)?;
            Ok(TypedValue {
                value: ((values[0].value - values[1].value) / (values[2].value - values[1].value))
                    .clamp(0.0, 1.0),
                kind: Kind::Number,
            })
        }
        _ => Err("unsupported CSS math function".to_owned()),
    }
}

fn unary_value(
    values: &[TypedValue],
    operation: impl FnOnce(f64) -> f64,
) -> Result<TypedValue, String> {
    (values.len() == 1)
        .then(|| TypedValue {
            value: operation(values[0].value),
            kind: values[0].kind,
        })
        .ok_or_else(|| "CSS math function requires one value".to_owned())
}

fn unary_number(
    values: &[TypedValue],
    operation: impl FnOnce(f64) -> f64,
) -> Result<TypedValue, String> {
    if values.len() != 1 || values[0].kind != Kind::Number {
        return Err("CSS math function requires one number".to_owned());
    }
    Ok(TypedValue {
        value: operation(values[0].value),
        kind: Kind::Number,
    })
}

fn trig(values: &[TypedValue], operation: impl FnOnce(f64) -> f64) -> Result<TypedValue, String> {
    if values.len() != 1 || !matches!(values[0].kind, Kind::Number | Kind::Angle) {
        return Err("CSS trigonometry requires one number or angle".to_owned());
    }
    let radians = if values[0].kind == Kind::Angle {
        values[0].value.to_radians()
    } else {
        values[0].value
    };
    Ok(TypedValue {
        value: operation(radians),
        kind: Kind::Number,
    })
}

fn inverse_trig(
    values: &[TypedValue],
    operation: impl FnOnce(f64) -> f64,
) -> Result<TypedValue, String> {
    if values.len() != 1 || values[0].kind != Kind::Number {
        return Err("inverse CSS trigonometry requires one number".to_owned());
    }
    Ok(TypedValue {
        value: operation(values[0].value).to_degrees(),
        kind: Kind::Angle,
    })
}

fn same_value_kinds(values: &[TypedValue]) -> Result<Kind, String> {
    let first = values
        .first()
        .ok_or_else(|| "CSS math function requires values".to_owned())?
        .kind;
    values
        .iter()
        .all(|value| value.kind == first)
        .then_some(first)
        .ok_or_else(|| "CSS math values have incompatible dimensions".to_owned())
}

fn extremum(
    values: &[TypedValue],
    operation: impl Fn(f64, f64) -> f64,
) -> Result<TypedValue, String> {
    let kind = same_value_kinds(values)?;
    Ok(TypedValue {
        value: values
            .iter()
            .skip(1)
            .fold(values[0].value, |current, value| {
                operation(current, value.value)
            }),
        kind,
    })
}

fn evaluate_round(arguments: &[Node], values: &[TypedValue]) -> Result<TypedValue, String> {
    let strategy = arguments
        .first()
        .and_then(|value| match value {
            Node::Keyword(value) => Some(value.as_str()),
            _ => None,
        })
        .unwrap_or("nearest");
    if !matches!(values.len(), 1 | 2) {
        return Err("round() requires a value and optional step".to_owned());
    }
    if values.len() == 2 && values[0].kind != values[1].kind {
        return Err("round() values have incompatible dimensions".to_owned());
    }
    let step = values.get(1).map(|value| value.value).unwrap_or(1.0);
    if step == 0.0 {
        return Err("round() step cannot be zero".to_owned());
    }
    let ratio = values[0].value / step;
    let rounded = match strategy {
        "up" => ratio.ceil(),
        "down" => ratio.floor(),
        "to-zero" => ratio.trunc(),
        // CSS resolves an exact nearest tie toward positive infinity.
        "nearest" => (ratio + 0.5).floor(),
        _ => return Err("unknown round() strategy".to_owned()),
    };
    Ok(TypedValue {
        value: rounded * step,
        kind: values[0].kind,
    })
}

fn serialize_root(node: &Node) -> String {
    match node {
        Node::Function { name, arguments } if name != "calc" => serialize_function(name, arguments),
        Node::Function { arguments, .. } => format!("calc({})", serialize_node(&arguments[0])),
        _ => format!("calc({})", serialize_node(node)),
    }
}

fn serialize_node(node: &Node) -> String {
    serialize_with_precedence(node, 0, false, '\0')
}

fn serialize_with_precedence(
    node: &Node,
    parent_precedence: u8,
    right_child: bool,
    parent_operator: char,
) -> String {
    let precedence = match node {
        Node::Binary {
            operator: '+' | '-',
            ..
        } => 1,
        Node::Binary { .. } => 2,
        Node::Negate(_) => 3,
        _ => 4,
    };
    let output = match node {
        Node::Literal { value, unit } => format!("{}{}", format_number(*value), unit),
        Node::Constant(value) if *value == std::f64::consts::PI => "pi".to_owned(),
        Node::Constant(value) if *value == std::f64::consts::E => "e".to_owned(),
        Node::Constant(value) if value.is_nan() => "NaN".to_owned(),
        Node::Constant(value) if value.is_infinite() => "infinity".to_owned(),
        Node::Constant(value) => format_number(*value),
        Node::Size => "size".to_owned(),
        Node::Keyword(value) => value.clone(),
        Node::Negate(value) => format!("-{}", serialize_with_precedence(value, 3, true, '-')),
        Node::Binary {
            operator,
            left,
            right,
        } => {
            let (left, right) =
                if *operator == '+' && serialization_rank(right) < serialization_rank(left) {
                    (right.as_ref(), left.as_ref())
                } else {
                    (left.as_ref(), right.as_ref())
                };
            format!(
                "{} {} {}",
                serialize_with_precedence(left, precedence, false, *operator),
                operator,
                serialize_with_precedence(right, precedence, true, *operator)
            )
        }
        Node::Function { name, arguments } => serialize_function(name, arguments),
    };
    let needs_parentheses = precedence < parent_precedence
        || right_child && precedence == parent_precedence && matches!(parent_operator, '-' | '/');
    if needs_parentheses {
        format!("({output})")
    } else {
        output
    }
}

fn serialization_rank(node: &Node) -> u8 {
    match node {
        Node::Literal { unit, .. } if unit == "%" => 0,
        Node::Literal { unit, .. }
            if matches!(unit.as_str(), "px" | "cm" | "mm" | "q" | "in" | "pc" | "pt") =>
        {
            1
        }
        Node::Literal { unit, .. }
            if matches!(
                unit.as_str(),
                "em" | "rem"
                    | "ex"
                    | "rex"
                    | "cap"
                    | "rcap"
                    | "ch"
                    | "rch"
                    | "ic"
                    | "ric"
                    | "lh"
                    | "rlh"
            ) =>
        {
            2
        }
        Node::Literal { .. } => 3,
        Node::Size => 5,
        _ => 4,
    }
}

fn serialize_function(name: &str, arguments: &[Node]) -> String {
    format!(
        "{name}({})",
        arguments
            .iter()
            .map(serialize_node)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_standard_named_and_system_color_normalizes_and_computes() {
        assert_eq!(NAMED_COLORS.len(), 148);
        for (name, _) in NAMED_COLORS {
            assert_eq!(
                normalize_property_value("color", name).as_deref(),
                Some(*name)
            );
            let computed = computed_color("color", name).expect("named color computes");
            assert!(computed.starts_with("rgb("), "{name}: {computed}");
        }
        for (name, expected) in SYSTEM_COLORS {
            assert_eq!(
                normalize_property_value("color", name).as_deref(),
                Some(*name)
            );
            assert_eq!(computed_color("color", name).as_deref(), Some(*expected));
        }
        for invalid in ["FakeColor", "-moz-ButtonDefault", "#12", "#ggg", "rgb(foo)"] {
            assert!(
                normalize_property_value("color", invalid).is_none(),
                "{invalid}"
            );
            assert!(!supports_property("color", invalid), "{invalid}");
        }
    }

    #[test]
    fn edge_complex_width_expression_reduces_to_a_canonical_pixel_calc() {
        let value = "calc( 1px * ( ( 2.71828 * 0.987654321 - 0.123456789 / ( 987654.321 * 12345.678 ) ) + 0.5555555 / sin( sin( 10000.1 * tan( 50000 ) / tan( 20000 ) + 1.0 / pi * 5.0 - 0.1111 ) / 100.0 + tan( 30000 + 40000 * 50000 + 0.0001 ) / 9999.9 * pi ) - 0.999999 * -100000.5 ) )";
        assert_eq!(
            resolve_length(value, EvaluationContext::constants_only()),
            Some(100075.0)
        );
        assert_eq!(
            normalize_property_value("width", value).as_deref(),
            Some("calc(100075px)")
        );
    }

    #[test]
    fn positive_css_lengths_use_blink_layout_unit_flooring() {
        assert_eq!(
            resolve_length("1.0078in", EvaluationContext::constants_only()),
            Some(96.734375)
        );
        assert_eq!(
            resolve_length("1.0104in", EvaluationContext::constants_only()),
            Some(96.984375)
        );
        assert_eq!(
            resolve_length("1.0050in", EvaluationContext::constants_only()),
            Some(96.46875)
        );
        assert_eq!(
            resolve_length("1.0052in", EvaluationContext::constants_only()),
            Some(96.484375)
        );
        assert_eq!(
            resolve_length("1.0053in", EvaluationContext::constants_only()),
            Some(96.5)
        );
    }

    #[test]
    fn css_math_dimensions_and_level_four_functions_are_evaluated() {
        for (value, expected) in [
            ("calc(1in + 96px)", "calc(192px)"),
            ("min(10px, 20px)", "calc(10px)"),
            ("max(10px, 20px)", "calc(20px)"),
            ("clamp(1px, 3px, 2px)", "calc(2px)"),
            ("calc(sin(pi / 2) * 1px)", "calc(1px)"),
            ("calc(round(up, 2.1, 1) * 1px)", "calc(3px)"),
            ("calc(mod(7, 4) * 1px)", "calc(3px)"),
            ("calc(pow(2, 3) * 1px)", "calc(8px)"),
            ("calc(hypot(3, 4) * 1px)", "calc(5px)"),
            ("calc(log(exp(2)) * 1px)", "calc(2px)"),
            ("calc(progress(10px, 0px, 20px) * 100px)", "calc(50px)"),
        ] {
            assert_eq!(
                normalize_property_value("width", value).as_deref(),
                Some(expected)
            );
        }
        assert!(normalize_property_value("width", "calc(1dppx + 96dpi)").is_none());
        assert_eq!(
            normalize_property_value("width", "calc-size(auto, size + 10px)").as_deref(),
            Some("calc-size(auto, 10px + size)")
        );
    }

    #[test]
    fn nested_math_shorthands_comments_and_rounding_are_normalized() {
        assert_eq!(
            normalize_property_value(
                "transform",
                "translate(calc(1px + 2px), calc(10% + 1px)) rotate(calc(.5turn / 2))",
            )
            .as_deref(),
            Some("translate(calc(3px), calc(10% + 1px)) rotate(calc(90deg))")
        );
        assert_eq!(
            normalize_property_value("margin", "calc(1px/*x*/ + 2px) calc(2px * 2)").as_deref(),
            Some("calc(3px) calc(4px)")
        );
        assert_eq!(
            normalize_property_value("width", "calc(round(nearest, -2.5, 1) * 1px)").as_deref(),
            Some("calc(-2px)")
        );
        assert!(normalize_property_value("width", "calc(1px+2px)").is_none());
        assert_eq!(
            normalize_property_value("width", "calc((50% + 1px) * 2)").as_deref(),
            Some("calc((50% + 1px) * 2)")
        );
    }
}
