/*! # grass
An implementation of Sass in pure rust.

Spec progress as of 0.11.2, released on 2022-09-03:

| Passing | Failing | Total |
|---------|---------|-------|
| 4205    | 2051    | 6256  |

## Use as library
```
fn main() -> Result<(), Box<grass::Error>> {
    let sass = grass::from_string("a { b { color: &; } }".to_string(), &grass::Options::default())?;
    assert_eq!(sass, "a b {\n  color: a b;\n}\n");
    Ok(())
}
```

## Use as binary
```bash
cargo install grass
grass input.scss
```
*/

#![allow(warnings)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(missing_debug_implementations)]
#![allow(
    clippy::use_self,
    clippy::missing_docs_in_private_items,
    clippy::unreachable,
    clippy::module_name_repetitions,
    // filter isn't fallible
    clippy::manual_filter_map,
    clippy::new_ret_no_self,
    renamed_and_removed_lints,
    clippy::unknown_clippy_lints,
    clippy::single_match,
    clippy::unimplemented,
    clippy::option_if_let_else,
    clippy::branches_sharing_code,
    clippy::derive_partial_eq_without_eq,

    // temporarily allowed while under heavy development.
    // eventually these allows should be refactored away
    // to no longer be necessary
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::single_match_else,
    clippy::redundant_pub_crate,
    // the api is changing too often to allot this
    clippy::missing_errors_doc,
    clippy::missing_const_for_fn,
    clippy::multiple_crate_versions,

    clippy::wrong_self_convention,
    clippy::items_after_statements,
    // this is only available on nightly
    clippy::unnested_or_patterns,
    clippy::uninlined_format_args,
)]
#![cfg_attr(feature = "profiling", inline(never))]

use std::path::Path;

#[cfg(feature = "wasm-exports")]
use wasm_bindgen::prelude::*;

pub(crate) use beef::lean::Cow;

use codemap::CodeMap;

pub use crate::error::{
    PublicSassErrorKind as ErrorKind, SassError as Error, SassResult as Result,
};
pub use crate::fs::{Fs, NullFs, StdFs};
pub(crate) use crate::token::Token;
use crate::{
    builtin::modules::{ModuleConfig, Modules},
    lexer::Lexer,
    output::{AtRuleContext, Css},
    parse::{common::ContextFlags, visitor::Visitor, Parser},
};

mod args;
mod atrule;
mod builtin;
mod color;
mod common;
mod error;
mod fs;
mod interner;
mod lexer;
mod output;
mod parse;
mod scope;
mod selector;
mod serializer;
mod style;
mod token;
mod unit;
mod utils;
mod value;

#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum OutputStyle {
    /// The default style, this mode writes each
    /// selector and declaration on its own line.
    Expanded,
    /// Ideal for release builds, this mode removes
    /// as many extra characters as possible and
    /// writes the entire stylesheet on a single line.
    Compressed,
}

/// Configuration for Sass compilation
///
/// The simplest usage is `grass::Options::default()`;
/// however, a builder pattern is also exposed to offer
/// more control.
#[derive(Debug)]
pub struct Options<'a> {
    fs: &'a dyn Fs,
    style: OutputStyle,
    load_paths: Vec<&'a Path>,
    allows_charset: bool,
    unicode_error_messages: bool,
    quiet: bool,
}

impl Default for Options<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            fs: &StdFs,
            style: OutputStyle::Expanded,
            load_paths: Vec::new(),
            allows_charset: true,
            unicode_error_messages: true,
            quiet: false,
        }
    }
}

impl<'a> Options<'a> {
    /// This option allows you to control the file system that Sass will see.
    ///
    /// By default, it uses [`StdFs`], which is backed by [`std::fs`],
    /// allowing direct, unfettered access to the local file system.
    #[must_use]
    #[inline]
    pub fn fs(mut self, fs: &'a dyn Fs) -> Self {
        self.fs = fs;
        self
    }

    /// `grass` currently offers 2 different output styles
    ///
    ///  - `OutputStyle::Expanded` writes each selector and declaration on its own line.
    ///  - `OutputStyle::Compressed` removes as many extra characters as possible
    ///    and writes the entire stylesheet on a single line.
    ///
    /// By default, output is expanded.
    #[must_use]
    #[inline]
    pub const fn style(mut self, style: OutputStyle) -> Self {
        self.style = style;
        self
    }

    /// This flag tells Sass not to emit any warnings
    /// when compiling. By default, Sass emits warnings
    /// when deprecated features are used or when the
    /// `@warn` rule is encountered. It also silences the
    /// `@debug` rule.
    ///
    /// By default, this value is `false` and warnings are emitted.
    #[must_use]
    #[inline]
    pub const fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// All Sass implementations allow users to provide
    /// load paths: paths on the filesystem that Sass
    /// will look in when locating modules. For example,
    /// if you pass `node_modules/susy/sass` as a load path,
    /// you can use `@import "susy"` to load `node_modules/susy/sass/susy.scss`.
    ///
    /// Imports will always be resolved relative to the current
    /// file first, though. Load paths will only be used if no
    /// relative file exists that matches the module's URL. This
    /// ensures that you can't accidentally mess up your relative
    /// imports when you add a new library.
    ///
    /// This method will append a single path to the list.
    #[must_use]
    #[inline]
    pub fn load_path(mut self, path: &'a Path) -> Self {
        self.load_paths.push(path);
        self
    }

    /// Append multiple loads paths
    ///
    /// Note that this method does *not* remove existing load paths
    ///
    /// See [`Options::load_path`](Options::load_path) for more information about load paths
    #[must_use]
    #[inline]
    pub fn load_paths(mut self, paths: &'a [&'a Path]) -> Self {
        self.load_paths.extend_from_slice(paths);
        self
    }

    /// This flag tells Sass whether to emit a `@charset`
    /// declaration or a UTF-8 byte-order mark.
    ///
    /// By default, Sass will insert either a `@charset`
    /// declaration (in expanded output mode) or a byte-order
    /// mark (in compressed output mode) if the stylesheet
    /// contains any non-ASCII characters.
    #[must_use]
    #[inline]
    pub const fn allows_charset(mut self, allows_charset: bool) -> Self {
        self.allows_charset = allows_charset;
        self
    }

    /// This flag tells Sass only to emit ASCII characters as
    /// part of error messages.
    ///
    /// By default Sass will emit non-ASCII characters for
    /// these messages.
    ///
    /// This flag does not affect the CSS output.
    #[must_use]
    #[inline]
    pub const fn unicode_error_messages(mut self, unicode_error_messages: bool) -> Self {
        self.unicode_error_messages = unicode_error_messages;
        self
    }

    pub(crate) fn is_compressed(&self) -> bool {
        matches!(self.style, OutputStyle::Compressed)
    }
}

fn raw_to_parse_error(map: &CodeMap, err: Error, unicode: bool) -> Box<Error> {
    let (message, span) = err.raw();
    Box::new(Error::from_loc(message, map.look_up_span(span), unicode))
}

fn from_string_with_file_name(input: String, file_name: &str, options: &Options) -> Result<String> {
    let mut map = CodeMap::new();
    let file = map.add_file(file_name.to_owned(), input);
    let empty_span = file.span.subspan(0, 0);

    let mut parser = Parser {
        toks: &mut Lexer::new_from_file(&file),
        map: &mut map,
        path: file_name.as_ref(),
        is_plain_css: false,
        // scopes: &mut Scopes::new(),
        // global_scope: &mut Scope::new(),
        // super_selectors: &mut NeverEmptyVec::new(ExtendedSelector::new(SelectorList::new(
        //     empty_span,
        // ))),
        span_before: empty_span,
        // content: &mut Vec::new(),
        flags: ContextFlags::empty(),
        // at_root: true,
        // at_root_has_selector: false,
        // extender: &mut Extender::new(empty_span),
        // content_scopes: &mut Scopes::new(),
        options,
        modules: &mut Modules::default(),
        module_config: &mut ModuleConfig::default(),
    };

    let stmts = match parser.__parse() {
        Ok(v) => v,
        Err(e) => return Err(raw_to_parse_error(&map, *e, options.unicode_error_messages)),
    };

    let mut visitor = Visitor::new(&mut parser);
    match visitor.visit_stylesheet(stmts) {
        Ok(_) => {}
        Err(e) => return Err(raw_to_parse_error(&map, *e, options.unicode_error_messages)),
    }
    let stmts = match visitor.finish() {
        Ok(v) => v,
        Err(e) => return Err(raw_to_parse_error(&map, *e, options.unicode_error_messages)),
    };

    Css::from_stmts(stmts, AtRuleContext::None, options.allows_charset)
        .map_err(|e| raw_to_parse_error(&map, *e, options.unicode_error_messages))?
        .pretty_print(&map, options.style)
        .map_err(|e| raw_to_parse_error(&map, *e, options.unicode_error_messages))
}

/// Compile CSS from a path
///
/// n.b. grass does not currently support files or paths that are not valid UTF-8
///
/// ```
/// fn main() -> Result<(), Box<grass::Error>> {
///     let sass = grass::from_path("input.scss", &grass::Options::default())?;
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "profiling", inline(never))]
#[cfg_attr(not(feature = "profiling"), inline)]
pub fn from_path(p: &str, options: &Options) -> Result<String> {
    from_string_with_file_name(
        String::from_utf8(options.fs.read(Path::new(p))?)?,
        p,
        options,
    )
}

/// Compile CSS from a string
///
/// ```
/// fn main() -> Result<(), Box<grass::Error>> {
///     let sass = grass::from_string("a { b { color: &; } }".to_string(), &grass::Options::default())?;
///     assert_eq!(sass, "a b {\n  color: a b;\n}\n");
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "profiling", inline(never))]
#[cfg_attr(not(feature = "profiling"), inline)]
pub fn from_string(input: String, options: &Options) -> Result<String> {
    from_string_with_file_name(input, "stdin", options)
}

#[cfg(feature = "wasm-exports")]
#[wasm_bindgen(js_name = from_string)]
pub fn from_string_js(input: String) -> std::result::Result<String, String> {
    from_string(input, &Options::default()).map_err(|e| e.to_string())
}
