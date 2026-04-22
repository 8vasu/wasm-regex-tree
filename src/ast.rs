// wasm-regex-tree - WebAssembly visualizer for Rust regular expressions.
// Copyright (C) 2026 Soumendra Ganguly

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::node::Node;
use regex_syntax::ast::Ast;
use regex_syntax::ast::{visit, Visitor};

pub struct TreeBuilder {
    nodes: Vec<Node>,
    stack: Vec<usize>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        TreeBuilder {
            nodes: Vec::new(),
            stack: Vec::new(),
        }
    }
}

impl Visitor for TreeBuilder {
    type Output = Vec<Node>;
    type Err = ();

    fn finish(self) -> Result<Vec<Node>, ()> {
        Ok(self.nodes)
    }

    fn visit_pre(&mut self, ast: &Ast) -> Result<(), ()> {
        let parent = self.stack.last().copied();
        let index = self.nodes.len();

        self.nodes.push(Node {
            label: label(ast),
            kind: kind(ast),
            parent,
            children: Vec::new(),
        });

        if let Some(p) = parent {
            self.nodes[p].children.push(index);
        }

        self.stack.push(index);
        Ok(())
    }

    fn visit_post(&mut self, _ast: &Ast) -> Result<(), ()> {
        self.stack.pop();
        Ok(())
    }
}

pub fn build(pattern: &str) -> Result<Vec<Node>, String> {
    let ast = regex_syntax::ast::parse::Parser::new()
        .parse(pattern)
        .map_err(|e| e.to_string())?;
    visit(&ast, TreeBuilder::new()).map_err(|_| "traversal failed".to_string())
}

fn kind(ast: &Ast) -> String {
    match ast {
        Ast::Empty(_) => "empty",
        Ast::Flags(_) => "flags",
        Ast::Literal(_) => "literal",
        Ast::Dot(_) => "dot",
        Ast::Assertion(_) => "assertion",
        Ast::ClassUnicode(_) => "class_unicode",
        Ast::ClassPerl(_) => "class_perl",
        Ast::ClassBracketed(_) => "class_bracketed",
        Ast::Repetition(_) => "repetition",
        Ast::Group(_) => "group",
        Ast::Alternation(_) => "alternation",
        Ast::Concat(_) => "concat",
    }
    .to_string()
}

fn printer_label(ast: &Ast) -> String {
    let mut dst = String::new();
    regex_syntax::ast::print::Printer::new()
        .print(ast, &mut dst)
        .unwrap();
    dst
}

fn label(ast: &Ast) -> String {
    match ast {
        Ast::Empty(_) => "∅".to_string(),
        Ast::Dot(_) => ".".to_string(),
        Ast::Literal(lit) => lit.c.to_string(),
        Ast::Assertion(_) => printer_label(ast),
        Ast::ClassPerl(_) => printer_label(ast),
        Ast::ClassUnicode(_) => printer_label(ast),
        Ast::ClassBracketed(_) => printer_label(ast),
        Ast::Flags(_) => printer_label(ast),
        Ast::Repetition(r) => {
            use regex_syntax::ast::RepetitionKind::*;
            let s = match r.op.kind {
                ZeroOrOne => "?".to_string(),
                ZeroOrMore => "*".to_string(),
                OneOrMore => "+".to_string(),
                Range(ref rng) => {
                    use regex_syntax::ast::RepetitionRange::*;
                    match rng {
                        Exactly(n) => format!("{{{n}}}"),
                        AtLeast(n) => format!("{{{n},}}"),
                        Bounded(m, n) => format!("{{{m},{n}}}"),
                    }
                }
            };
            if r.greedy {
                s
            } else {
                format!("{s}?")
            }
        }
        Ast::Group(g) => {
            use regex_syntax::ast::GroupKind::*;
            match &g.kind {
                CaptureIndex(n) => format!("({n})"),
                CaptureName { name, .. } => format!("(?<{}>)", name.name),
                NonCapturing(_) => "(?:)".to_string(),
            }
        }
        Ast::Alternation(_) => "|".to_string(),
        Ast::Concat(_) => "·".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(nodes: &[Node]) -> Vec<&str> {
        nodes.iter().map(|n| n.label.as_str()).collect()
    }

    fn kinds(nodes: &[Node]) -> Vec<&str> {
        nodes.iter().map(|n| n.kind.as_str()).collect()
    }

    fn children_of<'a>(nodes: &'a [Node], label: &str) -> Vec<&'a str> {
        let idx = nodes.iter().position(|n| n.label == label).unwrap();
        nodes[idx]
            .children
            .iter()
            .map(|&c| nodes[c].label.as_str())
            .collect()
    }

    #[test]
    fn literal() {
        let nodes = build("a").unwrap();
        assert_eq!(labels(&nodes), vec!["a"]);
        assert_eq!(kinds(&nodes), vec!["literal"]);
    }

    #[test]
    fn concat() {
        let nodes = build("ab").unwrap();
        assert_eq!(nodes[0].kind, "concat");
        assert_eq!(children_of(&nodes, "·"), vec!["a", "b"]);
    }

    #[test]
    fn alternation() {
        let nodes = build("a|b").unwrap();
        assert_eq!(nodes[0].kind, "alternation");
        assert_eq!(children_of(&nodes, "|"), vec!["a", "b"]);
    }

    #[test]
    fn repetition_greedy() {
        let nodes = build("a+").unwrap();
        assert_eq!(nodes[0].label, "+");
        assert_eq!(children_of(&nodes, "+"), vec!["a"]);
    }

    #[test]
    fn repetition_lazy() {
        let nodes = build("a+?").unwrap();
        assert_eq!(nodes[0].label, "+?");
    }

    #[test]
    fn group_capture() {
        let nodes = build("(a)").unwrap();
        assert_eq!(nodes[0].kind, "group");
        assert_eq!(nodes[0].label, "(1)");
        assert_eq!(children_of(&nodes, "(1)"), vec!["a"]);
    }

    #[test]
    fn nested() {
        let nodes = build("(a|b)+c").unwrap();
        assert_eq!(nodes[0].kind, "concat");
        assert_eq!(nodes[0].children.len(), 2);
    }

    #[test]
    fn invalid_pattern() {
        assert!(build("(a|b[c").is_err());
    }

    #[test]
    fn class_perl() {
        let nodes = build(r"\d+").unwrap();
        assert_eq!(nodes[1].kind, "class_perl");
    }

    // --- more literals ---

    #[test]
    fn literal_space() {
        let n = build(" ").unwrap();
        assert_eq!(n[0].label, " ");
    }
    #[test]
    fn literal_digit() {
        let n = build("5").unwrap();
        assert_eq!(n[0].kind, "literal");
    }
    #[test]
    fn literal_unicode() {
        let n = build("é").unwrap();
        assert_eq!(n[0].kind, "literal");
    }
    #[test]
    fn escaped_dot() {
        let n = build(r"\.").unwrap();
        assert_eq!(n[0].kind, "literal");
    }
    #[test]
    fn escaped_star() {
        let n = build(r"\*").unwrap();
        assert_eq!(n[0].kind, "literal");
    }
    #[test]
    fn hex_literal() {
        let n = build(r"\x61").unwrap();
        assert_eq!(n[0].kind, "literal");
        assert_eq!(n[0].label, "a");
    }
    #[test]
    fn tab_literal() {
        let n = build(r"\t").unwrap();
        assert_eq!(n[0].kind, "literal");
    }
    #[test]
    fn newline_literal() {
        let n = build(r"\n").unwrap();
        assert_eq!(n[0].kind, "literal");
    }

    // --- dot ---

    #[test]
    fn dot_node() {
        let n = build(".").unwrap();
        assert_eq!(n[0].kind, "dot");
        assert_eq!(n[0].label, ".");
    }

    // --- concat ---

    #[test]
    fn concat_three() {
        let n = build("abc").unwrap();
        assert_eq!(n[0].children.len(), 3);
    }
    #[test]
    fn concat_five() {
        let n = build("abcde").unwrap();
        assert_eq!(n[0].children.len(), 5);
    }
    #[test]
    fn concat_order() {
        let n = build("xy").unwrap();
        assert_eq!(children_of(&n, "·"), vec!["x", "y"]);
    }

    // --- alternation ---

    #[test]
    fn alt_three() {
        let n = build("a|b|c").unwrap();
        assert_eq!(n[0].children.len(), 3);
    }
    #[test]
    fn alt_four() {
        let n = build("a|b|c|d").unwrap();
        assert_eq!(n[0].children.len(), 4);
    }
    #[test]
    fn alt_words() {
        let n = build("foo|bar").unwrap();
        assert_eq!(n[0].kind, "alternation");
    }
    #[test]
    fn alt_children_labels() {
        let n = build("x|y").unwrap();
        assert_eq!(children_of(&n, "|"), vec!["x", "y"]);
    }

    // --- repetition ---

    #[test]
    fn star() {
        let n = build("a*").unwrap();
        assert_eq!(n[0].label, "*");
    }
    #[test]
    fn question_mark() {
        let n = build("a?").unwrap();
        assert_eq!(n[0].label, "?");
    }
    #[test]
    fn exact_rep() {
        let n = build("a{3}").unwrap();
        assert_eq!(n[0].label, "{3}");
    }
    #[test]
    fn at_least() {
        let n = build("a{2,}").unwrap();
        assert_eq!(n[0].label, "{2,}");
    }
    #[test]
    fn bounded_rep() {
        let n = build("a{2,5}").unwrap();
        assert_eq!(n[0].label, "{2,5}");
    }
    #[test]
    fn lazy_star() {
        let n = build("a*?").unwrap();
        assert_eq!(n[0].label, "*?");
    }
    #[test]
    fn lazy_question() {
        let n = build("a??").unwrap();
        assert_eq!(n[0].label, "??");
    }
    #[test]
    fn rep_child() {
        let n = build("a*").unwrap();
        assert_eq!(children_of(&n, "*"), vec!["a"]);
    }

    // --- groups ---

    #[test]
    fn named_group() {
        let n = build("(?P<foo>a)").unwrap();
        assert_eq!(n[0].label, "(?<foo>)");
    }
    #[test]
    fn non_capturing() {
        let n = build("(?:a)").unwrap();
        assert_eq!(n[0].kind, "group");
        assert_eq!(n[0].label, "(?:)");
    }
    #[test]
    fn nested_groups() {
        let n = build("((a))").unwrap();
        assert_eq!(kinds(&n).iter().filter(|&&k| k == "group").count(), 2);
    }
    #[test]
    fn group_child() {
        let n = build("(ab)").unwrap();
        let g = n[0].children[0];
        assert_eq!(n[g].kind, "concat");
    }

    // --- classes ---

    #[test]
    fn class_w() {
        let n = build(r"\w").unwrap();
        assert_eq!(n[0].kind, "class_perl");
    }
    #[test]
    fn class_s() {
        let n = build(r"\s").unwrap();
        assert_eq!(n[0].kind, "class_perl");
    }
    #[test]
    fn class_upper_d() {
        let n = build(r"\D").unwrap();
        assert_eq!(n[0].kind, "class_perl");
    }
    #[test]
    fn bracketed() {
        let n = build("[a-z]").unwrap();
        assert_eq!(n[0].kind, "class_bracketed");
        assert_eq!(n[0].label, "[a-z]");
    }
    #[test]
    fn negated_class() {
        let n = build("[^a-z]").unwrap();
        assert_eq!(n[0].kind, "class_bracketed");
    }
    #[test]
    fn unicode_class() {
        let n = build(r"\p{L}").unwrap();
        assert_eq!(n[0].kind, "class_unicode");
    }
    #[test]
    fn unicode_neg_class() {
        let n = build(r"\P{L}").unwrap();
        assert_eq!(n[0].kind, "class_unicode");
    }

    // --- assertions ---

    #[test]
    fn caret() {
        let n = build("^").unwrap();
        assert_eq!(n[0].kind, "assertion");
    }
    #[test]
    fn dollar() {
        let n = build("$").unwrap();
        assert_eq!(n[0].kind, "assertion");
    }
    #[test]
    fn word_boundary() {
        let n = build(r"\b").unwrap();
        assert_eq!(n[0].kind, "assertion");
    }
    #[test]
    fn not_word_boundary() {
        let n = build(r"\B").unwrap();
        assert_eq!(n[0].kind, "assertion");
    }
    #[test]
    fn start_text() {
        let n = build(r"\A").unwrap();
        assert_eq!(n[0].kind, "assertion");
    }
    #[test]
    fn end_text() {
        let n = build(r"\z").unwrap();
        assert_eq!(n[0].kind, "assertion");
    }

    // --- flags ---

    #[test]
    fn flags_node() {
        let n = build("(?i)").unwrap();
        assert_eq!(n[0].kind, "flags");
    }
    #[test]
    fn inline_flag_group() {
        let n = build("(?i:a)").unwrap();
        assert_eq!(n[0].kind, "group");
    }

    // --- invalid ---

    #[test]
    fn invalid_open_paren() {
        assert!(build("(abc").is_err());
    }
    #[test]
    fn invalid_open_bracket() {
        assert!(build("[abc").is_err());
    }
    #[test]
    fn invalid_lone_plus() {
        assert!(build("+").is_err());
    }
    #[test]
    fn invalid_lone_star() {
        assert!(build("*").is_err());
    }

    // --- complex ---

    #[test]
    fn date_pattern() {
        let n = build(r"(?P<year>\d{4})-(?P<month>0[1-9]|1[0-2])-(?P<day>0[1-9]|[12]\d|3[01])")
            .unwrap();
        assert_eq!(n[0].kind, "concat");
        assert!(n.iter().any(|x| x.label == "(?<year>)"));
        assert!(n.iter().any(|x| x.label == "(?<month>)"));
        assert!(n.iter().any(|x| x.label == "(?<day>)"));
    }

    #[test]
    fn email_like() {
        let n = build(r"[a-z]+@[a-z]+\.[a-z]+").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert!(kinds(&n).iter().filter(|&&k| k == "repetition").count() >= 2);
    }

    #[test]
    fn nested_groups_and_reps() {
        let n = build(r"((a|b)+c)*").unwrap();
        assert_eq!(n[0].kind, "repetition");
        assert!(kinds(&n).iter().any(|&k| k == "group"));
    }

    #[test]
    fn ip_like() {
        let n = build(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
        assert_eq!(kinds(&n).iter().filter(|&&k| k == "repetition").count(), 4);
    }

    #[test]
    fn word_boundary_word() {
        let n = build(r"\bword\b").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert_eq!(kinds(&n).iter().filter(|&&k| k == "assertion").count(), 2);
    }

    #[test]
    fn alternation_of_groups() {
        let n = build(r"(foo)|(bar)").unwrap();
        assert_eq!(kinds(&n).iter().filter(|&&k| k == "group").count(), 2);
    }

    #[test]
    fn deeply_nested() {
        let n = build(r"((((a))))").unwrap();
        assert_eq!(kinds(&n).iter().filter(|&&k| k == "group").count(), 4);
    }

    #[test]
    fn non_greedy_complex() {
        let n = build(r"<.+?>").unwrap();
        assert!(n.iter().any(|x| x.label == "+?"));
    }

    #[test]
    fn multiline_anchors() {
        let n = build(r"^foo$").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert_eq!(kinds(&n).iter().filter(|&&k| k == "assertion").count(), 2);
    }
}
