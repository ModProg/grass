use std::io::Write;

#[macro_use]
mod macros;

error!(
    after_style,
    "a {}
    @use \"foo\";
    ",
    "Error: @use rules must be written before any other rules."
);
error!(
    interpolation_in_as_identifier,
    "@use \"sass:math\" as m#{a}th;", "Error: expected \";\"."
);
error!(
    use_as_quoted_string,
    "@use \"sass:math\" as \"math\";", "Error: Expected identifier."
);
error!(
    use_as_missing_s,
    "@use \"sass:math\" a math;", "Error: expected \";\"."
);
error!(
    unknown_module_get_variable,
    "a { color: foo.$bar; }", "Error: There is no module with the namespace \"foo\"."
);
error!(
    unknown_module_get_function,
    "a { color: foo.bar(); }", "Error: There is no module with the namespace \"foo\"."
);
error!(
    unknown_function,
    "@use \"sass:math\";\na { color: math.bar(); }", "Error: Undefined function."
);
error!(
    module_function_missing_open_parens,
    "@use \"sass:math\";\na { color: math.floor; }", "Error: expected \"(\"."
);
error!(
    module_not_quoted_string,
    "@use a", "Error: Expected string."
);
error!(
    use_file_name_is_invalid_identifier,
    r#"@use "a b";"#, r#"Error: The default namespace "a b" is not a valid Sass identifier."#
);
error!(
    use_empty_string,
    r#"@use "";"#, r#"Error: The default namespace "" is not a valid Sass identifier."#
);
test!(
    use_as,
    "@use \"sass:math\" as foo;
    a {
        color: foo.clamp(0, 1, 2);
    }",
    "a {\n  color: 1;\n}\n"
);
test!(
    use_as_uppercase,
    "@use \"sass:math\" AS foo;
    a {
        color: foo.clamp(0, 1, 2);
    }",
    "a {\n  color: 1;\n}\n"
);
test!(
    use_as_universal,
    "@use \"sass:math\" as *;
    a {
        color: cos(2);
    }",
    "a {\n  color: -0.4161468365;\n}\n"
);
test!(
    use_single_quotes,
    "@use 'sass:math';
    a {
        color: math.cos(2);
    }",
    "a {\n  color: -0.4161468365;\n}\n"
);

#[test]
fn use_user_defined_same_directory() {
    let input = "@use \"use_user_defined_same_directory\";\na {\n color: use_user_defined_same_directory.$a;\n}";
    tempfile!(
        "use_user_defined_same_directory.scss",
        "$a: red; a { color: $a; }"
    );
    assert_eq!(
        "a {\n  color: red;\n}\n\na {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn private_variable_begins_with_underscore() {
    let input = "@use \"private_variable_begins_with_underscore\" as module;\na {\n color: module.$_foo;\n}";
    tempfile!(
        "private_variable_begins_with_underscore.scss",
        "$_foo: red; a { color: $_foo; }"
    );

    assert_err!(
        "Error: Private members can't be accessed from outside their modules.",
        input
    );
}

#[test]
fn private_variable_begins_with_hyphen() {
    let input =
        "@use \"private_variable_begins_with_hyphen\" as module;\na {\n color: module.$-foo;\n}";
    tempfile!(
        "private_variable_begins_with_hyphen.scss",
        "$-foo: red; a { color: $-foo; }"
    );

    assert_err!(
        "Error: Private members can't be accessed from outside their modules.",
        input
    );
}

#[test]
fn private_function() {
    let input = "@use \"private_function\" as module;\na {\n color: module._foo(green);\n}";
    tempfile!(
        "private_function.scss",
        "@function _foo($a) { @return $a; } a { color: _foo(red); }"
    );

    assert_err!(
        "Error: Private members can't be accessed from outside their modules.",
        input
    );
}

#[test]
fn global_variable_exists_private() {
    let input = r#"
        @use "global_variable_exists_private" as module;
        a {
            color: global-variable-exists($name: foo, $module: module);
            color: global-variable-exists($name: _foo, $module: module);
        }"#;
    tempfile!(
        "global_variable_exists_private.scss",
        "$foo: red;\n$_foo: red;\n"
    );

    assert_eq!(
        "a {\n  color: true;\n  color: false;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_user_defined_as() {
    let input = "@use \"use_user_defined_as\" as module;\na {\n color: module.$a;\n}";
    tempfile!("use_user_defined_as.scss", "$a: red; a { color: $a; }");
    assert_eq!(
        "a {\n  color: red;\n}\n\na {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_user_defined_function() {
    let input = "@use \"use_user_defined_function\" as module;\na {\n color: module.foo(red);\n}";
    tempfile!(
        "use_user_defined_function.scss",
        "@function foo($a) { @return $a; }"
    );
    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_idempotent_no_alias() {
    let input = "@use \"use_idempotent_no_alias\";\n@use \"use_idempotent_no_alias\";\n";
    tempfile!("use_idempotent_no_alias.scss", "");

    assert_err!(
        "Error: There's already a module with namespace \"use-idempotent-no-alias\".",
        input
    );
}

#[test]
fn use_idempotent_with_alias() {
    let input = "@use \"use_idempotent_with_alias__a\" as foo;\n@use \"use_idempotent_with_alias__b\" as foo;\n";
    tempfile!("use_idempotent_with_alias__a.scss", "");
    tempfile!("use_idempotent_with_alias__b.scss", "");

    assert_err!(
        "Error: There's already a module with namespace \"foo\".",
        input
    );
}

#[test]
fn use_idempotent_builtin() {
    let input = "@use \"sass:math\";\n@use \"sass:math\";\n";

    assert_err!(
        "Error: There's already a module with namespace \"math\".",
        input
    );
}

#[test]
fn use_with_simple() {
    let input = "@use \"use_with_simple\" with ($a: red);\na {\n color: use_with_simple.$a;\n}";
    tempfile!("use_with_simple.scss", "$a: green !default;");
    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_as_with() {
    let input = "@use \"use_as_with\" as module with ($a: red);\na {\n color: module.$a;\n}";
    tempfile!("use_as_with.scss", "$a: green !default;");
    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_whitespace_and_comments() {
    let input = "@use  /**/  \"use_whitespace_and_comments\"  /**/  as  /**/  foo  /**/  with  /**/  (  /**/  $a  /**/  :  /**/  red  /**/  );";
    tempfile!(
        "use_whitespace_and_comments.scss",
        "$a: green !default; a { color: $a }"
    );
    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_loud_comment_after_close_paren_with() {
    let input = r#"@use "b" as foo with ($a : red)  /**/  ;"#;
    tempfile!(
        "use_loud_comment_after_close_paren_with.scss",
        "$a: green !default; a { color: $a }"
    );
    assert_err!(r#"Error: expected ";"."#, input);
}

#[test]
fn use_with_builtin_module() {
    let input = "@use \"sass:math\" with ($e: 2.7);";

    assert_err!("Error: Built-in modules can't be configured.", input);
}

#[test]
fn use_with_variable_never_used() {
    let input = "@use \"use_with_variable_never_used\" with ($a: red);";
    tempfile!("use_with_variable_never_used.scss", "");

    assert_err!(
        "Error: This variable was not declared with !default in the @used module.",
        input
    );
}

#[test]
fn use_with_same_variable_multiple_times() {
    let input = "@use \"use_with_same_variable_multiple_times\" as foo with ($a: b, $a: c);";
    tempfile!("use_with_same_variable_multiple_times.scss", "");

    assert_err!(
        "Error: The same variable may only be configured once.",
        input
    );
}

#[test]
fn use_variable_redeclaration_var_dne() {
    let input = "@use \"use_variable_redeclaration_var_dne\" as mod;\nmod.$a: red;";
    tempfile!("use_variable_redeclaration_var_dne.scss", "");

    assert_err!("Error: Undefined variable.", input);
}

#[test]
fn use_variable_redeclaration_global() {
    let input = "@use \"use_variable_redeclaration_global\" as mod;\nmod.$a: red !global;";
    tempfile!("use_variable_redeclaration_global.scss", "$a: green;");

    assert_err!(
        "Error: !global isn't allowed for variables in other modules.",
        input
    );
}

#[test]
fn use_variable_redeclaration_simple() {
    let input =
        "@use \"use_variable_redeclaration_simple\" as mod;\nmod.$a: red; a { color: mod.$a; }";
    tempfile!("use_variable_redeclaration_simple.scss", "$a: green;");

    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_variable_redeclaration_default() {
    let input = "@use \"use_variable_redeclaration_default\" as mod;\nmod.$a: 1 % red !default; a { color: mod.$a; }";
    tempfile!("use_variable_redeclaration_default.scss", "$a: green;");

    assert_eq!(
        "a {\n  color: green;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_variable_redeclaration_private() {
    let input = "@use \"use_variable_redeclaration_private\" as mod;\nmod.$-a: red;";
    tempfile!("use_variable_redeclaration_private.scss", "$a: green;");

    assert_err!(
        "Error: Private members can't be accessed from outside their modules.",
        input
    );
}

#[test]
fn use_cannot_see_modules_imported_by_other_modules() {
    let input = r#"
       @use "use_cannot_see_modules_imported_by_other_modules__a" as a;
       @use "use_cannot_see_modules_imported_by_other_modules__b" as b;"#;

    tempfile!(
        "use_cannot_see_modules_imported_by_other_modules__a.scss",
        "$a: green;"
    );
    tempfile!(
        "use_cannot_see_modules_imported_by_other_modules__b.scss",
        "a { color: a.$a; }"
    );

    assert_err!("Error: There is no module with the namespace \"a\".", input);
}
#[test]
fn use_can_see_modules_imported_by_other_modules_when_aliased_as_star() {
    let input = r#"
       @use "use_can_see_modules_imported_by_other_modules_when_aliased_as_star__a" as *;
       a { color: math.$e; }
    "#;

    tempfile!(
        "use_can_see_modules_imported_by_other_modules_when_aliased_as_star__a.scss",
        "@use \"sass:math\";"
    );

    assert_err!(
        r#"Error: There is no module with the namespace "math"."#,
        input
    );
}

#[test]
fn use_modules_imported_by_other_modules_does_not_cause_conflict() {
    let input = r#"
       @use "use_modules_imported_by_other_modules_does_not_cause_conflict__a" as a;
       @use "use_modules_imported_by_other_modules_does_not_cause_conflict__b" as b;"#;

    tempfile!(
        "use_modules_imported_by_other_modules_does_not_cause_conflict__a.scss",
        "$a: red;"
    );
    tempfile!(
        "use_modules_imported_by_other_modules_does_not_cause_conflict__b.scss",
        "@use \"use_modules_imported_by_other_modules_does_not_cause_conflict__a\" as a; a { color: a.$a; }"
    );

    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_mixin_can_use_scope_from_own_module() {
    let input = r#"
        @use "use_mixin_can_use_scope_from_own_module__a" as a;
        @include a.foo();
    "#;

    tempfile!(
        "use_mixin_can_use_scope_from_own_module__a.scss",
        "$a: red;

        @mixin foo() {
          a {
            color: $a;
          }
        }"
    );

    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_function_can_use_scope_from_own_module() {
    let input = r#"
        @use "use_function_can_use_scope_from_own_module__a" as a;

        a {
            color: a.foo();
        }
    "#;

    tempfile!(
        "use_function_can_use_scope_from_own_module__a.scss",
        "$a: red;

        @function foo() {
            @return $a;
        }"
    );

    assert_eq!(
        "a {\n  color: red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

#[test]
fn use_variable_redeclaration_builtin() {
    let input = "@use \"sass:math\";\nmath.$e: red;";

    assert_err!("Error: Cannot modify built-in variable.", input);
}

#[test]
fn use_variable_declaration_between_use() {
    let input = r#"
        $a: red;
        $b: green;
        @use "sass:math";
        $b: red;
        @use "sass:meta";
        a {
            color: $a $b;
        }"#;

    assert_eq!(
        "a {\n  color: red red;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}
