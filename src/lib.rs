use std::borrow::Cow;
use std::str;

/// GNAT generates some weird compressed names in DW_AT_name that we should probably ignore.
pub fn is_short_name(b: &[u8]) -> bool {
    if !b.starts_with(b"ada_main__") || b.len() < 12 {
        return false;
    }
    let ch = b[10];
    if ch < b'a' || ch > b'z' {
        return false;
    }
    for c in b[11..].iter() {
        if *c < b'0' || *c > b'9' {
            return false;
        }
    }
    return true;
}

fn get_prefix(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut index = 0;
    while index + 2 <= bytes.len() {
        if &bytes[index..(index + 2)] == b"__" {
            return Some((&bytes[..index], &bytes[(index + 2)..]));
        }
        index += 1;
    }
    None
}

fn get_suffix(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut index = bytes.len();
    while index >= 2 {
        if &bytes[(index - 2)..index] == b"__" {
            return Some((&bytes[..(index - 2)], &bytes[index..]));
        }
        index -= 1;
    }
    None
}

fn hex_to_u16(bytes: &[u8]) -> Option<u16> {
    let mut ret = 0;
    for b in bytes {
        match *b {
            b'0'..=b'9' => ret = (ret << 4) | (*b - b'0') as u16,
            b'a'..=b'f' => ret = (ret << 4) | (*b - b'a' + 10) as u16,
            _ => return None,
        }
    }
    Some(ret)
}

/// See https://cs.nyu.edu/courses/spring01/G22.2130-001/gnat-html-old/namet__ads.htm
fn bytes_to_string(mut bytes: &[u8]) -> Option<Cow<str>> {
    if bytes
        .iter()
        .all(|c| c.is_ascii() && *c != b'U' && *c != b'W')
    {
        return Some(Cow::Borrowed(str::from_utf8(bytes).unwrap()));
    }
    let mut buf: Vec<u16> = Vec::new();
    while !bytes.is_empty() {
        if bytes[0] == b'U' {
            if bytes.len() < 3 {
                return None;
            }
            buf.push(hex_to_u16(&bytes[1..3])?);
            bytes = &bytes[3..];
        } else if bytes[0] == b'W' {
            if bytes.len() < 5 {
                return None;
            }
            buf.push(hex_to_u16(&bytes[1..5])?);
            bytes = &bytes[5..];
        } else {
            buf.push(bytes[0] as u16);
            bytes = &bytes[1..];
        }
    }
    String::from_utf16(&buf).ok().map(|v| v.into())
}

fn parse_operator(mut bytes: &[u8]) -> Option<&'static str> {
    if bytes.first() != Some(&b'O') {
        return None;
    }
    bytes = &bytes[1..];
    Some(match bytes {
        b"abs" => "abs",
        b"and" => "and",
        b"mod" => "mod",
        b"not" => "not",
        b"or" => "or",
        b"rem" => "rem",
        b"xor" => "xor",
        b"eq" => "=",
        b"ne" => "/=",
        b"lt" => "<",
        b"le" => "<=",
        b"gt" => ">",
        b"ge" => ">=",
        b"add" => "+",
        b"subtract" => "-",
        b"concat" => "&",
        b"multiply" => "*",
        b"divide" => "/",
        b"expon" => "**",
        _ => return None,
    })
}

pub trait DemangleVisitor<'a> {
    fn enter_prefix(&mut self) {}
    fn enter_ident(&mut self) {}
    fn text(&mut self, text: Cow<'a, str>);
    fn exit(&mut self) {}
}

fn is_anonymous_block(bytes: &[u8]) -> bool {
    if bytes.len() < 3 || &bytes[0..2] != b"B_" {
        return false;
    }
    bytes[2..].iter().all(|c| *c >= b'0' && *c <= b'9')
}

/// Actually do the demangling. Returns None if it doesn't look like something
/// we can demangle.
/// See https://github.com/gcc-mirror/gcc/blob/master/gcc/ada/exp_dbug.ads#L36
pub fn demangle<'a, V: DemangleVisitor<'a>>(mut bytes: &'a [u8], visitor: &mut V) -> Option<()> {
    if bytes.starts_with(b"_ada_") {
        bytes = &bytes[5..];
    }
    if let Some((rest, suffix)) = get_suffix(bytes) {
        if let Some(c) = suffix.first() {
            if let b'0'..=b'9' = *c {
                bytes = rest;
            }
        }
    }
    // Iterate over components
    while let Some((mut prefix, rest)) = get_prefix(bytes) {
        if let Some(i) = prefix.iter().rposition(|c| *c == b'T') {
            prefix = &prefix[..i];
        }
        bytes = rest;
        if is_anonymous_block(prefix) {
            continue;
        }
        visitor.enter_prefix();
        visitor.enter_ident();
        visitor.text(bytes_to_string(prefix)?);
        visitor.exit();
        visitor.text(Cow::Borrowed("."));
        visitor.exit();
    }

    if let Some(i) = bytes
        .iter()
        .rposition(|c| matches!(*c, b'X' | b'N' | b'E' | b'B'))
    {
        bytes = &bytes[..i];
    }

    visitor.enter_ident();
    if let Some(o) = parse_operator(bytes) {
        visitor.text(Cow::Borrowed(o));
    } else {
        visitor.text(bytes_to_string(bytes)?);
    }
    visitor.exit();
    Some(())
}
