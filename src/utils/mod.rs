pub(crate) use chars::*;
// pub(crate) use comment_whitespace::*;
// pub(crate) use number::*;
// pub(crate) use read_until::*;
pub(crate) use strings::*;

mod chars;
// mod comment_whitespace;
// mod number;
// mod read_until;
mod strings;

#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub(crate) fn is_plain_css_import(url: &str) -> bool {
    if url.len() < 5 {
        return false;
    }

    let lower = url.to_ascii_lowercase();

    lower.ends_with(".css")
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("//")
}

pub(crate) fn opposite_bracket(b: char) -> char {
    debug_assert!(matches!(b, '(' | '{' | '[' | ')' | '}' | ']'));
    match b {
        '(' => ')',
        '{' => '}',
        '[' => ']',
        ')' => '(',
        '}' => '{',
        ']' => '[',
        _ => unreachable!(),
    }
}

pub(crate) fn is_special_function(s: &str) -> bool {
    s.starts_with("calc(")
        || s.starts_with("var(")
        || s.starts_with("env(")
        || s.starts_with("min(")
        || s.starts_with("max(")
        || s.starts_with("clamp(")
}

pub(crate) fn trim_ascii(
    s: &str,
    // default=false
    exclude_escape: bool,
) -> &str {
    match s.chars().position(|c| !c.is_ascii_whitespace()) {
        Some(start) => &s[start..last_non_whitespace(s, exclude_escape).unwrap() + 1],
        None => "",
    }
}

fn last_non_whitespace(s: &str, exclude_escape: bool) -> Option<usize> {
    let mut idx = s.len() - 1;
    for c in s.chars().rev() {
        if !c.is_ascii_whitespace() {
            if exclude_escape && idx != 0 && idx != s.len() && c == '\\' {
                return Some(idx + 1);
            } else {
                return Some(idx);
            }
        }

        idx -= 1;
    }

    None
}
