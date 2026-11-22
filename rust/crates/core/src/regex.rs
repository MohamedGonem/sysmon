//! A small, dependency-free regular-expression engine.
//!
//! It supports a practical subset of POSIX extended regular expressions:
//! literals, `.`, `*`, `+`, `?`, `{m}`, `{m,}`, `{m,n}`, character classes
//! (`[abc]`, `[a-z]`, `[^...]`), groups (`(...)` and `(?:...)`), alternation,
//! anchors (`^`, `$`), word boundaries (`\b`, `\B`), and the escapes `\d`,
//! `\w`, `\s`, and their negated forms. Matching is ASCII case-insensitive.
//!
//! POSIX bracket classes such as `[[:digit:]]` are not supported. Inside a
//! character class, `\d`, `\w`, and `\s` are treated as literal characters;
//! use ranges such as `[0-9]` instead.
//!
//! This intentionally stays in pure Rust so the core remains portable to
//! platforms that do not provide POSIX regular expressions.

use std::fmt;

const MAX_PATTERN_CHARS: usize = 10_000;
const MAX_REPEAT: usize = 1024;
const MAX_INSTRUCTIONS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexError {
    message: String,
}

impl RegexError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RegexError {}

#[derive(Debug)]
pub struct Regex {
    instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
enum Instruction {
    Char(char),
    Any,
    Class(CharClass),
    Split(usize, usize),
    Jump(usize),
    AssertStart,
    AssertEnd,
    AssertWordBoundary(bool),
    Match,
}

#[derive(Debug, Clone)]
struct CharClass {
    negated: bool,
    ranges: Vec<(char, char)>,
}

impl CharClass {
    fn matches(&self, character: char) -> bool {
        self.contains(character) != self.negated
    }

    fn contains(&self, character: char) -> bool {
        self.in_ranges(character)
            || (character.is_ascii_alphabetic() && self.in_ranges(swap_ascii_case(character)))
    }

    fn in_ranges(&self, character: char) -> bool {
        self.ranges
            .iter()
            .any(|&(start, end)| start <= character && character <= end)
    }
}

fn swap_ascii_case(character: char) -> char {
    if character.is_ascii_lowercase() {
        character.to_ascii_uppercase()
    } else {
        character.to_ascii_lowercase()
    }
}

fn chars_equal(left: char, right: char) -> bool {
    left.eq_ignore_ascii_case(&right)
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        if pattern.chars().count() > MAX_PATTERN_CHARS {
            return Err(RegexError::new(format!(
                "pattern exceeds {MAX_PATTERN_CHARS} characters"
            )));
        }

        let expression = Parser::new(pattern).parse()?;
        let mut compiler = Compiler::default();
        compiler.compile(&expression)?;
        compiler.emit(Instruction::Match);

        Ok(Self {
            instructions: compiler.instructions,
        })
    }

    pub fn is_match(&self, text: &str) -> bool {
        let characters: Vec<char> = text.chars().collect();
        self.is_match_chars(&characters)
    }

    fn is_match_chars(&self, text: &[char]) -> bool {
        let mut current = Vec::new();
        let mut next = Vec::new();
        let mut visited = vec![false; self.instructions.len()];
        let mut next_visited = vec![false; self.instructions.len()];
        let mut stack = Vec::new();

        for position in 0..=text.len() {
            // Starting a new thread at every position makes the search
            // unanchored. `^` still restricts the thread to position 0.
            self.add_thread(&mut current, 0, position, text, &mut visited, &mut stack);

            if current
                .iter()
                .any(|&pc| matches!(self.instructions[pc], Instruction::Match))
            {
                return true;
            }

            if position == text.len() {
                break;
            }

            next.clear();
            next_visited.fill(false);

            for &pc in &current {
                let matches = match &self.instructions[pc] {
                    Instruction::Char(expected) => chars_equal(*expected, text[position]),
                    Instruction::Any => true,
                    Instruction::Class(class) => class.matches(text[position]),
                    _ => false,
                };

                if matches {
                    self.add_thread(
                        &mut next,
                        pc + 1,
                        position + 1,
                        text,
                        &mut next_visited,
                        &mut stack,
                    );
                }
            }

            std::mem::swap(&mut current, &mut next);
            std::mem::swap(&mut visited, &mut next_visited);
        }

        false
    }

    fn add_thread(
        &self,
        list: &mut Vec<usize>,
        start: usize,
        position: usize,
        text: &[char],
        visited: &mut [bool],
        stack: &mut Vec<usize>,
    ) {
        stack.clear();
        stack.push(start);

        while let Some(pc) = stack.pop() {
            if visited[pc] {
                continue;
            }
            visited[pc] = true;

            match &self.instructions[pc] {
                Instruction::Jump(target) => stack.push(*target),
                Instruction::Split(first, second) => {
                    stack.push(*second);
                    stack.push(*first);
                }
                Instruction::AssertStart => {
                    if position == 0 {
                        stack.push(pc + 1);
                    }
                }
                Instruction::AssertEnd => {
                    if position == text.len() {
                        stack.push(pc + 1);
                    }
                }
                Instruction::AssertWordBoundary(positive) => {
                    let before = position
                        .checked_sub(1)
                        .and_then(|index| text.get(index))
                        .copied();
                    let after = text.get(position).copied();

                    if (is_word(before) != is_word(after)) == *positive {
                        stack.push(pc + 1);
                    }
                }
                _ => list.push(pc),
            }
        }
    }
}

fn is_word(character: Option<char>) -> bool {
    character.is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[derive(Debug)]
enum Expression {
    Empty,
    Literal(char),
    Any,
    Class(CharClass),
    Start,
    End,
    WordBoundary(bool),
    Concat(Vec<Expression>),
    Alternate(Vec<Expression>),
    Repeat {
        expression: Box<Expression>,
        min: usize,
        max: Option<usize>,
    },
}

struct Parser {
    characters: Vec<char>,
    position: usize,
}

impl Parser {
    fn new(pattern: &str) -> Self {
        Self {
            characters: pattern.chars().collect(),
            position: 0,
        }
    }

    fn parse(mut self) -> Result<Expression, RegexError> {
        let expression = self.parse_alternation()?;

        if let Some(character) = self.peek() {
            return Err(self.error(format!("unexpected '{character}'")));
        }

        Ok(expression)
    }

    fn parse_alternation(&mut self) -> Result<Expression, RegexError> {
        let mut branches = vec![self.parse_concat()?];

        while self.eat('|') {
            branches.push(self.parse_concat()?);
        }

        Ok(if branches.len() == 1 {
            branches.remove(0)
        } else {
            Expression::Alternate(branches)
        })
    }

    fn parse_concat(&mut self) -> Result<Expression, RegexError> {
        let mut expressions = Vec::new();

        while let Some(character) = self.peek() {
            if character == '|' || character == ')' {
                break;
            }
            expressions.push(self.parse_repetition()?);
        }

        Ok(match expressions.len() {
            0 => Expression::Empty,
            1 => expressions.remove(0),
            _ => Expression::Concat(expressions),
        })
    }

    fn parse_repetition(&mut self) -> Result<Expression, RegexError> {
        let expression = self.parse_atom()?;
        let Some(character) = self.peek() else {
            return Ok(expression);
        };

        let (min, max) = match character {
            '*' => {
                self.position += 1;
                (0, None)
            }
            '+' => {
                self.position += 1;
                (1, None)
            }
            '?' => {
                self.position += 1;
                (0, Some(1))
            }
            '{' => self.parse_bounds()?,
            _ => return Ok(expression),
        };

        // Lazy and greedy quantifiers are equivalent for a boolean match.
        self.eat('?');

        if matches!(self.peek(), Some('*' | '+' | '?' | '{')) {
            return Err(self.error("multiple repetition operators"));
        }

        Ok(Expression::Repeat {
            expression: Box::new(expression),
            min,
            max,
        })
    }

    fn parse_bounds(&mut self) -> Result<(usize, Option<usize>), RegexError> {
        debug_assert_eq!(self.peek(), Some('{'));
        self.position += 1;

        let min = self.parse_number()?;

        if self.eat('}') {
            return Ok((min, Some(min)));
        }

        if !self.eat(',') {
            return Err(self.error("expected ',' or '}' in repetition"));
        }

        if self.eat('}') {
            return Ok((min, None));
        }

        let max = self.parse_number()?;

        if !self.eat('}') {
            return Err(self.error("expected '}' in repetition"));
        }

        if max < min {
            return Err(self.error("repetition maximum is smaller than minimum"));
        }

        Ok((min, Some(max)))
    }

    fn parse_number(&mut self) -> Result<usize, RegexError> {
        let start = self.position;
        let mut value = 0usize;

        while let Some(character) = self.peek() {
            if !character.is_ascii_digit() {
                break;
            }
            self.position += 1;
            value = value
                .checked_mul(10)
                .and_then(|value| value.checked_add(character as usize - '0' as usize))
                .ok_or_else(|| self.error("repetition is too large"))?;

            if value > MAX_REPEAT {
                return Err(self.error(format!("repetition exceeds {MAX_REPEAT}")));
            }
        }

        if self.position == start {
            return Err(self.error("expected a number in repetition"));
        }

        Ok(value)
    }

    fn parse_atom(&mut self) -> Result<Expression, RegexError> {
        let Some(character) = self.next() else {
            return Err(self.error("expected an expression"));
        };

        match character {
            '(' => {
                if self.eat('?') && !self.eat(':') {
                    return Err(self.error("only non-capturing groups '(?:...)' are supported"));
                }

                let expression = self.parse_alternation()?;

                if !self.eat(')') {
                    return Err(self.error("missing ')'"));
                }

                Ok(expression)
            }
            '[' => self.parse_class(),
            '.' => Ok(Expression::Any),
            '^' => Ok(Expression::Start),
            '$' => Ok(Expression::End),
            '\\' => self.parse_escape(),
            ')' => Err(self.error("unmatched ')'")),
            '*' | '+' | '?' => Err(self.error(format!("nothing to repeat '{character}'"))),
            character => Ok(Expression::Literal(character)),
        }
    }

    fn parse_escape(&mut self) -> Result<Expression, RegexError> {
        let Some(character) = self.next() else {
            return Err(self.error("trailing '\\'"));
        };

        Ok(match character {
            'd' => Expression::Class(CharClass {
                negated: false,
                ranges: vec![('0', '9')],
            }),
            'D' => Expression::Class(CharClass {
                negated: true,
                ranges: vec![('0', '9')],
            }),
            'w' => Expression::Class(CharClass {
                negated: false,
                ranges: word_ranges(),
            }),
            'W' => Expression::Class(CharClass {
                negated: true,
                ranges: word_ranges(),
            }),
            's' => Expression::Class(CharClass {
                negated: false,
                ranges: whitespace_ranges(),
            }),
            'S' => Expression::Class(CharClass {
                negated: true,
                ranges: whitespace_ranges(),
            }),
            'b' => Expression::WordBoundary(true),
            'B' => Expression::WordBoundary(false),
            'n' => Expression::Literal('\n'),
            't' => Expression::Literal('\t'),
            'r' => Expression::Literal('\r'),
            character => Expression::Literal(character),
        })
    }

    fn parse_class(&mut self) -> Result<Expression, RegexError> {
        let negated = self.eat('^');
        let mut ranges = Vec::new();
        let mut first = true;

        loop {
            let Some(character) = self.peek() else {
                return Err(self.error("missing ']' in character class"));
            };

            if character == ']' && !first {
                self.position += 1;
                break;
            }

            if character == '[' && self.characters.get(self.position + 1) == Some(&':') {
                return Err(
                    self.error("POSIX character classes are not supported; use \\d, \\w, or \\s")
                );
            }

            first = false;
            let start = self.parse_class_character()?;

            if self.peek() == Some('-')
                && self
                    .characters
                    .get(self.position + 1)
                    .is_some_and(|character| *character != ']')
            {
                self.position += 1;
                let end = self.parse_class_character()?;

                if start > end {
                    return Err(self.error("character class range is out of order"));
                }

                ranges.push((start, end));
            } else {
                ranges.push((start, start));
            }
        }

        if ranges.is_empty() {
            return Err(self.error("empty character class"));
        }

        Ok(Expression::Class(CharClass { negated, ranges }))
    }

    fn parse_class_character(&mut self) -> Result<char, RegexError> {
        match self.next() {
            Some('\\') => {
                let Some(character) = self.next() else {
                    return Err(self.error("trailing '\\' in character class"));
                };

                Ok(match character {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    character => character,
                })
            }
            Some(character) => Ok(character),
            None => Err(self.error("missing ']' in character class")),
        }
    }

    fn peek(&self) -> Option<char> {
        self.characters.get(self.position).copied()
    }

    fn next(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.position += 1;
        Some(character)
    }

    fn eat(&mut self, character: char) -> bool {
        if self.peek() == Some(character) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn error(&self, message: impl Into<String>) -> RegexError {
        RegexError::new(format!("{} at position {}", message.into(), self.position))
    }
}

fn word_ranges() -> Vec<(char, char)> {
    vec![('0', '9'), ('A', 'Z'), ('_', '_'), ('a', 'z')]
}

fn whitespace_ranges() -> Vec<(char, char)> {
    vec![
        ('\t', '\t'),
        ('\n', '\n'),
        ('\u{000b}', '\u{000b}'),
        ('\u{000c}', '\u{000c}'),
        ('\r', '\r'),
        (' ', ' '),
    ]
}

#[derive(Default)]
struct Compiler {
    instructions: Vec<Instruction>,
}

impl Compiler {
    fn compile(&mut self, expression: &Expression) -> Result<(), RegexError> {
        if self.instructions.len() >= MAX_INSTRUCTIONS {
            return Err(RegexError::new("pattern is too complex"));
        }

        match expression {
            Expression::Empty => {}
            Expression::Literal(character) => {
                self.emit(Instruction::Char(*character));
            }
            Expression::Any => {
                self.emit(Instruction::Any);
            }
            Expression::Class(class) => {
                self.emit(Instruction::Class(class.clone()));
            }
            Expression::Start => {
                self.emit(Instruction::AssertStart);
            }
            Expression::End => {
                self.emit(Instruction::AssertEnd);
            }
            Expression::WordBoundary(positive) => {
                self.emit(Instruction::AssertWordBoundary(*positive));
            }
            Expression::Concat(expressions) => {
                for expression in expressions {
                    self.compile(expression)?;
                }
            }
            Expression::Alternate(branches) => {
                self.compile_alternate(branches)?;
            }
            Expression::Repeat {
                expression,
                min,
                max,
            } => {
                self.compile_repeat(expression, *min, *max)?;
            }
        }

        Ok(())
    }

    fn compile_alternate(&mut self, branches: &[Expression]) -> Result<(), RegexError> {
        let mut end_jumps = Vec::new();

        for (index, branch) in branches.iter().enumerate() {
            if index + 1 < branches.len() {
                let split = self.emit(Instruction::Split(0, 0));
                let body = self.instructions.len();
                self.compile(branch)?;
                let jump = self.emit(Instruction::Jump(0));
                end_jumps.push(jump);
                let next = self.instructions.len();
                self.instructions[split] = Instruction::Split(body, next);
            } else {
                self.compile(branch)?;
            }
        }

        let end = self.instructions.len();
        for jump in end_jumps {
            self.instructions[jump] = Instruction::Jump(end);
        }

        Ok(())
    }

    fn compile_repeat(
        &mut self,
        expression: &Expression,
        min: usize,
        max: Option<usize>,
    ) -> Result<(), RegexError> {
        if let Some(max) = max {
            if max < min {
                return Err(RegexError::new(
                    "repetition maximum is smaller than minimum",
                ));
            }
            if max > MAX_REPEAT {
                return Err(RegexError::new(format!("repetition exceeds {MAX_REPEAT}")));
            }
        }

        for _ in 0..min {
            self.compile(expression)?;
        }

        match max {
            Some(max) => {
                for _ in min..max {
                    self.compile_optional(expression)?;
                }
            }
            None => self.compile_star(expression)?,
        }

        Ok(())
    }

    fn compile_optional(&mut self, expression: &Expression) -> Result<(), RegexError> {
        let split = self.emit(Instruction::Split(0, 0));
        let body = self.instructions.len();
        self.compile(expression)?;
        let end = self.instructions.len();
        self.instructions[split] = Instruction::Split(body, end);

        Ok(())
    }

    fn compile_star(&mut self, expression: &Expression) -> Result<(), RegexError> {
        let split = self.emit(Instruction::Split(0, 0));
        let body = self.instructions.len();
        self.compile(expression)?;
        self.emit(Instruction::Jump(split));
        let end = self.instructions.len();
        self.instructions[split] = Instruction::Split(body, end);

        Ok(())
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        let pc = self.instructions.len();
        self.instructions.push(instruction);
        pc
    }
}

#[cfg(test)]
mod tests {
    use super::Regex;

    fn matches(pattern: &str, text: &str) -> bool {
        Regex::new(pattern)
            .unwrap_or_else(|error| panic!("{pattern:?} should compile: {error}"))
            .is_match(text)
    }

    #[test]
    fn matches_literals_case_insensitively() {
        assert!(matches("firefox", "FireFox"));
        assert!(matches("firefox", "a firefox process"));
        assert!(!matches("firefox", "chrome"));
    }

    #[test]
    fn matches_dot_and_quantifiers() {
        assert!(matches("a.c", "abc"));
        assert!(!matches("a.c", "ac"));

        assert!(matches("ab*c", "ac"));
        assert!(matches("ab*c", "abbbc"));
        assert!(!matches("ab*c", "adc"));

        assert!(!matches("ab+c", "ac"));
        assert!(matches("ab+c", "abc"));

        assert!(matches("ab?c", "ac"));
        assert!(matches("ab?c", "abc"));
        assert!(!matches("ab?c", "abbc"));
    }

    #[test]
    fn matches_bounded_repetition() {
        assert!(matches("^a{2,3}$", "aa"));
        assert!(matches("^a{2,3}$", "aaa"));
        assert!(!matches("^a{2,3}$", "a"));
        assert!(!matches("^a{2,3}$", "aaaa"));
        assert!(matches("^a{2}$", "aa"));
        assert!(!matches("^a{2}$", "aaa"));
        assert!(matches("^a{2,}$", "aaaa"));
    }

    #[test]
    fn matches_anchors() {
        assert!(matches("^sysmon", "sysmon-cli"));
        assert!(!matches("^sysmon", "run sysmon"));
        assert!(matches("cli$", "sysmon-cli"));
        assert!(!matches("cli$", "cli-tool"));
        assert!(matches("^$", ""));
    }

    #[test]
    fn matches_character_classes() {
        assert!(matches("^[a-c]+$", "abc"));
        assert!(matches("^[A-C]+$", "abc"));
        assert!(!matches("^[^a-c]+$", "xay"));
        assert!(matches("^[^a-c]+$", "xyz"));
        assert!(matches(r"^[\]-]+$", "]-"));
    }

    #[test]
    fn matches_groups_and_alternation() {
        assert!(matches("^(foo|bar)$", "bar"));
        assert!(!matches("^(foo|bar)$", "baz"));
        assert!(matches("^(?:ab)+$", "abab"));
        assert!(matches("^a(b|c)d$", "acd"));
    }

    #[test]
    fn matches_escapes_and_word_boundaries() {
        assert!(matches(r"a\.c", "a.c"));
        assert!(!matches(r"a\.c", "abc"));
        assert!(matches(r"\d+", "abc123"));
        assert!(matches(r"^\w+$", "a_b"));
        assert!(matches(r"\s+", "a b"));
        assert!(matches(r"\bfoo\b", "a foo b"));
        assert!(!matches(r"\bfoo\b", "foobar"));
        assert!(matches(r"\Bfoo", "barfoo"));
    }

    #[test]
    fn handles_nested_repetition_without_looping() {
        assert!(matches("^(a*)*$", "aaa"));
        assert!(matches("^(a*)*$", ""));
        assert!(!matches("^(a*)*$", "b"));
        assert!(matches("^(a|b)*$", "abba"));
    }

    #[test]
    fn empty_pattern_matches_everything() {
        assert!(matches("", ""));
        assert!(matches("", "anything"));
    }

    #[test]
    fn rejects_invalid_patterns() {
        for pattern in [
            "(",
            ")",
            "[",
            "[]",
            "*",
            "a**",
            "a{3,1}",
            "a{",
            "a{2",
            r"trailing\",
            "[[:digit:]]",
            "(?=lookahead)",
        ] {
            assert!(
                Regex::new(pattern).is_err(),
                "{pattern:?} should be rejected"
            );
        }
    }

    #[test]
    fn reports_error_positions() {
        let error = Regex::new("ab)").expect_err("unmatched ')' should fail");
        assert!(error.to_string().contains("position 2"), "{error}");
    }
}
