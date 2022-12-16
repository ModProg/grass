#[macro_use]
mod macros;

test!(
    single_codepoint,
    "a {\n  color: U+26;\n}\n",
    "a {\n  color: U+26;\n}\n"
);
test!(
    simple_range,
    "a {\n  color: U+0-7F;\n}\n",
    "a {\n  color: U+0-7F;\n}\n"
);
test!(
    simple_wildcard_range,
    "a {\n  color: U+45????;\n}\n",
    "a {\n  color: U+45????;\n}\n"
);
test!(
    lowercase_u,
    "a {\n  color: u+27a;\n}\n",
    "a {\n  color: u+27a;\n}\n"
);
error!(
    interpolated_range,
    "a {\n  color: U+2A#{70}C;\n}\n", "Error: Expected end of identifier."
);
error!(
    unicode_escape_within_range,
    "a {\n  color: U+B\\a;\n}\n", "Error: Expected end of identifier."
);
error!(
    longer_than_6_characters,
    "a {\n  color: U+1234567;\n}\n", "Error: Expected end of identifier."
);
error!(
    length_of_6_with_question_mark,
    "a {\n  color: U+123456?;\n}\n", "Error: Expected at most 6 digits."
);

// todo: escaped u at start \75 and \55
// with and without space
