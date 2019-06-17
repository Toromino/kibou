use regex::Regex;

pub fn strip_tags(input: &str) -> String {
    let allowed_tags = vec!["a", "b", "br", "em", "img", "strong", "u"];
    let forbidden_attributes = vec![
        "onabort",
        "onafterprint",
        "onbeforeprint",
        "onbeforeunload",
        "onblur",
        "oncanplay",
        "oncanplaythrough",
        "onchange",
        "onclick",
        "oncontextmenu",
        "oncopy",
        "oncuechange",
        "oncut",
        "ondblclick",
        "ondrag",
        "ondragend",
        "ondragenter",
        "ondragleave",
        "ondragover",
        "ondragstart",
        "ondrop",
        "ondurationchange",
        "onemptied",
        "onended",
        "onerror",
        "onfocus",
        "onhashchange",
        "oninput",
        "oninvalid",
        "onkeydown",
        "onkeypress",
        "onkeyup",
        "onload",
        "onloadeddata",
        "onloadedmetadata",
        "onloadstart",
        "onmessage",
        "onmousedown",
        "onmousemove",
        "onmouseout",
        "onmouseover",
        "onmouseup",
        "onmousewheel",
        "onoffline",
        "ononline",
        "onpagehide",
        "onpageshow",
        "onpaste",
        "onpause",
        "onplay",
        "onplaying",
        "onpopstate",
        "onprogress",
        "onratechange",
        "onreset",
        "onresize",
        "onscroll",
        "onsearch",
        "onseeked",
        "onseeking",
        "onselect",
        "onstalled",
        "onstorage",
        "onsubmit",
        "onsuspend",
        "ontimeupdate",
        "ontoggle",
        "onunload",
        "onvolumechange",
        "onwaiting",
        "onwheel",
    ];
    let mut output: String = input.to_string();
    let tag_regex: Regex = Regex::new("<[^>]*>").unwrap();

    for tag in tag_regex.captures_iter(&input) {
        let parsed_tag: Vec<&str> = tag
            .get(0)
            .unwrap()
            .as_str()
            .split(&[' ', '<', '>'][..])
            .collect();
        let mut tag_valid: bool = true;

        let mut parsed_start_tag = String::new();
        let stripped_characters = "/";
        for character in parsed_tag[1].chars() {
            if !stripped_characters.contains(character) {
                parsed_start_tag.push(character);
            }
        }

        if allowed_tags.contains(&parsed_start_tag.as_str()) {
            // The html sanitizer should try to strip malicious attributes from a tag rather than
            // just stripping a whole tag
            for tag_slice in parsed_tag.iter() {
                for attribute in forbidden_attributes.iter() {
                    if tag_slice.contains(attribute) {
                        output = str::replace(&output, tag_slice, "");
                    }
                }
            }
        } else {
            tag_valid = false;
        }

        if !tag_valid {
            output = str::replace(&output, tag.get(0).unwrap().as_str(), "");
        }
    }
    return output;
}
