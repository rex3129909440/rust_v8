use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => value,
        value => value.to_string(),
    }
}

#[test]
fn edge_150_css_own_shape_and_all_element_reference_reflections_are_exact() {
    let source = r##"
      (() => {
        const style = document.createElement("div").style;
        const emptyKeys = Reflect.ownKeys(style);
        const named = Object.getOwnPropertyDescriptor(style, "animationName");
        style.animationName = "spin";
        style.setProperty("--ok", "1");
        style.setProperty("animation-garbage", "bad");
        style.cssText += "; color-foo: bad; text-totally-invalid: bad; color: red";
        const indexed = Object.getOwnPropertyDescriptor(style, "0");

        const root = document.createElement("div");
        const one = document.createElement("span");
        const two = document.createElement("span");
        one.id = "one";
        two.id = "two";
        const button = document.createElement("button");
        const area = document.createElement("area");
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "a");
        root.append(one, two, button, area, svg);

        const reflection = (element, property, attribute) => {
          element.setAttribute(attribute, "one");
          const fromAttribute = element[property] === one;
          element[property] = two;
          const fromProperty =
            element.getAttribute(attribute) === "" && element[property] === two;
          element.setAttribute(attribute, "one");
          const invalidated = element[property] === one;
          element[property] = undefined;
          const nullable =
            element[property] === null && !element.hasAttribute(attribute);
          let brand = "";
          try { element[property] = {}; } catch (error) { brand = error.name; }
          return [fromAttribute, fromProperty, invalidated, nullable, brand];
        };

        return [
          emptyKeys.length,
          emptyKeys[0],
          emptyKeys.at(-1),
          emptyKeys.includes("animationName"),
          emptyKeys.includes("animationGarbage"),
          named.value,
          named.writable,
          named.enumerable,
          named.configurable,
          indexed.value,
          indexed.writable,
          indexed.enumerable,
          indexed.configurable,
          style.getPropertyValue("animation-garbage"),
          style.getPropertyValue("color-foo"),
          style.getPropertyValue("text-totally-invalid"),
          style.getPropertyValue("--ok"),
          ...reflection(button, "interestForElement", "interestfor"),
          ...reflection(button, "popoverTargetElement", "popovertarget"),
          ...reflection(area, "interestForElement", "interestfor"),
          ...reflection(svg, "interestForElement", "interestfor")
        ].join("|");
      })()
    "##;
    let expected = concat!(
        "744|accentColor|zoom|true|false||true|true|true|",
        "animation-name|false|true|true||||1|",
        "true|true|true|true|TypeError|",
        "true|true|true|true|TypeError|",
        "true|true|true|true|TypeError|",
        "true|true|true|true|TypeError"
    );
    let mut direct = EdgeRuntime::new().expect("direct CSS/reflection runtime");
    assert_eq!(text(&mut direct, source), expected);
    let mut traced = EdgeRuntime::new().expect("traced CSS/reflection runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn cssom_get_property_value_rejects_unknown_names_like_edge() {
    let source = r#"
      (() => {
        const element = document.createElement("div");
        element.style.setProperty("color", "red");
        element.style.setProperty("--CaseSensitive", "ok");
        const computed = getComputedStyle(element);
        return [
          element.style.getPropertyValue("color"),
          element.style.getPropertyValue("COLOR"),
          element.style.getPropertyValue(" color "),
          element.style.getPropertyValue("backgroundColor"),
          element.style.getPropertyValue("ActiveBorder"),
          computed.getPropertyValue("ActiveBorder"),
          element.style.getPropertyValue("--CaseSensitive"),
          element.style.getPropertyValue("--casesensitive")
        ].join("|");
      })()
    "#;
    let expected = "red|red|||||ok|";
    let mut direct = EdgeRuntime::new().expect("direct CSSOM runtime");
    assert_eq!(text(&mut direct, source), expected);
    let mut traced = EdgeRuntime::new().expect("traced CSSOM runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn css_system_colors_serialize_and_compute_like_edge() {
    let source = r#"
      (() => {
        const element = document.createElement("div");
        document.body.appendChild(element);
        element.style.color = "ActiveBorder";
        element.style.backgroundColor = "Canvas";
        return [
          element.style.getPropertyValue("color"),
          element.style.getPropertyValue("background-color"),
          getComputedStyle(element).getPropertyValue("color"),
          getComputedStyle(element).getPropertyValue("background-color")
        ].join("|");
      })()
    "#;
    let expected = "activeborder|canvas|rgb(0, 0, 0)|rgb(255, 255, 255)";
    let mut direct = EdgeRuntime::new().expect("direct system-color runtime");
    assert_eq!(text(&mut direct, source), expected);
    let mut traced = EdgeRuntime::new().expect("traced system-color runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn css_color_parsing_rejects_unknown_values_and_computes_complete_keywords() {
    let source = r##"
      (() => {
        const tokens = [
          "WindowText",
          "ActiveBorder",
          "AccentColor",
          "rebeccapurple",
          "transparent",
          "#1234",
          "rgb(1 2 3 / 50%)",
          "hsl(120 50% 25%)",
          "hwb(120 10% 20%)",
          "lab(50% 0 0)",
          "lch(50% 20 30)",
          "oklab(0.5 0 0)",
          "oklch(0.5 0.2 30)",
          "color(display-p3 1 0 0)",
          "color-mix(in srgb, red, blue)",
          "light-dark(white, black)",
          "-moz-ButtonDefault",
          "FakeColor",
          "#12",
          "rgb(foo)"
        ];
        return tokens.map(token => {
          const element = document.createElement("div");
          document.body.appendChild(element);
          element.style.color = token;
          return [
            token,
            CSS.supports("color", token),
            element.style.color,
            getComputedStyle(element).color,
            getComputedStyle(element).getPropertyValue(token)
          ].join("~");
        }).join("|");
      })()
    "##;
    let mut runtime = EdgeRuntime::new().expect("complete CSS color runtime");
    let result = text(&mut runtime, source);
    let rows = result.split('|').collect::<Vec<_>>();
    assert_eq!(rows.len(), 20);
    for row in &rows[..16] {
        let fields = row.split('~').collect::<Vec<_>>();
        assert_eq!(fields[1], "true", "valid color row: {row}");
        assert!(!fields[2].is_empty(), "declared color row: {row}");
        assert!(!fields[3].is_empty(), "computed color row: {row}");
        assert!(fields[4].is_empty(), "token is not a property name: {row}");
    }
    for row in &rows[16..] {
        let fields = row.split('~').collect::<Vec<_>>();
        assert_eq!(fields[1], "false", "invalid color row: {row}");
        assert!(fields[2].is_empty(), "invalid declared color row: {row}");
        assert_eq!(
            fields[3], "rgb(0, 0, 0)",
            "invalid computed color row: {row}"
        );
        assert!(fields[4].is_empty(), "invalid token property row: {row}");
    }
}

#[test]
fn edge_150_complete_legacy_and_mozilla_color_matrix_is_exact() {
    let source = r##"
      (() => {
        const tokens = [
          "ActiveBorder", "ActiveCaption", "AppWorkspace", "Background",
          "ButtonFace", "ButtonHighlight", "ButtonShadow", "ButtonText",
          "CaptionText", "GrayText", "Highlight", "HighlightText",
          "InactiveBorder", "InactiveCaption", "InactiveCaptionText",
          "InfoBackground", "InfoText", "Menu", "MenuText", "Scrollbar",
          "ThreeDDarkShadow", "ThreeDFace", "ThreeDHighlight",
          "ThreeDLightShadow", "ThreeDShadow", "Window", "WindowFrame",
          "WindowText", "FakeColor", "-moz-ButtonDefault",
          "-moz-ButtonHoverFace", "-moz-ButtonHoverText", "-moz-CellHighlight",
          "-moz-CellHighlightText", "-moz-Combobox", "-moz-ComboboxText",
          "-moz-Dialog", "-moz-DialogText", "-moz-dragtargetzone",
          "-moz-EvenTreeRow", "-moz-Field", "-moz-FieldText",
          "-moz-html-CellHighlight", "-moz-html-CellHighlightText",
          "-moz-mac-accentdarkestshadow", "-moz-mac-accentdarkshadow",
          "-moz-mac-accentface", "-moz-mac-accentlightesthighlight",
          "-moz-mac-accentlightshadow", "-moz-mac-accentregularhighlight",
          "-moz-mac-accentregularshadow", "-moz-mac-chrome-active",
          "-moz-mac-chrome-inactive", "-moz-mac-focusring",
          "-moz-mac-menuselect", "-moz-mac-menushadow",
          "-moz-mac-menutextselect", "-moz-MenuHover", "-moz-MenuHoverText",
          "-moz-MenuBarText", "-moz-MenuBarHoverText",
          "-moz-nativehyperlinktext", "-moz-OddTreeRow",
          "-moz-win-communicationstext", "-moz-win-mediatext",
          "-moz-win-accentcolor", "-moz-win-accentcolortext",
          "-moz-activehyperlinktext", "-moz-default-background-color",
          "-moz-default-color", "-moz-hyperlinktext", "-moz-visitedhyperlinktext"
        ];
        const rows = tokens.map(token => {
          const element = document.createElement("div");
          document.body.appendChild(element);
          element.style.color = token;
          const computed = getComputedStyle(element);
          return [
            token,
            CSS.supports("color", token),
            element.style.color,
            computed.color,
            computed.getPropertyValue(token)
          ].join("~");
        });
        const retained = document.createElement("div");
        retained.style.color = "rgb(0, 0, 0)";
        retained.style.color = "FakeColor";
        rows.push([retained.style.color, getComputedStyle(retained).color].join("~"));
        return rows.join("|");
      })()
    "##;
    let mut runtime = EdgeRuntime::new().expect("Edge CSS color matrix runtime");
    let result = text(&mut runtime, source);
    let rows = result.split('|').collect::<Vec<_>>();
    assert_eq!(rows.len(), 73);
    let expected_computed = [
        "rgb(0, 0, 0)",
        "rgb(255, 255, 255)",
        "rgb(255, 255, 255)",
        "rgb(255, 255, 255)",
        "rgb(240, 240, 240)",
        "rgb(240, 240, 240)",
        "rgb(240, 240, 240)",
        "rgb(0, 0, 0)",
        "rgb(0, 0, 0)",
        "rgb(109, 109, 109)",
        "rgb(0, 120, 215)",
        "rgb(255, 255, 255)",
        "rgb(0, 0, 0)",
        "rgb(255, 255, 255)",
        "rgb(128, 128, 128)",
        "rgb(255, 255, 255)",
        "rgb(0, 0, 0)",
        "rgb(255, 255, 255)",
        "rgb(0, 0, 0)",
        "rgb(255, 255, 255)",
        "rgb(0, 0, 0)",
        "rgb(240, 240, 240)",
        "rgb(0, 0, 0)",
        "rgb(0, 0, 0)",
        "rgb(0, 0, 0)",
        "rgb(255, 255, 255)",
        "rgb(0, 0, 0)",
        "rgb(0, 0, 0)",
    ];

    for (index, row) in rows[..28].iter().enumerate() {
        let fields = row.split('~').collect::<Vec<_>>();
        assert_eq!(fields[1], "true", "accepted Edge system color: {row}");
        assert_eq!(
            fields[2],
            fields[0].to_ascii_lowercase(),
            "declared system color serialization: {row}"
        );
        assert_eq!(
            fields[3], expected_computed[index],
            "computed system color must match Edge 150: {row}"
        );
        if fields[0] == "Background" {
            assert!(
                fields[4].starts_with("rgba(0, 0, 0, 0) none repeat scroll"),
                "Background is also a CSS property name: {row}"
            );
        } else {
            assert!(fields[4].is_empty(), "color is not a property name: {row}");
        }
    }
    for row in &rows[28..72] {
        let fields = row.split('~').collect::<Vec<_>>();
        assert_eq!(fields[1], "false", "unsupported Edge color: {row}");
        assert!(fields[2].is_empty(), "unsupported declaration: {row}");
        assert_eq!(fields[3], "rgb(0, 0, 0)", "initial computed color: {row}");
        assert!(fields[4].is_empty(), "unsupported property lookup: {row}");
    }
    assert_eq!(rows[72], "rgb(0, 0, 0)~rgb(0, 0, 0)");
}

#[test]
fn node_tree_mutations_fragments_cloning_and_relationships_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const parent = document.createElement("div");
          const first = document.createElement("span");
          const second = document.createElement("b");
          const children = parent.childNodes;
          parent.append(first, second);
          const liveChildren =
            children === parent.childNodes && children.length === 2;

          const fragment = document.createDocumentFragment();
          fragment.append(document.createElement("i"), "tail");
          const fragmentReturn = parent.appendChild(fragment);

          let cycle = "", badReference = "", missingChild = "";
          try { first.appendChild(parent); }
          catch (error) { cycle = error.name; }
          try {
            parent.insertBefore(
              document.createElement("u"),
              document.createElement("s")
            );
          } catch (error) { badReference = error.name; }
          try { parent.removeChild(document.createElement("u")); }
          catch (error) { missingChild = error.name; }

          const adjacent = document.createElement("div");
          adjacent.append("a", "", "b");
          adjacent.normalize();
          const normalized =
            adjacent.childNodes.length + ":" + adjacent.firstChild.data;
          adjacent.textContent = "replacement";
          const replacedText =
            adjacent.childNodes.length + ":" +
            adjacent.firstChild.nodeName + ":" + adjacent.textContent;

          parent.id = "source";
          parent.setAttribute("data-copy", "yes");
          const deep = parent.cloneNode(true);
          const shallow = parent.cloneNode(false);
          const cloneShape = [
            deep.constructor.name,
            deep.tagName,
            deep.id,
            deep.getAttribute("data-copy"),
            deep.children.length,
            deep.firstChild === first,
            shallow.children.length
          ].join(":");

          const positions = [
            parent.compareDocumentPosition(first),
            first.compareDocumentPosition(parent),
            first.compareDocumentPosition(second),
            second.compareDocumentPosition(first)
          ].join(",");

          document.body.appendChild(parent);
          const host = document.createElement("x-host");
          document.body.appendChild(host);
          const shadow = host.attachShadow({ mode: "open" });
          const inner = document.createElement("em");
          shadow.appendChild(inner);
          const roots = [
            inner.getRootNode() === shadow,
            inner.getRootNode({ composed: true }) === document
          ].join(",");

          return [
            liveChildren,
            fragmentReturn === fragment,
            fragment.childNodes.length,
            Array.from(parent.childNodes, node => node.nodeName).join(","),
            cycle, badReference, missingChild,
            normalized, replacedText, cloneShape, positions, roots,
            document.textContent === null
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|0|SPAN,B,I,#text|HierarchyRequestError|NotFoundError|NotFoundError|1:ab|1:#text:replacement|HTMLDivElement:DIV:source:yes:3:false:0|20,10,4,2|true,true|true"
    );
}

#[test]
fn document_factories_namespaces_adoption_and_import_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const html = document.createElement("DiV");
          const unknown = document.createElement("notarealtag");
          const custom = document.createElement("x-widget");
          const svg = document.createElementNS(
            "http://www.w3.org/2000/svg", "svg:path"
          );
          const math = document.createElementNS(
            "http://www.w3.org/1998/Math/MathML", "mrow"
          );
          const attr = document.createAttributeNS("urn:attributes", "x:name");
          const comment = document.createComment("note");
          const textNode = document.createTextNode("text");
          const pi = document.createProcessingInstruction("target", "data");
          const fragment = document.createDocumentFragment();
          let invalidElement = "", invalidPi = "", htmlCdata = "";
          try { document.createElement("bad name"); }
          catch (error) { invalidElement = error.name; }
          try { document.createProcessingInstruction("bad name", "x"); }
          catch (error) { invalidPi = error.name; }
          try { document.createCDATASection("x"); }
          catch (error) { htmlCdata = error.name; }

          const source = document.createElement("section");
          source.setAttribute("data-value", "copied");
          source.appendChild(document.createElement("strong"));
          const imported = document.importNode(source, true);
          const adopted = document.adoptNode(source);
          return [
            html.constructor.name, html.localName, html.tagName,
            unknown.constructor.name, custom.constructor.name,
            svg.constructor.name, svg.namespaceURI, svg.prefix,
            svg.localName, svg.tagName,
            math.constructor.name, math.namespaceURI,
            attr.namespaceURI, attr.prefix, attr.localName,
            comment.nodeType, textNode.nodeType, pi.nodeType,
            fragment.nodeType,
            invalidElement, invalidPi, htmlCdata,
            imported !== source,
            imported.getAttribute("data-value"),
            imported.children.length,
            imported.ownerDocument === document,
            adopted === source,
            adopted.ownerDocument === document,
            adopted.firstChild.ownerDocument === document
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "HTMLDivElement|div|DIV|HTMLUnknownElement|HTMLElement|SVGPathElement|http://www.w3.org/2000/svg|svg|path|svg:path|MathMLElement|http://www.w3.org/1998/Math/MathML|urn:attributes|x|name|8|3|7|11|InvalidCharacterError|InvalidCharacterError|NotSupportedError|true|copied|1|true|true|true|true"
    );
}

#[test]
fn document_and_element_queries_are_ordered_live_and_static_as_required() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const root = document.createElement("main");
          root.id = "query-root";
          root.innerHTML = "";
          const first = document.createElement("section");
          first.id = "first";
          first.className = "card selected";
          first.setAttribute("data-kind", "primary");
          const nested = document.createElement("span");
          nested.className = "label";
          first.appendChild(nested);
          const second = document.createElement("section");
          second.id = "second";
          second.className = "card";
          second.setAttribute("name", "named-section");
          root.append(first, second);
          document.body.appendChild(root);

          const liveTags = root.getElementsByTagName("section");
          const liveClasses = document.getElementsByClassName("card");
          const staticList = root.querySelectorAll(".card");
          const third = document.createElement("section");
          third.className = "card";
          root.appendChild(third);

          const complex =
            document.querySelector(
              'main#query-root > section.card[data-kind="primary"] span.label'
            ) === nested;
          const grouped = root.querySelectorAll("#second, #first");
          return [
            document.getElementById("first") === first,
            document.getElementsByName("named-section")[0] === second,
            liveTags.length,
            liveClasses.length,
            staticList.length,
            complex,
            Array.from(grouped, element => element.id).join(","),
            root.querySelector(":scope > #second") === second,
            root.matches("main#query-root"),
            nested.closest("section") === first,
            root.children === root.children,
            root.children.length,
            root.firstElementChild === first,
            root.lastElementChild === third,
            first.nextElementSibling === second
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|3|3|2|true|first,second|true|true|true|true|3|true|true|true"
    );
}

#[test]
fn attributes_character_data_range_selection_and_traversal_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const root = document.createElement("div");
          root.setAttribute("id", "attributes");
          root.setAttributeNS("urn:test", "x:value", "one");
          const attributes = root.attributes;
          const attribute = root.getAttributeNodeNS("urn:test", "value");
          attribute.value = "two";
          const replacement = document.createAttribute("title");
          replacement.value = "hello";
          const previous = root.setAttributeNode(replacement);

          root.classList.add("a", "b");
          root.classList.toggle("b");
          root.classList.replace("a", "c");

          const textNode = document.createTextNode("abcdef");
          root.appendChild(textNode);
          const tail = textNode.splitText(3);
          textNode.replaceData(1, 1, "Z");

          document.body.appendChild(root);
          const range = document.createRange();
          range.setStart(textNode, 1);
          range.setEnd(tail, 2);
          const selection = document.getSelection();
          selection.removeAllRanges();
          selection.addRange(range);

          const walker = document.createTreeWalker(
            root, NodeFilter.SHOW_ALL
          );
          const iterator = document.createNodeIterator(
            root, NodeFilter.SHOW_ALL
          );
          return [
            attributes === root.attributes,
            attributes.length,
            attribute.ownerElement === root,
            root.getAttributeNS("urn:test", "value"),
            previous === null,
            root.title,
            root.className,
            textNode.data,
            tail.data,
            textNode.wholeText,
            range.toString(),
            selection.rangeCount,
            selection.toString(),
            walker.currentNode === root,
            walker.nextNode() === textNode,
            iterator.nextNode() === root,
            iterator.nextNode() === textNode
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "true|4|true|two|true|hello|c|aZc|def|aZcdef|Zcde|1|Zcde|true|true|true|true"
    );
}

#[test]
fn document_all_and_cookie_follow_edge_legacy_semantics() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const all = document.all;
          const element = document.createElement("div");
          element.id = "legacy-all-id";
          element.setAttribute("name", "legacy-all-name");
          document.body.appendChild(element);
          const allShape = [
            typeof all,
            Boolean(all),
            all == null,
            all === undefined,
            Object.prototype.toString.call(all),
            all === document.all,
            all("legacy-all-id") === element,
            all.namedItem("legacy-all-id") === element,
            all["legacy-all-id"] === element,
            all.length === document.getElementsByTagName("*").length
          ].join(",");

          document.cookie = "first=one; Path=/";
          document.cookie = "second=two; Path=/";
          document.cookie = "first=three; Path=/";
          document.cookie = "hidden=value; Max-Age=0; Path=/";
          const cookie = document.cookie;
          document.cookie = "first=; Max-Age=0; Path=/";
          const afterDelete = document.cookie;
          return [allShape, cookie, afterDelete].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "undefined,false,true,false,[object HTMLAllCollection],true,true,true,true,true|second=two; first=three|second=two"
    );
}

#[test]
fn html_fragment_parsing_serialization_and_outer_html_use_the_dom_tree() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const container = document.createElement("div");
          const live = container.getElementsByTagName("*");
          container.innerHTML =
            '<section id="a"><span data-x="1">A &amp; B</span><!--c--><br></section>' +
            '<svg><circle cx="2"></circle></svg>' +
            '<script>if (a < b) value = "<x>";</script>';
          const section = container.firstElementChild;
          const span = section.firstElementChild;
          const before = [
            live.length,
            container.querySelector("#a") === section,
            span.textContent,
            span.ownerDocument === document,
            container.querySelector("svg").namespaceURI,
            container.querySelector("circle").namespaceURI,
            container.lastElementChild.textContent,
            container.lastElementChild.childNodes.length,
            container.lastElementChild.innerHTML
          ].join(",");

          span.outerHTML = '<b class="replacement">R</b><i>I</i>';
          const detached = span.parentNode === null;
          const replacement = section.firstElementChild;
          const after = [
            detached,
            replacement.tagName,
            replacement.nextElementSibling.tagName,
            section.children.length,
            section.textContent,
            live.length,
            replacement.outerHTML,
            container.innerHTML.includes("<!--c-->"),
            container.innerHTML.includes("<br>")
          ].join(",");
          container.innerHTML = "";
          return [before, after, container.childNodes.length, live.length].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "6,true,A & B,true,http://www.w3.org/2000/svg,http://www.w3.org/2000/svg,if (a < b) value = \"<x>\";,1,if (a < b) value = \"<x>\";|true,B,I,3,RI,7,<b class=\"replacement\">R</b>,true,true|0|0"
    );
}

#[test]
fn childnode_parentnode_and_adjacent_insertion_methods_mutate_in_document_order() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const root = document.createElement("div");
          root.innerHTML = '<p id="a">A</p><p id="b">B</p>';
          const a = root.firstElementChild;
          const b = root.lastElementChild;
          const em = document.createElement("em");
          em.textContent = "E";
          a.before("0", em);
          const strong = document.createElement("strong");
          strong.textContent = "T";
          a.after(strong, "x");

          const inside = document.createElement("i");
          inside.textContent = "I";
          const inserted = a.insertAdjacentElement("beforeend", inside);
          a.insertAdjacentText("afterbegin", "!");
          a.insertAdjacentHTML("beforeend", "<u>U</u><small>S</small>");
          b.insertAdjacentHTML("beforebegin", "<hr><q>Q</q>");

          const mark = document.createElement("mark");
          mark.textContent = "M";
          strong.replaceWith("r", mark);

          let invalid = "";
          try { a.insertAdjacentText("middle", "bad"); }
          catch (error) { invalid = error.name; }

          const detached = document.createElement("aside");
          const detachedInsert =
            detached.insertAdjacentElement("beforebegin", document.createElement("b"));

          const holder = document.createElement("section");
          holder.append("tail");
          holder.prepend("head", document.createElement("br"));
          holder.replaceChildren("only", document.createElement("img"));

          return [
            inserted === inside,
            root.innerHTML,
            a.textContent,
            invalid,
            detachedInsert === null,
            holder.innerHTML,
            root.children.length,
            root.childNodes.length
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|0<em>E</em><p id=\"a\">!A<i>I</i><u>U</u><small>S</small></p>r<mark>M</mark>x<hr><q>Q</q><p id=\"b\">B</p>|!AIUS|SyntaxError|true|only<img>|6|9"
    );
}

#[test]
fn shadow_dom_options_tree_queries_and_closed_visibility_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const host = document.createElement("div");
          host.id = "open-host";
          document.body.appendChild(host);
          const root = host.attachShadow({
            mode: "open",
            delegatesFocus: true,
            slotAssignment: "manual",
            serializable: true,
            clonable: true
          });
          root.innerHTML =
            '<section class="inside"><span id="deep">text</span></section>';
          const deep = root.getElementById("deep");
          const list = root.querySelectorAll(".inside > span");
          let duplicate = "";
          try { host.attachShadow({ mode: "open" }); }
          catch (error) { duplicate = error.name; }

          const closedHost = document.createElement("x-closed");
          document.body.appendChild(closedHost);
          const closed = closedHost.attachShadow({ mode: "closed" });
          closed.innerHTML = "<b>secret</b>";

          let invalidMode = "", invalidHost = "";
          try { document.createElement("div").attachShadow({ mode: "wrong" }); }
          catch (error) { invalidMode = error.name; }
          try { document.createElement("input").attachShadow({ mode: "open" }); }
          catch (error) { invalidHost = error.name; }

          return [
            host.shadowRoot === root,
            root.mode,
            root.host === host,
            root.delegatesFocus,
            root.slotAssignment,
            root.serializable,
            root.clonable,
            root.children === root.children,
            root.children.length,
            root.querySelector(".inside").tagName,
            list.constructor.name,
            list.length,
            deep.ownerDocument === document,
            deep.getRootNode() === root,
            deep.getRootNode({ composed: true }) === document,
            deep.isConnected,
            root.innerHTML,
            root.getHTML(),
            host.outerHTML,
            duplicate,
            closedHost.shadowRoot === null,
            closed.mode,
            closed.firstElementChild.textContent,
            invalidMode,
            invalidHost
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "true|open|true|true|manual|true|true|true|1|SECTION|NodeList|1|true|true|true|true|<section class=\"inside\"><span id=\"deep\">text</span></section>|<section class=\"inside\"><span id=\"deep\">text</span></section>|<div id=\"open-host\"></div>|NotSupportedError|true|closed|secret|TypeError|NotSupportedError"
    );
}

#[test]
fn live_dom_exotic_collections_dataset_and_extended_selectors_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const root = document.createElement("div");
          root.innerHTML =
            '<ul id="items">' +
              '<li class="item" data-code="Ab-one"><em class="mark"></em></li>' +
              '<li class="item skip" data-code="ab-two"></li>' +
              '<li class="item" data-code="Ab-three"></li>' +
              '<li class="item" data-code="zz-four"></li>' +
              '<li class="item" data-code="Ab-five"></li>' +
            '</ul>' +
            '<span id="named-child" name="lookup"></span><!-- ignored -->';
          document.body.appendChild(root);

          const childNodes = root.childNodes;
          const spans = root.getElementsByTagName("span");
          const names = document.getElementsByName("lookup");
          const firstSpan = spans[0];
          const added = document.createElement("span");
          added.id = "later";
          added.setAttribute("name", "lookup");
          root.appendChild(added);

          const dataset = added.dataset;
          added.setAttribute("data-user-id", "7");
          dataset.longName = 9;
          const datasetBeforeDelete = [
            dataset.userId,
            added.getAttribute("data-long-name"),
            Object.keys(dataset).join(","),
            "userId" in dataset,
            dataset === added.dataset,
            Object.prototype.toString.call(dataset)
          ].join(",");
          delete dataset.userId;

          const empty = document.createElement("p");
          empty.appendChild(document.createComment("comment"));
          root.appendChild(empty);

          return [
            childNodes === root.childNodes,
            childNodes[3] === added,
            spans[0] === firstSpan,
            spans[1] === added,
            spans.namedItem("later") === added,
            spans.later === added,
            root.children["named-child"] === firstSpan,
            names.constructor.name,
            names[1] === added,
            names.length,
            datasetBeforeDelete,
            added.hasAttribute("data-user-id"),
            root.querySelectorAll(
              'li:nth-child(2n+1):not(.skip)[data-code^="ab" i]'
            ).length,
            root.querySelector('li:has(> em.mark)') ===
              root.querySelector("li"),
            root.querySelectorAll("li:is(.skip, [data-code$='five'])").length,
            document.querySelector(":root") === document.documentElement,
            empty.matches(":empty"),
            root.querySelectorAll("li:only-child").length
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|true|true|true|true|true|NodeList|true|2|7,9,userId,longName,true,true,[object DOMStringMap]|false|3|true|2|true|true|0"
    );
}

#[test]
fn selection_namespace_and_document_tree_constraints_are_dom_consistent() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const selection = document.getSelection();
          const text = document.createTextNode("abcdef");
          const box = document.createElement("div");
          box.appendChild(text);
          document.body.appendChild(box);
          selection.collapse(text, 2);
          const collapsedRange = selection.getRangeAt(0);
          const shadow = box.attachShadow({mode: "open"});

          selection.setBaseAndExtent(text, 1, text, 4);
          const selectedRange = selection.getRangeAt(0);
          let badSelectionOffset = "";
          try { selection.collapse(text, 99); }
          catch (error) { badSelectionOffset = error.name; }
          let badRangeIndex = "";
          try { selection.getRangeAt(2); }
          catch (error) { badRangeIndex = error.name; }

          const xml = document.implementation.createDocument(null, "root");
          const root = xml.documentElement;
          root.setAttributeNS(
            "http://www.w3.org/2000/xmlns/",
            "xmlns:demo",
            "urn:demo"
          );
          const child = xml.createElementNS("urn:demo", "demo:item");
          root.appendChild(child);

          const replacement = xml.createElement("replacement");
          const selfResult = root.replaceChild(child, child);
          let secondRoot = "";
          try { xml.appendChild(replacement); }
          catch (error) { secondRoot = error.name; }

          const ordered = document.implementation.createDocument(null, "");
          const orderedRoot = ordered.createElement("root");
          ordered.appendChild(orderedRoot);
          let lateDoctype = "";
          try {
            ordered.appendChild(
              document.implementation.createDocumentType("root", "", "")
            );
          } catch (error) { lateDoctype = error.name; }

          return [
            document.getSelection() === selection,
            shadow.getSelection() === selection,
            collapsedRange.startContainer === text,
            collapsedRange.startOffset,
            collapsedRange.collapsed,
            selection.rangeCount,
            selectedRange.toString(),
            selection.anchorOffset,
            selection.focusOffset,
            badSelectionOffset,
            badRangeIndex,
            child.lookupNamespaceURI("demo"),
            child.lookupPrefix("urn:demo"),
            child.isDefaultNamespace("urn:demo"),
            selfResult === child,
            child.parentNode === root,
            secondRoot,
            lateDoctype
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "true|true|true|2|true|1|bcd|1|4|IndexSizeError|IndexSizeError|urn:demo|demo|false|true|true|HierarchyRequestError|HierarchyRequestError"
    );
}

#[test]
fn range_boundaries_and_contextual_fragments_preserve_dom_nodes() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const host = document.createElement("section");
          host.innerHTML = "<p>one</p><p>two</p>";
          document.body.appendChild(host);
          const firstText = host.firstElementChild.firstChild;
          const secondText = host.lastElementChild.firstChild;
          const range = document.createRange();
          range.setStart(firstText, 1);
          range.setEnd(secondText, 2);
          const fragment = range.createContextualFragment(
            '<b id="range-node">B</b><i>I</i><!--range-comment-->'
          );

          let badOffset = "", badDoctype = "", wrongDocument = "";
          try { range.setStart(firstText, 20); }
          catch (error) { badOffset = error.name; }
          try {
            range.setStart(
              document.implementation.createDocumentType("html", "", ""),
              0
            );
          }
          catch (error) { badDoctype = error.name; }
          const other = document.implementation.createHTMLDocument();
          try { range.comparePoint(other.body, 0); }
          catch (error) { wrongDocument = error.name; }

          range.setStart(secondText, 3);
          const collapsedForward = [
            range.collapsed,
            range.endContainer === secondText,
            range.endOffset
          ].join(",");
          range.setEnd(firstText, 0);
          const collapsedBackward = [
            range.collapsed,
            range.startContainer === firstText,
            range.startOffset
          ].join(",");

          return [
            fragment.constructor.name,
            fragment.childNodes.length,
            fragment.firstChild.tagName,
            fragment.firstChild.id,
            fragment.lastChild.nodeType,
            fragment.textContent,
            badOffset,
            badDoctype,
            wrongDocument,
            range.isPointInRange(other.body, 0),
            collapsedForward,
            collapsedBackward
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "DocumentFragment|3|B|range-node|8|BI|IndexSizeError|InvalidNodeTypeError|WrongDocumentError|false|true,true,3|true,true,0"
    );
}

#[test]
fn dom_parser_xml_serializer_namespaces_and_document_write_share_the_real_tree() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const parser = new DOMParser();
          const xml = parser.parseFromString(
            '<root xmlns="urn:root" xmlns:p="urn:part">' +
            '<p:item p:id="7"><![CDATA[x<y]]><?go now?></p:item>' +
            '</root>',
            'application/xml'
          );
          const root = xml.documentElement;
          const item = root.firstElementChild;
          const created = xml.createElementNS("urn:later", "q:later");
          created.textContent = "a&b";
          item.appendChild(created);

          const serializer = new XMLSerializer();
          const whole = serializer.serializeToString(xml);
          const isolated = serializer.serializeToString(item);

          const html = parser.parseFromString(
            '<main id="parsed"><b>value</b></main>',
            'text/html'
          );
          html.querySelector("b").textContent = "changed";
          const htmlSerialized = serializer.serializeToString(html);

          document.open();
          document.write('<html><head></head><body><section id="written">');
          document.writeln('<i>live</i>');
          document.write('</section></body></html>');
          document.close();

          return [
            root.namespaceURI,
            item.namespaceURI,
            item.getAttributeNS("urn:part", "id"),
            root.ownerDocument === xml && created.ownerDocument === xml,
            item.childNodes[0].nodeType + ":" + item.childNodes[1].nodeType,
            whole.includes('<![CDATA[x<y]]>') &&
              whole.includes('<?go now?>') &&
              whole.includes('<q:later xmlns:q="urn:later">a&amp;b</q:later>'),
            isolated.startsWith('<p:item xmlns:p="urn:part"'),
            html.querySelector("#parsed b").textContent,
            htmlSerialized.includes(
              '<html xmlns="http://www.w3.org/1999/xhtml">'
            ),
            document.querySelector("#written i").textContent.trim(),
            document.readyState
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "urn:root|urn:part|7|true|4:7|true|true|changed|true|live|complete"
    );
}

#[test]
fn range_content_operations_preserve_partial_dom_structure() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const host = document.createElement("div");
          host.innerHTML = "<p>ab<b>cd</b>ef</p><i>gh</i>";
          const start = host.firstChild.firstChild;
          const end = host.lastChild.firstChild;
          const range = document.createRange();
          range.setStart(start, 1);
          range.setEnd(end, 1);

          const cloned = range.cloneContents();
          const cloneHTML =
            cloned.firstChild.outerHTML + cloned.lastChild.outerHTML;
          const extracted = range.extractContents();
          const extractHTML =
            extracted.firstChild.outerHTML + extracted.lastChild.outerHTML;
          const remaining = host.innerHTML;
          const collapsed = [
            range.collapsed,
            range.startContainer === start,
            range.startOffset,
            range.endContainer === start,
            range.endOffset
          ].join(",");

          const all = document.createRange();
          all.selectNodeContents(host);
          const allClone = all.cloneContents();
          all.deleteContents();

          return [
            cloneHTML,
            extractHTML,
            remaining,
            collapsed,
            allClone.firstChild.outerHTML + allClone.lastChild.outerHTML,
            host.childNodes.length
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "<p>b<b>cd</b>ef</p><i>g</i>|<p>b<b>cd</b>ef</p><i>g</i>|<p>a</p><i>h</i>|true,true,1,true,1|<p>a</p><i>h</i>|0"
    );
}

#[test]
fn range_insert_and_surround_split_text_and_update_boundaries() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const p = document.createElement("p");
          p.textContent = "abc";
          const original = p.firstChild;
          const insertion = document.createRange();
          insertion.setStart(original, 1);
          insertion.collapse(true);
          const strong = document.createElement("strong");
          strong.textContent = "X";
          insertion.insertNode(strong);
          const insertionShape = [
            p.innerHTML,
            insertion.startContainer === original,
            insertion.startOffset,
            insertion.endContainer === p,
            insertion.endOffset
          ].join(",");

          const tail = p.lastChild;
          const wrapping = document.createRange();
          wrapping.setStart(tail, 0);
          wrapping.setEnd(tail, 1);
          const em = document.createElement("em");
          wrapping.surroundContents(em);
          const wrappingShape = [
            p.innerHTML,
            em.textContent,
            wrapping.startContainer === p,
            wrapping.endContainer === p,
            wrapping.endOffset - wrapping.startOffset
          ].join(",");

          const invalidHost = document.createElement("div");
          invalidHost.innerHTML = "<b>one</b><i>two</i>";
          const invalid = document.createRange();
          invalid.setStart(invalidHost.firstChild.firstChild, 1);
          invalid.setEnd(invalidHost.lastChild.firstChild, 1);
          let invalidName = "";
          try {
            invalid.surroundContents(document.createElement("u"));
          } catch (error) {
            invalidName = error.name;
          }
          return [insertionShape, wrappingShape, invalidName].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "a<strong>X</strong>bc,true,1,true,2|a<strong>X</strong><em>b</em>c,b,true,true,1|InvalidStateError"
    );
}

#[test]
fn element_independent_methods_keep_state_and_return_dom_interfaces() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const element = document.createElement("div");
          document.body.appendChild(element);
          element.setHTMLUnsafe("<span>one</span>");
          element.setHTML("<b>two</b>");
          const htmlSnapshot = element.getHTML();
          element.scroll(4, 5);
          element.scrollBy({ left: 3, top: 2 });
          element.setPointerCapture(19);
          const captured = element.hasPointerCapture(19);
          element.releasePointerCapture(19);
          const released = !element.hasPointerCapture(19);
          const fullscreen = element.requestFullscreen();
          const pointerLock = element.requestPointerLock();
          const animation = element.animate(
            [{ opacity: 0 }, { opacity: 1 }],
            { duration: 10 }
          );
          const manualAnimation = new Animation(
            new KeyframeEffect(
              element,
              [{ transform: "none" }, { transform: "scale(2)" }],
              { duration: 20 }
            ),
            document.timeline
          );
          manualAnimation.play();
          const animatedChild = document.createElement("span");
          element.appendChild(animatedChild);
          const childAnimation = animatedChild.animate(
            [{ opacity: 1 }, { opacity: 0.5 }],
            { duration: 30 }
          );
          const elementAnimations = element.getAnimations();
          const subtreeAnimations =
            element.getAnimations({ subtree: true });
          const documentAnimations = document.getAnimations();
          const animationTracking =
            elementAnimations.length === 2 &&
            elementAnimations[0] === animation &&
            elementAnimations[1] === manualAnimation &&
            !elementAnimations.includes(childAnimation) &&
            subtreeAnimations.length === 3 &&
            subtreeAnimations[2] === childAnimation &&
            documentAnimations.includes(animation) &&
            documentAnimations.includes(manualAnimation) &&
            documentAnimations.includes(childAnimation);
          animation.cancel();
          const cancellationTracking =
            !element.getAnimations().includes(animation) &&
            !document.getAnimations().includes(animation) &&
            element.getAnimations().includes(manualAnimation);

          const visibilityHost = document.createElement("section");
          const visibilityNode = document.createElement("div");
          visibilityHost.appendChild(visibilityNode);
          document.body.appendChild(visibilityHost);
          const detachedVisibility =
            document.createElement("div").checkVisibility() === false;
          visibilityNode.style.setProperty("visibility", "hidden");
          const visibilityOption =
            visibilityNode.checkVisibility() === true &&
            visibilityNode.checkVisibility({
              visibilityProperty: true
            }) === false;
          visibilityNode.style.removeProperty("visibility");
          visibilityNode.style.setProperty("opacity", "0");
          const opacityOption =
            visibilityNode.checkVisibility() === true &&
            visibilityNode.checkVisibility({
              opacityProperty: true
            }) === false;
          visibilityNode.style.removeProperty("opacity");
          visibilityHost.style.setProperty("display", "none");
          const hiddenByAncestor =
            visibilityNode.checkVisibility() === false;
          visibilityHost.style.removeProperty("display");
          visibilityNode.hidden = true;
          const hiddenAttribute =
            visibilityNode.checkVisibility() === false;
          visibilityNode.hidden = false;
          visibilityHost.style.setProperty(
            "content-visibility",
            "hidden"
          );
          const hiddenByContentVisibility =
            visibilityNode.checkVisibility() === false;
          visibilityHost.style.removeProperty("content-visibility");
          visibilityNode.style.setProperty("display", "contents");
          const displayContents =
            visibilityNode.checkVisibility() === false;
          return [
            htmlSnapshot,
            element.scrollLeft + ":" + element.scrollTop,
            captured && released,
            element.getBoundingClientRect() instanceof DOMRect,
            element.getClientRects() instanceof DOMRectList,
            fullscreen instanceof Promise &&
              document.fullscreenElement === element,
            pointerLock instanceof Promise &&
              document.pointerLockElement === element,
            animation instanceof Animation,
            element.computedStyleMap() instanceof StylePropertyMap,
            Array.isArray(element.getAnimations()),
            element.checkVisibility(),
            animationTracking,
            cancellationTracking,
            detachedVisibility,
            visibilityOption,
            opacityOption,
            hiddenByAncestor,
            hiddenAttribute,
            hiddenByContentVisibility,
            displayContents
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        concat!(
            "<b>two</b>|0:0|true|true|true|true|true|true|true|true|true|",
            "true|true|true|true|true|true|true|true|true"
        )
    );
}

#[test]
fn html_url_and_input_reflected_attributes_share_content_attribute_state() {
    let setup = r##"
        (() => {
          const anchor = document.createElement("a");
          anchor.href = "/x?q=1#h";
          const anchorForward = [
            anchor.getAttribute("href"),
            anchor.href,
            anchor.origin,
            anchor.pathname,
            anchor.search,
            anchor.hash,
            anchor.toString()
          ].join(",");
          anchor.setAttribute("href", "/y?q=2#z");
          const anchorReverse = [
            anchor.href,
            anchor.origin,
            anchor.pathname,
            anchor.search,
            anchor.hash
          ].join(",");

          const link = document.createElement("link");
          link.href = "/x.css";
          const script = document.createElement("script");
          script.src = "/x.js";
          const image = document.createElement("img");
          image.src = "/x.png";
          const form = document.createElement("form");
          form.action = "/go";

          const input = document.createElement("input");
          input.name = "query";
          input.required = true;
          input.type = "email";
          input.placeholder = "mail";
          input.disabled = true;
          const inputForward = [
            input.getAttribute("name"),
            input.hasAttribute("required"),
            input.getAttribute("type"),
            input.getAttribute("placeholder"),
            input.hasAttribute("disabled"),
            input.outerHTML
          ].join(",");
          input.setAttribute("name", "reverse");
          input.removeAttribute("required");
          input.setAttribute("type", "not-a-real-type");
          input.setAttribute("placeholder", "reverse-placeholder");
          input.removeAttribute("disabled");
          const inputReverse = [
            input.name,
            input.required,
            input.type,
            input.placeholder,
            input.disabled
          ].join(",");

          input.defaultValue = "initial";
          input.value = "dirty";
          input.setAttribute("value", "new-default");
          input.defaultChecked = true;
          input.checked = false;
          input.removeAttribute("checked");
          const dirty = [
            input.defaultValue,
            input.value,
            input.defaultChecked,
            input.checked
          ].join(",");

          return [
            anchorForward,
            anchorReverse,
            [
              link.getAttribute("href"), link.href,
              script.getAttribute("src"), script.src,
              image.getAttribute("src"), image.src,
              form.getAttribute("action"), form.action
            ].join(","),
            inputForward,
            inputReverse,
            dirty
          ].join("|");
        })()
    "##;
    let expected = concat!(
        "/x?q=1#h,https://sandbox.test/x?q=1#h,https://sandbox.test,/x,?q=1,#h,",
        "https://sandbox.test/x?q=1#h|",
        "https://sandbox.test/y?q=2#z,https://sandbox.test,/y,?q=2,#z|",
        "/x.css,https://sandbox.test/x.css,/x.js,https://sandbox.test/x.js,",
        "/x.png,https://sandbox.test/x.png,/go,https://sandbox.test/go|",
        "query,true,email,mail,true,",
        "<input name=\"query\" required=\"\" type=\"email\" placeholder=\"mail\" disabled=\"\">|",
        "reverse,false,text,reverse-placeholder,false|",
        "new-default,dirty,false,false"
    );

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    assert_eq!(text(&mut direct, setup), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    let actual = text(&mut traced, setup);
    assert_eq!(actual, expected, "{:#?}", traced.proxy_trace());
}

#[test]
fn inline_css_named_properties_cssom_and_style_attribute_stay_bidirectional() {
    let setup = r#"
        (() => {
          const element = document.createElement("div");
          const sameStyle = element.style === element.style;
          element.style.display = "none";
          const named = [
            element.style.display,
            element.style.getPropertyValue("display"),
            element.style.cssText,
            element.getAttribute("style"),
            element.outerHTML,
            element.checkVisibility(),
            "display" in element.style,
            sameStyle
          ].join("|");

          element.style.setProperty("color", "red", "important");
          const cssom = [
            element.style.color,
            element.style.getPropertyPriority("color"),
            element.style.length,
            element.style.item(0),
            element.style.item(1),
            element.getAttribute("style")
          ].join("|");

          element.setAttribute(
            "style",
            "opacity: 0; visibility: hidden; content-visibility: hidden"
          );
          const attribute = [
            element.style.opacity,
            element.style.visibility,
            element.style.contentVisibility,
            element.style.cssText,
            element.checkVisibility({ opacityProperty: true }),
            element.checkVisibility({ visibilityProperty: true })
          ].join("|");

          element.style.cssText = "display: block; background-color: blue";
          const cssText = [
            element.style.display,
            element.style.backgroundColor,
            element.getAttribute("style"),
            element.outerHTML
          ].join("|");

          element.removeAttribute("style");
          const removed = [
            element.style.cssText,
            element.style.display,
            element.getAttribute("style")
          ].join("|");
          return [named, cssom, attribute, cssText, removed].join("~");
        })()
    "#;
    let expected = concat!(
        "none|none|display: none;|display: none;|",
        "<div style=\"display: none;\"></div>|false|true|true~",
        "red|important|2|display|color|display: none; color: red !important;~",
        "0|hidden|hidden|opacity: 0; visibility: hidden; content-visibility: hidden;|",
        "false|false~",
        "block|blue|display: block; background-color: blue;|",
        "<div style=\"display: block; background-color: blue;\"></div>~",
        "||"
    );

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    assert_eq!(text(&mut direct, setup), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    let actual = text(&mut traced, setup);
    let trace = traced.proxy_trace();
    assert_eq!(actual, expected, "{trace:#?}");
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "set" && entry.api.ends_with(".style.display") })
    );
    assert!(
        trace.iter().any(|entry| {
            entry.operation == "call" && entry.api.ends_with(".style.setProperty")
        })
    );
}

#[test]
fn form_owners_labels_and_data_lists_follow_the_live_document_tree() {
    let setup = r#"
        (() => {
          const form = document.createElement("form");
          form.id = "owner";
          document.body.appendChild(form);

          const input = document.createElement("input");
          input.name = "query";
          const button = document.createElement("button");
          const select = document.createElement("select");
          const textarea = document.createElement("textarea");
          const fieldset = document.createElement("fieldset");
          const output = document.createElement("output");
          const controls = [input, button, select, textarea, fieldset, output];
          for (const control of controls) {
            control.setAttribute("form", "owner");
            document.body.appendChild(control);
          }
          const elements = form.elements;
          const externalOwners =
            controls.every(control => control.form === form) &&
            form.length === 6 &&
            elements.length === 6 &&
            elements[0] === input &&
            elements[5] === output &&
            elements.namedItem("query") === input;

          input.setAttribute("form", "missing");
          const invalidOwner =
            input.form === null &&
            form.length === 5 &&
            elements.length === 5;
          input.setAttribute("form", "owner");
          const restoredOwner =
            input.form === form &&
            elements.length === 6 &&
            elements[0] === input;

          const nested = document.createElement("input");
          form.appendChild(nested);
          const ancestorOwner =
            nested.form === form &&
            elements.length === 7 &&
            elements[0] === nested;
          nested.setAttribute("form", "");
          const explicitEmptyOwner =
            nested.form === null &&
            elements.length === 6;
          nested.removeAttribute("form");

          input.id = "labelled";
          const explicitLabel = document.createElement("label");
          explicitLabel.htmlFor = "labelled";
          const implicitLabel = document.createElement("label");
          const implicitControl = document.createElement("input");
          implicitLabel.appendChild(implicitControl);
          document.body.append(explicitLabel, implicitLabel);
          const labels = input.labels;
          const labelRelations =
            explicitLabel.getAttribute("for") === "labelled" &&
            explicitLabel.control === input &&
            implicitLabel.control === implicitControl &&
            labels === input.labels &&
            labels.length === 1 &&
            labels[0] === explicitLabel &&
            implicitControl.labels.length === 1 &&
            implicitControl.labels[0] === implicitLabel;
          explicitLabel.htmlFor = "missing";
          const labelMutation =
            explicitLabel.control === null &&
            labels.length === 0;
          explicitLabel.setAttribute("for", "labelled");
          const labelRestore =
            explicitLabel.htmlFor === "labelled" &&
            explicitLabel.control === input &&
            labels.length === 1;

          const dataList = document.createElement("datalist");
          dataList.id = "choices";
          const option = document.createElement("option");
          option.value = "one";
          dataList.appendChild(option);
          document.body.appendChild(dataList);
          input.setAttribute("list", "choices");
          const options = dataList.options;
          const listRelation =
            input.list === dataList &&
            options.length === 1 &&
            options[0] === option;
          dataList.id = "renamed";
          const listMutation =
            input.list === null &&
            options.length === 1;
          input.setAttribute("list", "renamed");
          const listRestore =
            input.list === dataList &&
            dataList.options === options;

          return [
            externalOwners,
            invalidOwner,
            restoredOwner,
            ancestorOwner,
            explicitEmptyOwner,
            labelRelations,
            labelMutation,
            labelRestore,
            listRelation,
            listMutation,
            listRestore
          ].join("|");
        })()
    "#;
    let expected = "true|true|true|true|true|true|true|true|true|true|true";

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    assert_eq!(text(&mut direct, setup), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    let actual = text(&mut traced, setup);
    let trace = traced.proxy_trace();
    assert_eq!(actual, expected, "{trace:#?}");
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "get" && entry.api.ends_with(".elements"))
    );
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "get" && entry.api.ends_with(".labels"))
    );
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "get" && entry.api.ends_with(".list"))
    );
}

#[test]
fn form_control_and_image_idl_attributes_reflect_content_attributes_bidirectionally() {
    let setup = r##"
        (() => {
          const form = document.createElement("form");
          form.id = "reflected-form";
          form.name = "search";
          form.acceptCharset = "utf-8";
          form.target = "_blank";
          form.noValidate = true;
          form.method = "not-a-method";
          form.enctype = "not-an-enctype";
          form.autocomplete = "invalid";
          form.rel = "noopener external";
          const formForward =
            form.getAttribute("name") === "search" &&
            form.getAttribute("accept-charset") === "utf-8" &&
            form.getAttribute("target") === "_blank" &&
            form.hasAttribute("novalidate") &&
            form.getAttribute("method") === "not-a-method" &&
            form.method === "get" &&
            form.getAttribute("enctype") === "not-an-enctype" &&
            form.enctype === "application/x-www-form-urlencoded" &&
            form.getAttribute("autocomplete") === "invalid" &&
            form.autocomplete === "on" &&
            form.getAttribute("rel") === "noopener external" &&
            form.relList.contains("external");
          form.setAttribute("name", "reverse");
          form.removeAttribute("novalidate");
          form.setAttribute("method", "POST");
          form.relList.add("nofollow");
          const formReverse =
            form.name === "reverse" &&
            !form.noValidate &&
            form.method === "post" &&
            form.getAttribute("rel").includes("nofollow");

          const select = document.createElement("select");
          select.name = "choice";
          select.disabled = true;
          select.required = true;
          select.multiple = true;
          select.size = 4;
          select.autocomplete = "shipping country";
          const selectForward =
            select.getAttribute("name") === "choice" &&
            select.hasAttribute("disabled") &&
            select.hasAttribute("required") &&
            select.hasAttribute("multiple") &&
            select.getAttribute("size") === "4" &&
            select.getAttribute("autocomplete") === "shipping country" &&
            select.type === "select-multiple";
          select.setAttribute("name", "reverse-choice");
          select.removeAttribute("disabled");
          select.removeAttribute("multiple");
          select.setAttribute("size", "2");
          const selectReverse =
            select.name === "reverse-choice" &&
            !select.disabled &&
            !select.multiple &&
            select.size === 2 &&
            select.type === "select-one";

          const textarea = document.createElement("textarea");
          textarea.name = "notes";
          textarea.placeholder = "type here";
          textarea.disabled = true;
          textarea.readOnly = true;
          textarea.required = true;
          textarea.cols = 40;
          textarea.rows = 5;
          textarea.maxLength = 80;
          textarea.minLength = 2;
          textarea.wrap = "hard";
          const textareaForward =
            textarea.getAttribute("name") === "notes" &&
            textarea.getAttribute("placeholder") === "type here" &&
            textarea.hasAttribute("disabled") &&
            textarea.hasAttribute("readonly") &&
            textarea.hasAttribute("required") &&
            textarea.getAttribute("cols") === "40" &&
            textarea.getAttribute("rows") === "5" &&
            textarea.getAttribute("maxlength") === "80" &&
            textarea.getAttribute("minlength") === "2" &&
            textarea.getAttribute("wrap") === "hard";
          textarea.setAttribute("name", "reverse-notes");
          textarea.removeAttribute("disabled");
          textarea.setAttribute("cols", "30");
          textarea.defaultValue = "initial";
          const cleanValue =
            textarea.textContent === "initial" &&
            textarea.value === "initial";
          textarea.value = "dirty";
          textarea.textContent = "new default";
          const dirtyValue =
            textarea.defaultValue === "new default" &&
            textarea.value === "dirty";
          form.appendChild(textarea);
          document.body.appendChild(form);
          form.reset();
          const textareaReverse =
            textarea.name === "reverse-notes" &&
            !textarea.disabled &&
            textarea.cols === 30 &&
            textarea.value === "new default";

          const option = document.createElement("option");
          option.text = "Visible";
          const optionFallback =
            option.textContent === "Visible" &&
            option.text === "Visible" &&
            option.value === "Visible" &&
            option.label === "Visible" &&
            option.index === -1;
          option.value = "";
          option.label = "Label";
          option.disabled = true;
          option.defaultSelected = true;
          select.appendChild(option);
          select.setAttribute("form", "reflected-form");
          document.body.appendChild(select);
          const optionForward =
            option.getAttribute("value") === "" &&
            option.value === "" &&
            option.getAttribute("label") === "Label" &&
            option.label === "Label" &&
            option.hasAttribute("disabled") &&
            option.hasAttribute("selected") &&
            option.form === form &&
            option.index === 0;
          option.removeAttribute("label");
          option.removeAttribute("disabled");
          const optionReverse =
            option.label === "Visible" &&
            !option.disabled;

          const image = new Image(10, 20);
          image.alt = "preview";
          image.srcset = "one.png 1x";
          image.sizes = "100vw";
          image.useMap = "#map";
          image.isMap = true;
          image.referrerPolicy = "invalid-policy";
          image.decoding = "invalid-decoding";
          image.fetchPriority = "invalid-priority";
          image.loading = "invalid-loading";
          image.crossOrigin = "invalid-cross-origin";
          const imageForward =
            image.getAttribute("width") === "10" &&
            image.getAttribute("height") === "20" &&
            image.width === 10 &&
            image.height === 20 &&
            image.getAttribute("alt") === "preview" &&
            image.getAttribute("srcset") === "one.png 1x" &&
            image.getAttribute("sizes") === "100vw" &&
            image.getAttribute("usemap") === "#map" &&
            image.hasAttribute("ismap") &&
            image.getAttribute("referrerpolicy") === "invalid-policy" &&
            image.referrerPolicy === "" &&
            image.getAttribute("decoding") === "invalid-decoding" &&
            image.decoding === "auto" &&
            image.getAttribute("fetchpriority") === "invalid-priority" &&
            image.fetchPriority === "auto" &&
            image.getAttribute("loading") === "invalid-loading" &&
            image.loading === "auto" &&
            image.getAttribute("crossorigin") === "invalid-cross-origin" &&
            image.crossOrigin === "anonymous";
          image.setAttribute("width", "33");
          image.removeAttribute("ismap");
          image.setAttribute("crossorigin", "use-credentials");
          const imageReverse =
            image.width === 33 &&
            !image.isMap &&
            image.crossOrigin === "use-credentials";

          return [
            formForward,
            formReverse,
            selectForward,
            selectReverse,
            textareaForward,
            cleanValue,
            dirtyValue,
            textareaReverse,
            optionFallback,
            optionForward,
            optionReverse,
            imageForward,
            imageReverse
          ].join("|");
        })()
    "##;
    let expected = "true|true|true|true|true|true|true|true|true|true|true|true|true";

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    assert_eq!(text(&mut direct, setup), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    let actual = text(&mut traced, setup);
    assert_eq!(actual, expected, "{:#?}", traced.proxy_trace());
}

#[test]
fn long_tail_reflection_css_properties_and_detached_relationships_are_live() {
    let setup = r##"
        (() => {
          const anchor = document.createElement("a");
          anchor.target = "_blank";
          anchor.download = "file.bin";
          anchor.ping = "/audit";
          anchor.rel = "noopener";
          anchor.hreflang = "zh-CN";
          anchor.type = "text/html";
          anchor.referrerPolicy = "origin";
          anchor.coords = "1,2,3";
          anchor.charset = "utf-8";
          anchor.name = "legacy";
          anchor.rev = "made";
          anchor.shape = "rect";
          anchor.hrefTranslate = "en";
          anchor.attributionSrc = "/source";
          anchor.text = "linked text";
          const anchorForward = [
            ["target", "_blank"],
            ["download", "file.bin"],
            ["ping", "/audit"],
            ["rel", "noopener"],
            ["hreflang", "zh-CN"],
            ["type", "text/html"],
            ["referrerpolicy", "origin"],
            ["coords", "1,2,3"],
            ["charset", "utf-8"],
            ["name", "legacy"],
            ["rev", "made"],
            ["shape", "rect"],
            ["hreftranslate", "en"],
            ["attributionsrc", "/source"]
          ].every(([name, value]) => anchor.getAttribute(name) === value) &&
            anchor.textContent === "linked text" &&
            anchor.relList.contains("noopener");
          anchor.setAttribute("target", "_self");
          anchor.setAttribute("download", "reverse.bin");
          anchor.setAttribute("ping", "/reverse");
          anchor.setAttribute("rel", "external");
          anchor.setAttribute("hreflang", "fr");
          anchor.setAttribute("type", "application/json");
          anchor.setAttribute("referrerpolicy", "same-origin");
          anchor.textContent = "reverse text";
          const anchorReverse =
            anchor.target === "_self" &&
            anchor.download === "reverse.bin" &&
            anchor.ping === "/reverse" &&
            anchor.rel === "external" &&
            anchor.relList.contains("external") &&
            anchor.hreflang === "fr" &&
            anchor.type === "application/json" &&
            anchor.referrerPolicy === "same-origin" &&
            anchor.text === "reverse text";

          const input = document.createElement("input");
          input.accept = "image/png";
          input.alt = "upload";
          input.autocomplete = "email";
          input.dirName = "direction";
          input.formAction = "/submit";
          input.formEnctype = "multipart/form-data";
          input.formMethod = "post";
          input.formNoValidate = true;
          input.formTarget = "_blank";
          input.height = 12;
          input.max = "10";
          input.maxLength = 8;
          input.min = "1";
          input.minLength = 2;
          input.multiple = true;
          input.pattern = "[a-z]+";
          input.readOnly = true;
          input.size = 25;
          input.src = "/button.png";
          input.step = "2";
          input.width = 30;
          input.align = "left";
          input.useMap = "#upload-map";
          input.webkitdirectory = true;
          input.incremental = true;
          const inputForward =
            input.getAttribute("accept") === "image/png" &&
            input.getAttribute("alt") === "upload" &&
            input.getAttribute("autocomplete") === "email" &&
            input.getAttribute("dirname") === "direction" &&
            input.getAttribute("formaction") === "/submit" &&
            input.formAction === "https://sandbox.test/submit" &&
            input.getAttribute("formenctype") === "multipart/form-data" &&
            input.getAttribute("formmethod") === "post" &&
            input.hasAttribute("formnovalidate") &&
            input.getAttribute("formtarget") === "_blank" &&
            input.getAttribute("height") === "12" &&
            input.getAttribute("max") === "10" &&
            input.getAttribute("maxlength") === "8" &&
            input.getAttribute("min") === "1" &&
            input.getAttribute("minlength") === "2" &&
            input.hasAttribute("multiple") &&
            input.getAttribute("pattern") === "[a-z]+" &&
            input.hasAttribute("readonly") &&
            input.getAttribute("size") === "25" &&
            input.getAttribute("src") === "/button.png" &&
            input.src === "https://sandbox.test/button.png" &&
            input.getAttribute("step") === "2" &&
            input.getAttribute("width") === "30" &&
            input.getAttribute("align") === "left" &&
            input.getAttribute("usemap") === "#upload-map" &&
            input.hasAttribute("webkitdirectory") &&
            input.hasAttribute("incremental");
          input.setAttribute("accept", "text/plain");
          input.removeAttribute("formnovalidate");
          input.setAttribute("max", "20");
          input.removeAttribute("multiple");
          input.setAttribute("src", "/reverse.png");
          const inputReverse =
            input.accept === "text/plain" &&
            !input.formNoValidate &&
            input.max === "20" &&
            !input.multiple &&
            input.src === "https://sandbox.test/reverse.png";

          const styled = document.createElement("div");
          styled.style.animationName = "spin";
          styled.style.gridTemplateColumns = "1fr 2fr";
          styled.style.touchAction = "none";
          styled.style.webkitLineClamp = "2";
          const unknownBefore = "fooBar" in styled.style;
          styled.style.fooBar = "ordinary";
          const cssLongTail =
            styled.style.getPropertyValue("animation-name") === "spin" &&
            styled.style.getPropertyValue("grid-template-columns") === "1fr 2fr" &&
            styled.style.getPropertyValue("touch-action") === "none" &&
            styled.style.getPropertyValue("-webkit-line-clamp") === "2" &&
            styled.getAttribute("style").includes("animation-name: spin") &&
            !unknownBefore &&
            styled.style.fooBar === "ordinary" &&
            Object.prototype.hasOwnProperty.call(styled.style, "fooBar") &&
            styled.style.getPropertyValue("foo-bar") === "";
          const wordStyle = document.createElement("div");
          wordStyle.style.animation = "spin 1s";
          wordStyle.style.columns = "2";
          wordStyle.style.fill = "red";
          wordStyle.style.stroke = "blue";
          wordStyle.style.offset = "path('M0 0')";
          wordStyle.style.page = "chapter";
          const cssSingleWords =
            wordStyle.style.getPropertyValue("animation") === "spin 1s" &&
            wordStyle.style.getPropertyValue("columns") === "2" &&
            wordStyle.style.getPropertyValue("fill") === "red" &&
            wordStyle.style.getPropertyValue("stroke") === "blue" &&
            wordStyle.style.getPropertyValue("offset") === "path('M0 0')" &&
            wordStyle.style.getPropertyValue("page") === "chapter";
          const invalidStyle = document.createElement("div");
          const invalidBefore =
            !("fooBar" in invalidStyle.style) &&
            !("notARealProperty" in invalidStyle.style) &&
            !("not-real-prop" in invalidStyle.style);
          invalidStyle.style.fooBar = "x";
          invalidStyle.style.notARealProperty = "y";
          invalidStyle.style["not-real-prop"] = "z";
          invalidStyle.style.setProperty("also-not-real", "value");
          invalidStyle.style.cssText = "still-not-real: value";
          const cssInvalidNames =
            invalidBefore &&
            invalidStyle.style.fooBar === "x" &&
            invalidStyle.style.notARealProperty === "y" &&
            invalidStyle.style["not-real-prop"] === "z" &&
            invalidStyle.style.getPropertyValue("foo-bar") === "" &&
            invalidStyle.style.getPropertyValue("not-a-real-property") === "" &&
            invalidStyle.style.getPropertyValue("not-real-prop") === "" &&
            invalidStyle.style.getPropertyValue("also-not-real") === "" &&
            invalidStyle.style.getPropertyValue("still-not-real") === "" &&
            invalidStyle.style.cssText === "" &&
            invalidStyle.getAttribute("style") === null;

          const detachedForm = document.createElement("form");
          const detachedInput = document.createElement("input");
          detachedForm.appendChild(detachedInput);
          const detachedFormBefore =
            detachedInput.form === detachedForm &&
            detachedForm.elements.length === 1 &&
            detachedForm.length === 1;
          document.body.appendChild(detachedForm);
          const detachedFormAttached = detachedForm.elements.length === 1;
          detachedForm.remove();
          const detachedFormAfter =
            detachedInput.form === detachedForm &&
            detachedForm.elements.length === 1;

          const implicitLabel = document.createElement("label");
          const implicitInput = document.createElement("input");
          implicitLabel.appendChild(implicitInput);
          const detachedImplicitLabel =
            implicitLabel.control === implicitInput &&
            implicitInput.labels.length === 1 &&
            implicitInput.labels[0] === implicitLabel;

          const labelRoot = document.createElement("div");
          const explicitLabel = document.createElement("label");
          const explicitInput = document.createElement("input");
          explicitLabel.htmlFor = "detached-control";
          explicitInput.id = "detached-control";
          labelRoot.append(explicitLabel, explicitInput);
          const detachedExplicitLabel =
            explicitLabel.control === explicitInput &&
            explicitInput.labels.length === 1 &&
            explicitInput.labels[0] === explicitLabel;

          const listRoot = document.createElement("div");
          const detachedList = document.createElement("datalist");
          const detachedListInput = document.createElement("input");
          detachedList.id = "detached-list";
          detachedListInput.setAttribute("list", "detached-list");
          listRoot.append(detachedListInput, detachedList);
          const detachedDataList = detachedListInput.list === detachedList;

          const referenceRoot = document.createElement("div");
          const interest = document.createElement("a");
          const popoverInput = document.createElement("input");
          const firstTarget = document.createElement("button");
          const secondTarget = document.createElement("button");
          firstTarget.id = "first-target";
          secondTarget.id = "second-target";
          referenceRoot.append(interest, popoverInput, firstTarget, secondTarget);
          interest.setAttribute("interestfor", "first-target");
          popoverInput.setAttribute("popovertarget", "first-target");
          const reflectedElementAttributes =
            interest.interestForElement === firstTarget &&
            popoverInput.popoverTargetElement === firstTarget;
          interest.interestForElement = secondTarget;
          popoverInput.popoverTargetElement = secondTarget;
          const reflectedElementSetters =
            interest.getAttribute("interestfor") === "" &&
            popoverInput.getAttribute("popovertarget") === "" &&
            interest.interestForElement === secondTarget &&
            popoverInput.popoverTargetElement === secondTarget;
          interest.setAttribute("interestfor", "first-target");
          popoverInput.setAttribute("popovertarget", "first-target");
          let interestBrand = "";
          try { interest.interestForElement = {}; }
          catch (error) { interestBrand = error.name; }
          const reflectedElementMutation =
            interest.interestForElement === firstTarget &&
            popoverInput.popoverTargetElement === firstTarget &&
            interestBrand === "TypeError";

          return [
            anchorForward,
            anchorReverse,
            inputForward,
            inputReverse,
            cssLongTail,
            cssSingleWords,
            cssInvalidNames,
            detachedFormBefore,
            detachedFormAttached,
            detachedFormAfter,
            detachedImplicitLabel,
            detachedExplicitLabel,
            detachedDataList,
            reflectedElementAttributes,
            reflectedElementSetters,
            reflectedElementMutation
          ].join("|");
        })()
    "##;
    let expected =
        "true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true";

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    assert_eq!(text(&mut direct, setup), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    let actual = text(&mut traced, setup);
    assert_eq!(actual, expected, "{:#?}", traced.proxy_trace());
}

#[test]
fn traversal_filters_skip_reject_and_follow_live_tree_mutations() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const root = document.createElement("main");
          const skipped = document.createElement("div");
          skipped.id = "skip";
          const throughSkip = document.createElement("span");
          throughSkip.id = "through-skip";
          skipped.appendChild(throughSkip);
          const rejected = document.createElement("section");
          rejected.id = "reject";
          const underReject = document.createElement("b");
          rejected.appendChild(underReject);
          const finalNode = document.createElement("p");
          root.append(skipped, rejected, finalNode);

          const filter = {
            acceptNode(node) {
              if (node.id === "skip") return NodeFilter.FILTER_SKIP;
              if (node.id === "reject") return NodeFilter.FILTER_REJECT;
              return NodeFilter.FILTER_ACCEPT;
            }
          };
          const walker = document.createTreeWalker(
            root,
            NodeFilter.SHOW_ELEMENT,
            filter
          );
          const first = walker.firstChild();
          const sibling = walker.nextSibling();
          const parent = walker.parentNode();
          walker.currentNode = root;
          const nextOne = walker.nextNode();
          const nextTwo = walker.nextNode();
          const noRejectedDescendant = walker.nextNode();

          const dynamicRoot = document.createElement("div");
          const initial = document.createElement("i");
          dynamicRoot.appendChild(initial);
          const iterator = document.createNodeIterator(
            dynamicRoot,
            NodeFilter.SHOW_ELEMENT
          );
          const iterRoot = iterator.nextNode();
          const iterInitial = iterator.nextNode();
          const inserted = document.createElement("u");
          dynamicRoot.appendChild(inserted);
          const iterInserted = iterator.nextNode();

          return [
            first === throughSkip,
            sibling === finalNode,
            parent === root,
            nextOne === throughSkip,
            nextTwo === finalNode,
            noRejectedDescendant === null,
            iterRoot === dynamicRoot,
            iterInitial === initial,
            iterInserted === inserted,
            iterator.referenceNode === inserted,
            iterator.pointerBeforeReferenceNode === false
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|true|true|true|true|true|true|true|true|true"
    );
}

#[test]
fn shadow_slots_assign_light_dom_nodes_and_expose_assigned_slot_relationships() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const host = document.createElement("div");
          const named = document.createElement("span");
          named.slot = "named";
          const text = document.createTextNode("default");
          host.append(named, text);
          const shadow = host.attachShadow({ mode: "open" });
          shadow.innerHTML =
            '<slot name="named"></slot><slot><i>fallback</i></slot>';
          const slots = shadow.querySelectorAll("slot");

          const manualHost = document.createElement("div");
          const manualChild = document.createElement("b");
          manualChild.slot = "unused";
          manualHost.appendChild(manualChild);
          const manualShadow = manualHost.attachShadow({
            mode: "open",
            slotAssignment: "manual"
          });
          manualShadow.innerHTML = "<slot></slot>";
          const manualSlot = manualShadow.querySelector("slot");
          const beforeManual = manualChild.assignedSlot;
          manualSlot.assign(manualChild);

          return [
            named.assignedSlot === slots[0],
            text.assignedSlot === slots[1],
            slots[0].assignedNodes()[0] === named,
            slots[0].assignedElements()[0] === named,
            slots[1].assignedNodes()[0] === text,
            slots[1].assignedNodes({ flatten: true })[0] === text,
            beforeManual === null,
            manualChild.assignedSlot === manualSlot,
            manualSlot.assignedNodes()[0] === manualChild
          ].join("|");
        })()
        "##,
    );
    assert_eq!(result, "true|true|true|true|true|true|true|true|true");
}

#[test]
fn aria_properties_reflect_attributes_and_element_reference_relationships() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const owner = document.createElement("div");
          const first = document.createElement("span");
          const second = document.createElement("span");
          first.id = "first-ref";
          second.id = "second-ref";
          owner.append(first, second);
          document.body.appendChild(owner);

          const initiallyNull = owner.ariaAtomic === null;
          owner.ariaAtomic = true;
          const stringReflection =
            owner.getAttribute("aria-atomic") === "true" &&
            owner.ariaAtomic === "true";
          owner.ariaAtomic = null;
          const removal = !owner.hasAttribute("aria-atomic");

          owner.setAttribute("aria-activedescendant", "first-ref");
          const resolvedSingle = owner.ariaActiveDescendantElement === first;
          owner.ariaActiveDescendantElement = second;
          const assignedSingle =
            owner.ariaActiveDescendantElement === second &&
            owner.getAttribute("aria-activedescendant") === "second-ref";

          owner.setAttribute("aria-controls", "first-ref second-ref");
          const resolvedMany =
            owner.ariaControlsElements[0] === first &&
            owner.ariaControlsElements[1] === second;
          owner.ariaControlsElements = [second, first];
          const assignedMany =
            owner.ariaControlsElements[0] === second &&
            owner.getAttribute("aria-controls") === "second-ref first-ref";

          return [
            initiallyNull,
            stringReflection,
            removal,
            resolvedSingle,
            assignedSingle,
            resolvedMany,
            assignedMany
          ].join("|");
        })()
        "##,
    );
    assert_eq!(result, "true|true|true|true|true|true|true");
}

#[test]
fn document_core_properties_are_tree_backed_and_keep_browser_interface_shapes() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const children = document.children;
          document.title = "DOM title";
          document.dir = "rtl";

          const face = new FontFace("audit-font", "url(audit.woff2)");
          const fonts = document.fonts;
          const initialFontSize = fonts.size;
          const addResult = fonts.add(face);
          const fontAdded =
            addResult === fonts && fonts.size === initialFontSize + 1 &&
            fonts.has(face);
          const fontDeleted = fonts.delete(face) && !fonts.has(face);

          const target = document.createElement("section");
          document.body.appendChild(target);
          target.requestFullscreen();
          const enteredFullscreen =
            document.fullscreen && document.fullscreenElement === target &&
            document.webkitFullscreenElement === target;
          const exitResult = document.exitFullscreen();

          return [
            document.defaultView === window,
            children === document.children,
            children instanceof HTMLCollection,
            document.implementation instanceof DOMImplementation,
            document.styleSheets instanceof StyleSheetList,
            fonts.constructor.name === "FontFaceSet" &&
              typeof FontFaceSet === "undefined",
            fonts.ready instanceof Promise,
            fonts.status,
            fontAdded,
            fontDeleted,
            document.title,
            document.querySelector("title").textContent,
            document.documentElement.getAttribute("dir"),
            document.scrollingElement === document.documentElement,
            enteredFullscreen,
            exitResult instanceof Promise,
            document.fullscreenElement === null
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|true|true|true|true|true|loaded|true|true|DOM title|DOM title|rtl|true|true|true|true"
    );
}

#[test]
fn element_attribute_algorithms_validate_namespaces_case_and_attr_ownership() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const html = document.createElement("div");
          html.setAttribute("DATA-Mixed", "one");
          const htmlCase =
            html.getAttributeNames()[0] === "data-mixed" &&
            html.getAttribute("DATA-MIXED") === "one";
          const preservedToggle =
            html.toggleAttribute("data-mixed", true) &&
            html.getAttribute("data-mixed") === "one";

          const svg = document.createElementNS(
            "http://www.w3.org/2000/svg",
            "svg"
          );
          svg.setAttribute("viewBox", "0 0 1 1");
          const svgCase =
            svg.getAttribute("viewBox") === "0 0 1 1" &&
            svg.getAttribute("viewbox") === null;

          svg.setAttributeNS("urn:first", "a:value", "first");
          svg.setAttributeNS("urn:first", "b:value", "second");
          const namespaceReplacement =
            svg.attributes.length === 2 &&
            svg.getAttributeNS("urn:first", "value") === "second" &&
            svg.getAttributeNodeNS("urn:first", "value").prefix === "b";

          const attr = document.createAttribute("owned");
          attr.value = "yes";
          const replaced = html.setAttributeNode(attr);
          const attrAttached =
            replaced === null && attr.ownerElement === html &&
            html.getAttribute("owned") === "yes";
          const removed = html.removeAttributeNode(attr);
          const attrDetached =
            removed === attr && attr.ownerElement === null &&
            !html.hasAttribute("owned");

          let invalid = "", namespaceError = "";
          try { html.setAttribute("not valid", "x"); }
          catch (error) { invalid = error.name; }
          try { html.setAttributeNS(null, "x:value", "x"); }
          catch (error) { namespaceError = error.name; }

          return [
            htmlCase,
            preservedToggle,
            svgCase,
            namespaceReplacement,
            attrAttached,
            attrDetached,
            invalid,
            namespaceError,
            html.matches("div[data-mixed]"),
            html.closest("div") === html,
            html.webkitMatchesSelector("div")
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|true|true|true|true|InvalidCharacterError|NamespaceError|true|true|true"
    );
}

#[test]
fn live_ranges_track_tree_character_data_split_and_normalize_mutations() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const host = document.createElement("div");
          const a = document.createElement("a");
          const b = document.createElement("b");
          const c = document.createElement("i");
          const bText = document.createTextNode("inside");
          b.appendChild(bText);
          host.append(a, b, c);

          const childRange = document.createRange();
          childRange.setStart(host, 2);
          childRange.setEnd(host, 3);
          const x = document.createElement("u");
          host.insertBefore(x, b);
          const afterInsert =
            childRange.startOffset === 3 && childRange.endOffset === 4;
          host.removeChild(a);
          const afterRemove =
            childRange.startOffset === 2 && childRange.endOffset === 3;

          const removedRange = document.createRange();
          removedRange.setStart(bText, 1);
          removedRange.setEnd(bText, 4);
          host.removeChild(b);
          const removedSubtree =
            removedRange.startContainer === host &&
            removedRange.endContainer === host &&
            removedRange.startOffset === 1 &&
            removedRange.endOffset === 1;

          const data = document.createTextNode("abcd");
          const dataRange = document.createRange();
          dataRange.setStart(data, 1);
          dataRange.setEnd(data, 4);
          data.replaceData(1, 2, "XYZ");
          const characterEdit =
            data.data === "aXYZd" &&
            dataRange.startOffset === 1 &&
            dataRange.endOffset === 5;

          const split = document.createTextNode("abcdef");
          const splitRange = document.createRange();
          splitRange.setStart(split, 1);
          splitRange.setEnd(split, 5);
          const splitTail = split.splitText(3);
          const splitTracking =
            splitRange.startContainer === split &&
            splitRange.startOffset === 1 &&
            splitRange.endContainer === splitTail &&
            splitRange.endOffset === 2;

          const normalizeHost = document.createElement("div");
          const left = document.createTextNode("ab");
          const right = document.createTextNode("cd");
          normalizeHost.append(left, right);
          const normalizeRange = document.createRange();
          normalizeRange.setStart(right, 1);
          normalizeRange.collapse(true);
          normalizeHost.normalize();
          const normalizeTracking =
            normalizeHost.childNodes.length === 1 &&
            left.data === "abcd" &&
            normalizeRange.startContainer === left &&
            normalizeRange.startOffset === 3;

          const staticText = document.createTextNode("fixed");
          const staticRange = new StaticRange({
            startContainer: staticText,
            startOffset: 4,
            endContainer: staticText,
            endOffset: 4
          });
          staticText.data = "x";
          const staticUnchanged = staticRange.startOffset === 4;

          return [
            afterInsert,
            afterRemove,
            removedSubtree,
            characterEdit,
            splitTracking,
            normalizeTracking,
            staticUnchanged
          ].join("|");
        })()
        "##,
    );
    assert_eq!(result, "true|true|true|true|true|true|true");
}

#[test]
fn dom_collections_focus_iterators_observers_and_auxiliary_factories_stay_live() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const element = document.createElement("div");
          document.body.appendChild(element);
          const classList = element.classList;
          element.setAttribute("class", "one two");
          const setSync =
            classList === element.classList &&
            classList.length === 2 &&
            classList.contains("two");
          element.removeAttribute("class");
          const removeSync = classList.length === 0 && classList.value === "";

          element.focus();
          const focusSync = document.activeElement === element;
          element.blur();
          const blurSync = document.activeElement === document.body;

          const root = document.createElement("section");
          const first = document.createElement("a");
          const removed = document.createElement("b");
          const nested = document.createElement("i");
          const last = document.createElement("u");
          removed.appendChild(nested);
          root.append(first, removed, last);
          const iterator = document.createNodeIterator(root);
          iterator.nextNode();
          iterator.nextNode();
          iterator.nextNode();
          root.removeChild(removed);
          const iteratorAdjusted =
            iterator.referenceNode === first &&
            iterator.pointerBeforeReferenceNode === false &&
            iterator.nextNode() === last;

          const observer = new MutationObserver(() => {});
          observer.observe(element, {
            attributes: true,
            attributeOldValue: true
          });
          element.setAttribute("data-state", "ready");
          const records = observer.takeRecords();
          const mutation =
            records.length === 1 &&
            records[0].type === "attributes" &&
            records[0].target === element &&
            records[0].attributeName === "data-state" &&
            records[0].oldValue === null;

          const implementation = document.implementation;
          const doctype = implementation.createDocumentType(
            "x:root", "public", "system"
          );
          const xml = implementation.createDocument(
            "urn:test", "x:root", doctype
          );
          const implementationShape =
            xml.doctype === doctype &&
            xml.documentElement.namespaceURI === "urn:test" &&
            xml.documentElement.prefix === "x" &&
            doctype.ownerDocument === xml;
          let invalidDoctype = "";
          try { implementation.createDocumentType("bad name", "", ""); }
          catch (error) { invalidDoctype = error.name; }

          const parser = new DOMParser();
          const parsed = parser.parseFromString(
            "<root><item/></root>", "application/xml"
          );
          let parserBrand = "";
          try {
            DOMParser.prototype.parseFromString.call(
              {}, "<root/>", "application/xml"
            );
          } catch (error) { parserBrand = error.name; }

          const pi = document.createProcessingInstruction(
            "xml-stylesheet", 'href="a.css" media="screen"'
          );
          const piShape =
             pi.target === "xml-stylesheet" &&
             pi.sheet === null &&
             typeof pi.getAttribute === "function" &&
             pi.getAttribute("href") === null &&
             pi.hasAttributes() === false &&
             Object.getPrototypeOf(pi) === ProcessingInstruction.prototype &&
            Object.getPrototypeOf(ProcessingInstruction.prototype) ===
              CharacterData.prototype;

          const evaluator = new XPathEvaluator();
          const xpath = evaluator.evaluate(
            "count(.//item)", parsed, null, XPathResult.NUMBER_TYPE
          );
          let evaluatorBrand = "";
          try {
            XPathEvaluator.prototype.evaluate.call(
              {}, ".", parsed, null, XPathResult.ANY_TYPE
            );
          } catch (error) { evaluatorBrand = error.name; }

          return [
            setSync,
            removeSync,
            focusSync,
            blurSync,
            iteratorAdjusted,
            mutation,
            implementationShape,
            invalidDoctype,
            parsed.documentElement.nodeName,
            parserBrand,
            piShape,
            xpath.numberValue,
            evaluatorBrand
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|true|true|true|true|true|InvalidCharacterError|root|TypeError|true|1|TypeError"
    );
}

#[test]
fn selection_slots_static_ranges_and_cookie_store_share_dom_state() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const text = document.createTextNode("alpha beta");
          document.body.appendChild(text);
          const selection = document.getSelection();
          const sameSelection =
            selection === window.getSelection() &&
            selection === document.getSelection();
          selection.collapse(text, 5);
          const collapsedRange =
            selection.rangeCount === 1 &&
            selection.isCollapsed &&
            selection.getRangeAt(0).startContainer === text;
          selection.extend(text, 0);
          const backward =
            selection.direction === "backward" &&
            selection.anchorOffset === 5 &&
            selection.focusOffset === 0 &&
            selection.getRangeAt(0).startOffset === 0 &&
            selection.getRangeAt(0).endOffset === 5;
          const contains =
            selection.containsNode(text, true) &&
            !selection.containsNode(document.body, false);
          const composed = selection.getComposedRanges();
          const staticShape =
            composed.length === 1 &&
            composed[0] instanceof StaticRange &&
            !(composed[0] instanceof Range);
          selection.modify("move", "forward", "word");
          const modified =
            selection.isCollapsed &&
            selection.anchorOffset === 6 &&
            selection.focusOffset === 6;
          selection.removeRange(selection.getRangeAt(0));
          let missingRange = "";
          try { selection.removeRange(document.createRange()); }
          catch (error) { missingRange = error.name; }
          let emptyCollapse = "";
          try { selection.collapseToStart(); }
          catch (error) { emptyCollapse = error.name; }

          const host = document.createElement("x-slots");
          const light = document.createElement("span");
          light.slot = "named";
          host.appendChild(light);
          document.body.appendChild(host);
          const shadow = host.attachShadow({ mode: "open" });
          const slot = document.createElement("slot");
          slot.name = "named";
          const fallback = document.createTextNode("fallback");
          slot.appendChild(fallback);
          let slotChanges = 0;
          slot.addEventListener("slotchange", () => slotChanges++);
          shadow.appendChild(slot);
          light.setAttribute("slot", "named");
          const automaticSlot =
            light.assignedSlot === slot &&
            slot.assignedNodes()[0] === light &&
            slot.assignedElements()[0] === light;
          light.removeAttribute("slot");
          const noAssignment =
            slot.assignedNodes().length === 0 &&
            slot.assignedNodes({ flatten: true })[0] === fallback;

          let cookieEvent = null;
          cookieStore.addEventListener("change", event => cookieEvent = event);
          document.cookie = "first=one; Path=/";
          const documentToStoreEvent =
            cookieEvent instanceof CookieChangeEvent &&
            cookieEvent.changed[0].name === "first" &&
            cookieEvent.changed[0].value === "one";
          cookieStore.set("second", "two");
          const storeToDocument =
            document.cookie.includes("first=one") &&
            document.cookie.includes("second=two");
          cookieStore.delete("first");
          const deletedFromDocument =
            !document.cookie.includes("first=one") &&
            document.cookie.includes("second=two");

          return [
            sameSelection,
            collapsedRange,
            backward,
            contains,
            staticShape,
            modified,
            missingRange,
            emptyCollapse,
            automaticSlot,
            noAssignment,
            slotChanges > 0,
            documentToStoreEvent,
            storeToDocument,
            deletedFromDocument
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|true|true|true|true|NotFoundError|InvalidStateError|true|true|true|true|true|true"
    );
}

#[test]
fn html_text_reflection_traversal_reentrancy_math_focus_and_cookie_expiry_are_live() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const host = document.createElement("div");
          host.innerText = "alpha\r\nbeta";
          const innerTextTree =
            host.childNodes.length === 3 &&
            host.firstChild.data === "alpha" &&
            host.childNodes[1] instanceof HTMLBRElement &&
            host.lastChild.data === "beta" &&
            host.innerText === "alpha\nbeta";

          const parent = document.createElement("div");
          const replaced = document.createElement("span");
          replaced.textContent = "old";
          parent.appendChild(replaced);
          replaced.outerText = "left\nright";
          const outerTextTree =
            parent.childNodes.length === 3 &&
            parent.firstChild.data === "left" &&
            parent.childNodes[1] instanceof HTMLBRElement &&
            parent.lastChild.data === "right" &&
            replaced.parentNode === null;

          host.hidden = "until-found";
          const hiddenUntilFound =
            host.hidden === "until-found" &&
            host.getAttribute("hidden") === "until-found";
          host.hidden = false;
          host.inert = true;
          host.autofocus = true;
          host.draggable = true;
          host.spellcheck = false;
          const reflection =
            !host.hasAttribute("hidden") &&
            host.hasAttribute("inert") &&
            host.hasAttribute("autofocus") &&
            host.getAttribute("draggable") === "true" &&
            host.getAttribute("spellcheck") === "false";
          const translatedParent = document.createElement("div");
          const translatedChild = document.createElement("span");
          translatedParent.translate = false;
          translatedParent.appendChild(translatedChild);
          const inheritedTranslate =
            translatedParent.getAttribute("translate") === "no" &&
            translatedChild.translate === false;

          const traversalRoot = document.createElement("div");
          traversalRoot.append(document.createElement("i"));
          let iterator;
          iterator = document.createNodeIterator(
            traversalRoot,
            NodeFilter.SHOW_ALL,
            { acceptNode() { iterator.nextNode(); return NodeFilter.FILTER_ACCEPT; } }
          );
          let iteratorReentry = "";
          try { iterator.nextNode(); }
          catch (error) { iteratorReentry = error.name; }
          let walker;
          walker = document.createTreeWalker(
            traversalRoot,
            NodeFilter.SHOW_ALL,
            { acceptNode() { walker.nextNode(); return NodeFilter.FILTER_ACCEPT; } }
          );
          let walkerReentry = "";
          try { walker.nextNode(); }
          catch (error) { walkerReentry = error.name; }

          const math = document.createElementNS(
            "http://www.w3.org/1998/Math/MathML", "math"
          );
          document.body.appendChild(math);
          math.focus();
          const mathFocus =
            document.activeElement === math &&
            Object.getPrototypeOf(math) === MathMLElement.prototype;
          math.blur();
          const mathBlur = document.activeElement === document.body;

          cookieStore.set({ name: "expires", value: "present", path: "/" });
          cookieStore.set({
            name: "expires",
            value: "gone",
            path: "/",
            expires: 0
          });
          const expiredCookie = !document.cookie.includes("expires=");

          return [
            innerTextTree,
            outerTextTree,
            hiddenUntilFound,
            reflection,
            inheritedTranslate,
            iteratorReentry,
            walkerReentry,
            mathFocus,
            mathBlur,
            expiredCookie
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        "true|true|true|true|true|InvalidStateError|InvalidStateError|true|true|true"
    );
}

#[test]
fn edge_layout_resize_and_intersection_observers_follow_https_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (async () => {
          const element = document.createElement("div");
          element.style.cssText = [
            "position: fixed",
            "left: 10px",
            "top: 20px",
            "width: 100px",
            "height: 50px",
            "padding: 5px",
            "border: 2px solid black",
            "box-sizing: content-box",
            "overflow: auto"
          ].join(";");
          document.body.appendChild(element);

          const rect = element.getBoundingClientRect();
          const rects = element.getClientRects();
          const geometry = [
            rect.x, rect.y, rect.width, rect.height,
            rect.top, rect.right, rect.bottom, rect.left,
            element.clientWidth, element.clientHeight,
            element.clientLeft, element.clientTop,
            element.offsetWidth, element.offsetHeight,
            element.offsetLeft, element.offsetTop,
            element.scrollWidth, element.scrollHeight,
            element.offsetParent,
            Object.prototype.toString.call(rects),
            rects.length,
            rects.item(0) === rects[0],
            rects[0].width,
            rects[0].height
          ].join(",");

          const detached = document.createElement("div");
          detached.style.cssText =
            "position:fixed;left:10px;top:20px;width:100px;height:50px;padding:5px;border:2px solid";
          const detachedRect = detached.getBoundingClientRect();
          const detachedGeometry = [
            detachedRect.x, detachedRect.y,
            detachedRect.width, detachedRect.height,
            detached.clientWidth, detached.offsetWidth,
            detached.getClientRects().length
          ].join(",");

          element.style.display = "none";
          const hiddenRect = element.getBoundingClientRect();
          const hiddenGeometry = [
            hiddenRect.x, hiddenRect.y,
            hiddenRect.width, hiddenRect.height,
            element.clientWidth, element.offsetWidth,
            element.getClientRects().length
          ].join(",");
          element.style.display = "block";

          const resize = [];
          const resizeObserver = new ResizeObserver((entries, observer) => {
            const entry = entries[0];
            resize.push([
              entries.length,
              observer === resizeObserver,
              entry.target === element,
              entry.contentRect.width,
              entry.contentRect.height,
              entry.contentBoxSize[0].inlineSize,
              entry.contentBoxSize[0].blockSize,
              entry.borderBoxSize[0].inlineSize,
              entry.borderBoxSize[0].blockSize,
              entry.devicePixelContentBoxSize[0].inlineSize,
              entry.devicePixelContentBoxSize[0].blockSize
            ].join(","));
          });
          resizeObserver.observe(element);

          const intersections = [];
          const intersectionObserver =
            new IntersectionObserver((entries, observer) => {
              const entry = entries[0];
              intersections.push([
                entries.length,
                observer === intersectionObserver,
                entry.target === element,
                entry.isIntersecting,
                entry.intersectionRatio,
                entry.boundingClientRect.x,
                entry.boundingClientRect.y,
                entry.boundingClientRect.width,
                entry.boundingClientRect.height,
                entry.rootBounds.x,
                entry.rootBounds.y,
                entry.rootBounds.width,
                entry.rootBounds.height,
                entry.intersectionRect.x,
                entry.intersectionRect.y,
                entry.intersectionRect.width,
                entry.intersectionRect.height,
                entry.isVisible,
                Number.isFinite(entry.time) && entry.time >= 0
              ].join(","));
            });
          intersectionObserver.observe(element);

          await new Promise(resolve =>
            requestAnimationFrame(() => requestAnimationFrame(resolve))
          );
          const resizeInitial = resize.join("|");
          const intersectionInitial = intersections.join("|");
          resize.length = 0;
          intersections.length = 0;
          element.style.width = "120px";
          element.style.top = "10000px";
          await new Promise(resolve =>
            requestAnimationFrame(() => requestAnimationFrame(resolve))
          );

          const resizeChanged = resize.join("|");
          const intersectionChanged = intersections.join("|");
          resizeObserver.disconnect();
          intersectionObserver.disconnect();
          element.remove();
          return [
            geometry,
            detachedGeometry,
            hiddenGeometry,
            resizeInitial,
            resizeChanged,
            intersectionInitial,
            intersectionChanged
          ].join("||");
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "10,20,114,64,20,124,84,10,110,60,2,2,114,64,10,20,110,60,,",
            "[object DOMRectList],1,true,114,64||",
            "0,0,0,0,0,0,0||",
            "0,0,0,0,0,0,0||",
            "1,true,true,100,50,100,50,114,64,100,50||",
            "1,true,true,120,50,120,50,134,64,120,50||",
            "1,true,true,true,1,10,20,114,64,0,0,1280,720,10,20,114,64,false,true||",
            "1,true,true,false,0,10,10000,134,64,0,0,1280,720,0,0,0,0,false,true"
        )
    );
}

#[test]
fn input_intrinsic_geometry_follows_edge_150_https_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const types = [
            "text", "search", "password", "email", "url", "tel", "number",
            "date", "time", "datetime-local", "month", "week", "color",
            "checkbox", "radio", "range", "button", "submit", "reset",
            "file", "image", "hidden"
          ];
          const rows = types.map(type => {
            const input = document.createElement("input");
            input.type = type;
            document.body.append(input);
            const rect = input.getBoundingClientRect();
            const style = getComputedStyle(input);
            const row = [
              type, rect.width, rect.height,
              input.clientWidth, input.clientHeight,
              input.offsetWidth, input.offsetHeight,
              style.width, style.height, style.boxSizing, style.display
            ].join(",");
            input.remove();
            return row;
          });
          const detached = document.createElement("input");
          const rect = detached.getBoundingClientRect();
          rows.push([
            "detached", rect.x, rect.y, rect.width, rect.height,
            detached.clientWidth, detached.offsetWidth,
            detached.getClientRects().length
          ].join(","));
          return rows.join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "text,177,21,173,17,177,21,169px,15px,content-box,inline-block|",
            "search,177,21,173,17,177,21,177px,21px,border-box,inline-block|",
            "password,177,21,173,17,177,21,169px,15px,content-box,inline-block|",
            "email,177,21,173,17,177,21,169px,15px,content-box,inline-block|",
            "url,177,21,173,17,177,21,169px,15px,content-box,inline-block|",
            "tel,177,21,173,17,177,21,169px,15px,content-box,inline-block|",
            "number,177,21,173,17,177,21,169px,15px,content-box,inline-block|",
            "date,118.328125,23,114,19,118,23,113.328125px,19px,content-box,inline-block|",
            "time,75,24,71,20,75,24,70px,20px,content-box,inline-block|",
            "datetime-local,164.328125,23,160,19,164,23,159.328125px,19px,content-box,inline-block|",
            "month,112.328125,23,108,19,112,23,107.328125px,19px,content-box,inline-block|",
            "week,140.328125,23,136,19,140,23,135.328125px,19px,content-box,inline-block|",
            "color,50,27,48,25,50,27,50px,27px,border-box,inline-block|",
            "checkbox,13,13,13,13,13,13,13px,13px,border-box,inline-block|",
            "radio,13,13,13,13,13,13,13px,13px,border-box,inline-block|",
            "range,129,16,129,16,129,16,129px,16px,content-box,inline-block|",
            "button,16,21,12,17,16,21,16px,21px,border-box,inline-block|",
            "submit,42.671875,23,39,19,43,23,42.671875px,23px,border-box,inline-block|",
            "reset,42.671875,23,39,19,43,23,42.671875px,23px,border-box,inline-block|",
            "file,253,23,253,23,253,23,253px,23px,content-box,inline-block|",
            "image,0,0,0,0,0,0,0,0,content-box,inline-block|",
            "hidden,0,0,0,0,0,0,auto,auto,,none|",
            "detached,0,0,0,0,0,0,0"
        )
    );
}

#[test]
fn range_geometry_and_overflow_scrolling_follow_edge_https_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const rangeRoot = document.createElement("div");
          rangeRoot.style.cssText =
            "position:fixed;left:200px;top:100px;width:200px;height:100px";
          const first = document.createElement("span");
          first.style.cssText =
            "position:absolute;left:0;top:0;width:30px;height:10px;display:block";
          const second = document.createElement("span");
          second.style.cssText =
            "position:absolute;left:50px;top:20px;width:20px;height:15px;display:block";
          rangeRoot.append(first, second);
          document.body.appendChild(rangeRoot);
          const range = document.createRange();
          range.setStartBefore(first);
          range.setEndAfter(second);
          const bounds = range.getBoundingClientRect();
          const rects = range.getClientRects();
          const rangeGeometry = [
            Object.prototype.toString.call(bounds),
            bounds instanceof DOMRect,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            bounds.top,
            bounds.right,
            bounds.bottom,
            bounds.left,
            Object.prototype.toString.call(rects),
            rects instanceof DOMRectList,
            rects.length,
            rects.item(0) === rects[0],
            Array.from(rects, rect =>
              [rect.x, rect.y, rect.width, rect.height].join(",")
            ).join("/")
          ].join("|");
          range.collapse(true);
          const collapsed = range.getBoundingClientRect();
          const collapsedGeometry = [
            collapsed.x,
            collapsed.y,
            collapsed.width,
            collapsed.height,
            range.getClientRects().length
          ].join("|");

          const scrolling = document.createElement("div");
          scrolling.style.cssText = [
            "position:fixed",
            "left:300px",
            "top:250px",
            "width:100px",
            "height:50px",
            "padding:5px",
            "border:2px solid",
            "overflow:auto"
          ].join(";");
          const overflowChild = document.createElement("div");
          overflowChild.style.cssText =
            "position:absolute;left:150px;top:80px;width:20px;height:10px";
          scrolling.appendChild(overflowChild);
          document.body.appendChild(scrolling);
          const initialScroll = [
            scrolling.clientWidth,
            scrolling.clientHeight,
            scrolling.scrollWidth,
            scrolling.scrollHeight,
            scrolling.scrollLeft,
            scrolling.scrollTop
          ].join("|");
          scrolling.scrollLeft = 1000;
          scrolling.scrollTop = 1000;
          const childRect = overflowChild.getBoundingClientRect();
          const clampedScroll = [
            scrolling.scrollLeft,
            scrolling.scrollTop,
            childRect.x,
            childRect.y
          ].join("|");

          const image = document.createElement("img");
          image.style.cssText =
            "position:fixed;left:400px;top:300px;width:10px;height:20px";
          document.body.appendChild(image);
          const imageRect = image.getBoundingClientRect();
          const imageGeometry = [
            image.x,
            image.y,
            image.width,
            image.height,
            image.naturalWidth,
            image.naturalHeight,
            imageRect.x,
            imageRect.y,
            imageRect.width,
            imageRect.height
          ].join("|");
          rangeRoot.remove();
          scrolling.remove();
          image.remove();
          return [
            rangeGeometry,
            collapsedGeometry,
            initialScroll,
            clampedScroll,
            imageGeometry
          ].join("||");
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "[object DOMRect]|true|200|100|70|35|100|270|135|200|",
            "[object DOMRectList]|true|2|true|200,100,30,10/250,120,20,15||",
            "0|0|0|0|0||",
            "95|45|170|90|0|0||",
            "75|45|377|287||",
            "400|300|10|20|0|0|400|300|10|20"
        )
    );
}

#[test]
fn document_and_shadow_hit_testing_follow_edge_https_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const hitRoot = document.createElement("div");
          hitRoot.id = "hit-root";
          hitRoot.style.cssText = [
            "position:fixed",
            "left:600px",
            "top:100px",
            "width:200px",
            "height:150px",
            "z-index:2147483000"
          ].join(";");
          const hitLow = document.createElement("div");
          hitLow.id = "hit-low";
          hitLow.style.cssText =
            "position:absolute;left:10px;top:10px;width:100px;height:100px;z-index:1";
          const hitHigh = document.createElement("div");
          hitHigh.id = "hit-high";
          hitHigh.style.cssText =
            "position:absolute;left:30px;top:30px;width:100px;height:100px;z-index:2";
          const hitIgnored = document.createElement("div");
          hitIgnored.id = "hit-ignored";
          hitIgnored.style.cssText =
            "position:absolute;left:40px;top:40px;width:100px;height:100px;z-index:3;pointer-events:none";
          hitRoot.append(hitLow, hitHigh, hitIgnored);
          document.body.appendChild(hitRoot);
          const elements = document.elementsFromPoint(650, 150);
          const stacking = [
            elements.slice(0, 3).map(element =>
              element.id || element.tagName
            ).join(","),
            document.elementFromPoint(650, 150) === hitHigh,
            elements.includes(hitIgnored),
            Object.prototype.toString.call(elements),
            Array.isArray(elements)
          ].join("|");

          hitHigh.style.visibility = "hidden";
          hitLow.style.opacity = "0";
          const bounds = [
            document.elementFromPoint(650, 150) === hitLow,
            document.elementFromPoint(605, 105) === hitRoot,
            document.elementFromPoint(800, 150) === hitRoot,
            document.elementFromPoint(-1, 0) === null,
            document.elementFromPoint(0, -1) === null,
            document.elementFromPoint(innerWidth, 0) === null,
            document.elementFromPoint(0, innerHeight) === null
          ].join("|");
          hitRoot.remove();

          const host = document.createElement("div");
          host.id = "hit-host";
          host.style.cssText = [
            "position:fixed",
            "left:600px",
            "top:300px",
            "width:100px",
            "height:80px",
            "z-index:2147483000"
          ].join(";");
          const shadow = host.attachShadow({ mode: "open" });
          const shadowLow = document.createElement("div");
          shadowLow.id = "shadow-low";
          shadowLow.style.cssText =
            "position:absolute;left:0;top:0;width:80px;height:60px;z-index:1";
          const shadowHigh = document.createElement("div");
          shadowHigh.id = "shadow-high";
          shadowHigh.style.cssText =
            "position:absolute;left:10px;top:10px;width:50px;height:40px;z-index:2";
          shadow.append(shadowLow, shadowHigh);
          document.body.appendChild(host);
          const documentHits = document.elementsFromPoint(620, 320);
          const shadowHits = shadow.elementsFromPoint(620, 320);
          const shadowResult = [
            document.elementFromPoint(620, 320) === host,
            documentHits[0] === host,
            shadow.elementFromPoint(620, 320) === shadowHigh,
            shadowHits.slice(0, 3).map(element =>
              element.id || element.tagName
            ).join(","),
            Object.prototype.toString.call(shadowHits),
            Array.isArray(shadowHits)
          ].join("|");
          host.remove();
          return [stacking, bounds, shadowResult].join("||");
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "hit-high,hit-low,hit-root|true|false|[object Array]|true||",
            "true|true|false|true|true|true|true||",
            "true|true|true|shadow-high,shadow-low,hit-host|[object Array]|true"
        )
    );
}

#[test]
fn scroll_into_view_alignment_follows_edge_https_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const container = document.createElement("div");
          container.style.cssText = [
            "position:fixed",
            "left:900px",
            "top:100px",
            "width:100px",
            "height:50px",
            "overflow:auto"
          ].join(";");
          const target = document.createElement("div");
          target.style.cssText =
            "position:absolute;left:150px;top:80px;width:20px;height:10px";
          const extent = document.createElement("div");
          extent.style.cssText =
            "position:absolute;left:280px;top:180px;width:20px;height:20px";
          container.append(target, extent);
          document.body.appendChild(container);
          const run = operation => {
            container.scrollTo(0, 0);
            operation();
            const rect = target.getBoundingClientRect();
            return [
              container.scrollLeft,
              container.scrollTop,
              rect.x,
              rect.y
            ].join(",");
          };
          const result = [
            container.clientWidth,
            container.clientHeight,
            container.scrollWidth,
            container.scrollHeight,
            run(() => target.scrollIntoView()),
            run(() => target.scrollIntoView(false)),
            run(() => target.scrollIntoView({
              block: "center",
              inline: "center"
            })),
            run(() => target.scrollIntoView({
              block: "nearest",
              inline: "nearest"
            })),
            run(() => target.scrollIntoViewIfNeeded()),
            run(() => target.scrollIntoViewIfNeeded(false))
          ].join("|");
          container.remove();
          return result;
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "85|35|300|200|",
            "85,80,965,100|",
            "85,55,965,125|",
            "118,68,932,112|",
            "85,55,965,125|",
            "118,68,932,112|",
            "85,55,965,125"
        )
    );
}

#[test]
fn standard_html_fragment_tree_builder_matches_edge_context_rules() {
    let source = r##"
      (() => {
        const values = [];

        const adoption = document.createElement("div");
        adoption.innerHTML = "<b><i>x</b>y</i>";
        values.push(adoption.innerHTML);

        const table = document.createElement("table");
        table.innerHTML = "<tr><td>A<td>B";
        values.push(table.innerHTML, table.tBodies.length, table.rows.length);

        const select = document.createElement("select");
        select.innerHTML = "<div>x</div><option>one<option>two";
        values.push(select.innerHTML, select.options.length);

        const textarea = document.createElement("textarea");
        textarea.innerHTML = "a&amp;<b>x</b>";
        values.push(textarea.value, textarea.innerHTML);

        const template = document.createElement("template");
        const otherTemplate = document.createElement("template");
        template.innerHTML =
          "<p>&NotEqualTilde;</p><template><i>x</i></template>";
        const nestedTemplate = template.content.lastChild;
        values.push(
          template.childNodes.length,
          template.content.childNodes.length,
          template.innerHTML,
          template.content.ownerDocument === otherTemplate.content.ownerDocument,
          template.content.ownerDocument.defaultView === null,
          template.content.firstChild.ownerDocument ===
            template.content.ownerDocument,
          nestedTemplate.content.ownerDocument === template.content.ownerDocument
        );

        const tbody = document.createElement("tbody");
        const range = document.createRange();
        range.selectNodeContents(tbody);
        const fragment = range.createContextualFragment("<tr><td>R");
        values.push(
          fragment.firstChild.tagName,
          fragment.firstChild.firstChild.tagName,
          fragment.textContent
        );

        const adjacentTable = document.createElement("table");
        adjacentTable.innerHTML =
          "<tbody><tr id=a><td>A</td></tr></tbody>";
        const adjacentBody = adjacentTable.tBodies[0];
        adjacentBody.insertAdjacentHTML("beforeend", "<tr id=b><td>B");
        values.push(
          adjacentBody.rows.length,
          adjacentBody.lastElementChild.id,
          adjacentBody.lastElementChild.cells[0].textContent
        );

        const parsed = new DOMParser().parseFromString(
          "<template id=t><span>P</span></template>",
          "text/html"
        );
        const parsedTemplate = parsed.querySelector("#t");
        values.push(
          parsed.documentElement.tagName,
          parsed.body.tagName,
          parsedTemplate.childNodes.length,
          parsedTemplate.content.firstChild.tagName,
          parsedTemplate.content.textContent,
          parsedTemplate.content.ownerDocument === parsed
        );
        return values.join("|");
      })()
    "##;
    let expected = concat!(
        "<b><i>x</i></b><i>y</i>|",
        "<tbody><tr><td>A</td><td>B</td></tr></tbody>|1|1|",
        "<div>x</div><option>one</option><option>two</option>|2|",
        "a&<b>x</b>|a&amp;&lt;b&gt;x&lt;/b&gt;|",
        "0|2|<p>\u{2242}\u{338}</p><template><i>x</i></template>|",
        "true|true|true|true|TR|TD|R|",
        "2|b|B|HTML|BODY|0|SPAN|P|false"
    );

    let mut direct = EdgeRuntime::new().expect("direct fragment runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::new().expect("traced fragment runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn template_contents_clone_import_and_adoption_match_edge_documents() {
    let source = r#"
      (() => {
        const template = document.createElement("template");
        template.innerHTML =
          "<b>x</b><template><i>y</i></template>";
        const deep = template.cloneNode(true);
        const shallow = template.cloneNode(false);
        const other = document.implementation.createHTMLDocument("");
        const imported = other.importNode(template, true);
        const otherTemplate = other.createElement("template");
        const sourceOwner = template.content.ownerDocument;
        other.adoptNode(template);
        return [
          deep.innerHTML,
          shallow.innerHTML,
          deep.isEqualNode(template),
          deep.content.ownerDocument === sourceOwner,
          imported.innerHTML,
          imported.content.ownerDocument ===
            otherTemplate.content.ownerDocument,
          imported.content.ownerDocument === sourceOwner,
          template.content.ownerDocument ===
            otherTemplate.content.ownerDocument,
          template.content.firstChild.ownerDocument ===
            template.content.ownerDocument
        ].join("|");
      })()
    "#;
    let expected = concat!(
        "<b>x</b><template><i>y</i></template>||true|true|",
        "<b>x</b><template><i>y</i></template>|true|false|true|true"
    );

    let mut direct = EdgeRuntime::new().expect("direct template runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::new().expect("traced template runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn css_math_calculation_serialization_supports_and_layout_match_edge_150() {
    let source = r#"
      (() => {
        const element = document.createElement("div");
        document.body.appendChild(element);
        element.style.width = 'calc( 1px * ( ( 2.71828 * 0.987654321 - 0.123456789 / ( 987654.321 * 12345.678 ) ) + 0.5555555 / sin( sin( 10000.1 * tan( 50000 ) / tan( 20000 ) + 1.0 / pi * 5.0 - 0.1111 ) / 100.0 + tan( 30000 + 40000 * 50000 + 0.0001 ) / 9999.9 * pi ) - 0.999999 * -100000.5 ) )';
        const specified = element.style.width;
        const computed = getComputedStyle(element).width;
        const rectIsCalculated = element.getBoundingClientRect().width > 100074;
        element.style.width = 'calc(1dppx + 96dpi)';
        const invalidPreservesPrevious = element.style.width === specified;
        const cases = [
          'calc(1px + 2px)',
          'min(10px, 20px)',
          'max(10px, 20px)',
          'clamp(1px, 3px, 2px)',
          'calc(sin(pi / 2) * 1px)',
          'calc(round(up, 2.1, 1) * 1px)',
          'calc(mod(7, 4) * 1px)',
          'calc(pow(2, 3) * 1px)',
          'calc(hypot(3, 4) * 1px)',
          'calc(log(exp(2)) * 1px)'
        ].map(value => {
          element.style.width = value;
          return element.style.width;
        });
        element.style.width = 'calc(1q + 1cm)';
        const fractional = [
          element.style.width,
          getComputedStyle(element).width,
          element.getBoundingClientRect().width
        ];
        element.style.width = 'calc(1px + infinity * 1px)';
        const infinity = [element.style.width, getComputedStyle(element).width];
        element.style.width = 'calc(NaN * 1px)';
        const nan = [element.style.width, getComputedStyle(element).width];
        element.style.width = 'calc(progress(10px, 0px, 20px) * 100px)';
        const progress = [element.style.width, getComputedStyle(element).width];
        const intrinsic = document.body.getBoundingClientRect().width;
        element.style.width = 'calc-size(auto, size + 10px)';
        const calcSize = [
          element.style.width,
          parseFloat(getComputedStyle(element).width) === intrinsic + 10,
          element.getBoundingClientRect().width === intrinsic + 10
        ];
        return [
          specified,
          computed,
          rectIsCalculated,
          invalidPreservesPrevious,
          CSS.supports('width', 'calc(1px + 2px)'),
          CSS.supports('width', 'calc(1dppx + 96dpi)'),
          ...cases,
          ...fractional,
          ...infinity,
          ...nan,
          ...progress,
          ...calcSize
        ].join('|');
      })()
    "#;
    let expected = concat!(
        "calc(100075px)|100075px|true|true|true|false|",
        "calc(3px)|calc(10px)|calc(20px)|calc(2px)|calc(1px)|",
        "calc(3px)|calc(3px)|calc(8px)|calc(5px)|calc(2px)|",
        "calc(38.7402px)|38.7344px|38.734375|",
        "calc(infinity * 1px)|3.35544e+07px|calc(NaN * 1px)|0px|",
        "calc(50px)|50px|calc-size(auto, 10px + size)|true|true"
    );

    let mut direct = EdgeRuntime::new().expect("direct CSS math runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::new().expect("traced CSS math runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn css_math_relative_font_units_follow_cascade_inheritance_and_root_font() {
    let source = r#"
      (() => {
        document.documentElement.style.fontSize = '20px';
        const parent = document.createElement('div');
        parent.style.fontSize = '150%';
        const child = document.createElement('div');
        child.style.fontSize = '2em';
        child.style.width = 'calc(1em + 1rem)';
        child.style.lineHeight = 'calc(1.2)';
        child.style.opacity = 'calc(50%)';
        child.style.scale = 'calc(50%)';
        child.style.rotate = 'calc(.5turn / 2)';
        child.style.animationDuration = 'calc(1s + 500ms)';
        parent.appendChild(child);
        document.body.appendChild(parent);
        return [
          getComputedStyle(document.documentElement).fontSize,
          getComputedStyle(parent).fontSize,
          getComputedStyle(child).fontSize,
          child.style.width,
          getComputedStyle(child).width,
          child.getBoundingClientRect().width,
          child.style.lineHeight,
          getComputedStyle(child).lineHeight,
          getComputedStyle(child).opacity,
          getComputedStyle(child).scale,
          child.style.rotate,
          getComputedStyle(child).rotate,
          child.style.animationDuration,
          getComputedStyle(child).animationDuration
        ].join('|');
      })()
    "#;
    let expected = concat!(
        "20px|30px|60px|calc(1em + 1rem)|80px|80|",
        "calc(1.2)|72px|0.5|0.5|calc(90deg)|90deg|calc(1.5s)|1.5s"
    );

    let mut direct = EdgeRuntime::new().expect("direct CSS relative-unit runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::new().expect("traced CSS relative-unit runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn css_numeric_typed_om_parses_and_simplifies_math_values() {
    let source = r#"
      (() => [...['1in', 'calc(1px)', 'calc(1em * 2)',
        'calc(1px + 2px)', 'min(1cm, 20px)', 'calc(50% + 1px)']
        .map(text => {
          const value = CSSNumericValue.parse(text);
          return `${value.constructor.name},${String(value)}`;
        }), (() => {
          const value = CSSStyleValue.parse('width', 'calc(1px + 2px)');
          return `${value.constructor.name},${String(value)}`;
        })()].join('|'))()
    "#;
    let expected = concat!(
        "CSSUnitValue,1in|CSSMathSum,calc(1px)|CSSMathSum,calc(2em)|",
        "CSSMathSum,calc(3px)|CSSUnitValue,20px|CSSMathSum,calc(50% + 1px)|",
        "CSSMathSum,calc(3px)"
    );

    let mut direct = EdgeRuntime::new().expect("direct CSS Typed OM runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::new().expect("traced CSS Typed OM runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(text(&mut traced, source), expected);
}
