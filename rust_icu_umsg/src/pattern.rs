// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Counts the arguments a MessageFormat pattern refers to.
//!
//! ICU's `umsg_format` is variadic: it reads its arguments based on the
//! *pattern*, not on what the caller actually passed.  The number it reads is
//! the highest argument index in the pattern, plus one.  Supplying fewer
//! arguments than that makes ICU read past the end of the argument list, which
//! is undefined behavior.  See
//! [google/rust_icu#371](https://github.com/google/rust_icu/issues/371).
//!
//! [required_arg_count] scans a pattern for that number so the caller can be
//! checked before the variadic call happens.  The scanner mirrors the grammar
//! that ICU's `MessagePattern` implements, and is deliberately conservative:
//! anything it does not model exactly makes it return [None], which means "do
//! not check".  Returning [None] leaves the previous behavior in place, whereas
//! returning a number that is too large would reject calls that ICU handles
//! fine, so the scanner errs towards [None] whenever it is unsure.
//!
//! The scanner only ever runs on a pattern that `umsg_open` already accepted,
//! so it does not need to diagnose malformed patterns.

/// The construct a message body sits in.
///
/// Only used to reproduce ICU's context-sensitive apostrophe rule: a single
/// apostrophe starts quoted literal text when it precedes `{` or `}`, a `|`
/// inside a `choice` argument, or a `#` inside a `plural` or `selectordinal`
/// argument.  Everywhere else a single apostrophe is literal text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Parent {
    /// The whole pattern.
    Top,
    /// A submessage of a `choice` argument.
    Choice,
    /// A submessage of a `plural` or `selectordinal` argument.
    Plural,
    /// A submessage of a `select` argument.
    Select,
}

/// Returns the number of arguments `umsg_format` will read for `pattern`, or
/// [None] if the pattern uses something this scanner does not model.
///
/// The count is the highest argument index the pattern refers to, plus one, so
/// a pattern that refers only to `{1}` still needs two arguments.  A pattern
/// with no arguments needs none.
///
/// [None] is returned for patterns that use named arguments (`{name}`), which
/// `umsg_format` rejects outright, and for anything the scanner cannot parse.
pub(crate) fn required_arg_count(pattern: &str) -> Option<usize> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut scanner = Scanner {
        pattern: &chars,
        pos: 0,
        max_index: None,
    };
    scanner.message(Parent::Top)?;
    Some(match scanner.max_index {
        None => 0,
        Some(index) => index as usize + 1,
    })
}

struct Scanner<'a> {
    pattern: &'a [char],
    pos: usize,
    /// The highest argument index seen so far.
    max_index: Option<u32>,
}

impl<'a> Scanner<'a> {
    fn peek(&self) -> Option<char> {
        self.pattern.get(self.pos).copied()
    }

    fn next_char(&self) -> Option<char> {
        self.pattern.get(self.pos + 1).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn record(&mut self, index: u32) {
        self.max_index = Some(match self.max_index {
            None => index,
            Some(seen) => std::cmp::max(seen, index),
        });
    }

    /// Scans a message body, recording every argument index it refers to.
    ///
    /// Stops on the `}` that closes the enclosing argument without consuming
    /// it, or at the end of the pattern for [Parent::Top].
    fn message(&mut self, parent: Parent) -> Option<()> {
        while let Some(c) = self.peek() {
            match c {
                '\'' => self.quoted_literal(parent),
                '{' => {
                    self.pos += 1;
                    self.argument()?;
                }
                // Only a nested message ends on a closing brace.  At the top
                // level ICU treats a stray `}` as literal text.
                '}' if parent != Parent::Top => return Some(()),
                _ => self.pos += 1,
            }
        }
        // Running out of pattern is the normal end of the whole message, but
        // means a nested message was never closed.
        if parent == Parent::Top {
            Some(())
        } else {
            None
        }
    }

    /// Steps over an apostrophe and any literal text it quotes.
    ///
    /// Called with the scanner sitting on the apostrophe.
    fn quoted_literal(&mut self, parent: Parent) {
        let starts_quote = match self.next_char() {
            // A doubled apostrophe is one literal apostrophe.
            Some('\'') => {
                self.pos += 2;
                return;
            }
            Some('{') | Some('}') => true,
            Some('|') => parent == Parent::Choice,
            Some('#') => parent == Parent::Plural,
            _ => false,
        };
        if !starts_quote {
            // A lone apostrophe is literal text.
            self.pos += 1;
            return;
        }
        // Skip the opening apostrophe, then run to the closing one.  A doubled
        // apostrophe inside the quoted text is a literal apostrophe and keeps
        // the text quoted.  Unterminated quoted text runs to the end.
        self.pos += 1;
        while let Some(c) = self.peek() {
            self.pos += 1;
            if c == '\'' {
                if self.peek() == Some('\'') {
                    self.pos += 1;
                } else {
                    return;
                }
            }
        }
    }

    /// Scans an argument, with the scanner just past its opening `{`.
    fn argument(&mut self) -> Option<()> {
        let name = self.take_until_comma_or_brace();
        // A named argument makes `umsg_format` fail with
        // U_ARGUMENT_TYPE_MISMATCH before it reads anything, so there is
        // nothing to check and no count to report.
        let index = arg_number(&name)?;
        self.record(index);

        match self.peek()? {
            '}' => {
                self.pos += 1;
                Some(())
            }
            ',' => {
                self.pos += 1;
                self.arg_type()
            }
            _ => None,
        }
    }

    /// Scans an argument's type and style, just past the `,` after its index.
    fn arg_type(&mut self) -> Option<()> {
        let arg_type = self.take_until_comma_or_brace();
        if arg_type.is_empty() {
            return None;
        }
        match self.peek()? {
            // A type with no style, such as `{0,number}`.  No arguments can
            // hide in a style that is not there.
            '}' => {
                self.pos += 1;
                Some(())
            }
            ',' => {
                self.pos += 1;
                // ICU matches these type names case-insensitively.
                let arg_type = arg_type.to_ascii_lowercase();
                match arg_type.as_str() {
                    // A `choice` style holds its submessages inline, separated
                    // by `|`, so the whole style scans as one message body.
                    "choice" => {
                        self.message(Parent::Choice)?;
                        self.expect('}')
                    }
                    "plural" | "selectordinal" => self.selector_style(Parent::Plural),
                    "select" => self.selector_style(Parent::Select),
                    // Everything else is a simple argument, whose style is
                    // literal text that cannot contain arguments.
                    _ => self.simple_style(),
                }
            }
            _ => None,
        }
    }

    /// Scans a `plural`, `selectordinal` or `select` style: a run of
    /// `selector {submessage}` pairs, ending on the argument's closing `}`.
    fn selector_style(&mut self, parent: Parent) -> Option<()> {
        loop {
            self.skip_whitespace();
            if self.peek()? == '}' {
                self.pos += 1;
                return Some(());
            }
            // The selector, or a `plural` argument's leading `offset:value`.
            let start = self.pos;
            while let Some(c) = self.peek() {
                if c.is_whitespace() || c == '{' || c == '}' {
                    break;
                }
                self.pos += 1;
            }
            if self.pos == start {
                return None;
            }
            self.skip_whitespace();
            // An `offset:value` is followed by the first selector rather than
            // by a submessage, so go around again to read that selector.
            if self.peek()? != '{' {
                continue;
            }
            self.pos += 1;
            self.message(parent)?;
            self.expect('}')?;
        }
    }

    /// Scans a simple argument's style, such as the `##.#` of
    /// `{0,number,##.#}`.
    ///
    /// ICU reads it as literal text up to the argument's closing brace, where
    /// any apostrophe quotes text up to the next apostrophe, and braces have to
    /// balance.  Nothing in it is an argument.
    fn simple_style(&mut self) -> Option<()> {
        let mut nested_braces = 0usize;
        loop {
            let c = self.peek()?;
            self.pos += 1;
            match c {
                '\'' => {
                    // Unlike in a message body, every apostrophe quotes here.
                    loop {
                        match self.peek()? {
                            '\'' => {
                                self.pos += 1;
                                break;
                            }
                            _ => self.pos += 1,
                        }
                    }
                }
                '{' => nested_braces += 1,
                '}' => {
                    if nested_braces == 0 {
                        return Some(());
                    }
                    nested_braces -= 1;
                }
                _ => {}
            }
        }
    }

    /// Consumes the text up to the next `,` or `}`, returning it trimmed.
    ///
    /// ICU allows whitespace around an argument's index and around its type
    /// name, so both are trimmed here.  The delimiter itself is left for the
    /// caller.
    fn take_until_comma_or_brace(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == ',' || c == '}' {
                break;
            }
            self.pos += 1;
        }
        self.pattern[start..self.pos]
            .iter()
            .collect::<String>()
            .trim()
            .to_string()
    }

    fn expect(&mut self, c: char) -> Option<()> {
        if self.peek()? == c {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }
}

/// Reads an argument index the way ICU's `MessagePattern` does.
///
/// Returns [None] for anything ICU would treat as an argument *name* instead,
/// which includes an empty identifier, a non-digit, and a number with a leading
/// zero.  ICU also caps argument numbers at [i32::MAX].
fn arg_number(name: &str) -> Option<u32> {
    let name = name.trim();
    let mut digits = name.chars();
    let first = digits.next()?;
    if !first.is_ascii_digit() {
        return None;
    }
    // "0" is a number, "01" is a name.
    if first == '0' && name.len() > 1 {
        return None;
    }
    let mut index = first.to_digit(10)?;
    for c in digits {
        index = index.checked_mul(10)?.checked_add(c.to_digit(10)?)?;
        if index > i32::MAX as u32 {
            return None;
        }
    }
    Some(index)
}

#[cfg(test)]
mod tests {
    use super::required_arg_count;

    #[test]
    fn no_arguments() {
        assert_eq!(Some(0), required_arg_count(""));
        assert_eq!(Some(0), required_arg_count("plain text"));
    }

    #[test]
    fn counts_highest_index_plus_one() {
        assert_eq!(Some(1), required_arg_count("{0}"));
        assert_eq!(Some(2), required_arg_count("{0} {1}"));
        assert_eq!(Some(2), required_arg_count("{1} {0}"));
        // The pattern from google/rust_icu#371: `{1}` alone still needs two.
        assert_eq!(Some(2), required_arg_count("String : {1}"));
        assert_eq!(Some(11), required_arg_count("{10}"));
    }

    #[test]
    fn ignores_whitespace_around_the_index() {
        assert_eq!(Some(2), required_arg_count("{ 1 }"));
        assert_eq!(Some(2), required_arg_count("{ 1 , number }"));
    }

    #[test]
    fn counts_arguments_with_a_type_and_style() {
        assert_eq!(Some(1), required_arg_count("{0,number}"));
        assert_eq!(Some(1), required_arg_count("{0,number,##.#}"));
        assert_eq!(
            Some(4),
            required_arg_count("{0,number,##.#} {1,number,integer} {2} {3,date,full}")
        );
    }

    #[test]
    fn a_simple_style_holds_no_arguments() {
        // The braces in the style are part of the style, not arguments.
        assert_eq!(Some(1), required_arg_count("{0,number,#{9}#}"));
        // Neither is an argument that the style quotes away.
        assert_eq!(Some(1), required_arg_count("{0,number,'{9}'}"));
    }

    #[test]
    fn counts_nested_arguments() {
        assert_eq!(
            Some(2),
            required_arg_count("{0,plural,one{one file}other{# files in {1}}}")
        );
        assert_eq!(
            Some(3),
            required_arg_count("{0,select,male{{1}}female{{2}}other{}}")
        );
        assert_eq!(
            Some(2),
            required_arg_count("{0,choice,0#no files|1#one file|1<{1} files}")
        );
        assert_eq!(
            Some(3),
            required_arg_count("{0,plural,offset:1 one{{1}}other{{2}}}")
        );
        assert_eq!(
            Some(2),
            required_arg_count("{0,selectordinal,one{#st}other{#th in {1}}}")
        );
    }

    #[test]
    fn a_selector_is_not_an_argument() {
        // The braces after `one` and `other` open submessages.  Their text
        // happens to be a bare number, which is not an argument index.
        assert_eq!(Some(1), required_arg_count("{0,plural,one{1}other{9}}"));
        assert_eq!(Some(1), required_arg_count("{0,select,other{9}}"));
    }

    #[test]
    fn quoted_arguments_do_not_count() {
        assert_eq!(Some(0), required_arg_count("'{0}'"));
        assert_eq!(Some(1), required_arg_count("'{9}' {0}"));
        // A doubled apostrophe is a literal apostrophe, so `{9}` is quoted.
        assert_eq!(Some(1), required_arg_count("it''s '{9}' {0}"));
        // A lone apostrophe not before a brace is literal text, so `{9}`
        // is a real argument.
        assert_eq!(Some(10), required_arg_count("it's {9}"));
        // Quoted text runs to the closing apostrophe, and a doubled
        // apostrophe inside it does not end it.
        assert_eq!(Some(0), required_arg_count("'{9} it'' still quoted {8}'"));
    }

    #[test]
    fn a_quoted_pound_only_quotes_inside_plural() {
        // Inside a plural submessage `'#` starts quoted text, hiding `{9}`.
        assert_eq!(Some(1), required_arg_count("{0,plural,other{'# {9}'}}"));
        // At the top level it does not, so `{9}` counts.
        assert_eq!(Some(10), required_arg_count("'# {9}'"));
    }

    #[test]
    fn a_quoted_pipe_only_quotes_inside_choice() {
        assert_eq!(Some(1), required_arg_count("{0,choice,0#'| {9}'}"));
        assert_eq!(Some(10), required_arg_count("'| {9}'"));
    }

    #[test]
    fn named_arguments_are_not_counted() {
        assert_eq!(None, required_arg_count("{name}"));
        assert_eq!(None, required_arg_count("{0} {name}"));
        // ICU reads a number with a leading zero as a name.
        assert_eq!(None, required_arg_count("{01}"));
        assert_eq!(None, required_arg_count("{}"));
    }

    #[test]
    fn unparseable_patterns_are_not_counted() {
        assert_eq!(None, required_arg_count("{0"));
        assert_eq!(None, required_arg_count("{0,number"));
        assert_eq!(None, required_arg_count("{0,plural,other{"));
        assert_eq!(None, required_arg_count("{0,}"));
    }

    #[test]
    fn a_stray_closing_brace_is_literal_text() {
        assert_eq!(Some(1), required_arg_count("a } {0}"));
    }
}
