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

use crate::node::{Node, NodeVec, TreeBuilder};
use regex_syntax::hir::{visit, Hir, HirKind, Visitor};

impl Visitor for TreeBuilder {
    type Output = NodeVec;
    type Err = ();

    fn finish(self) -> Result<Self::Output, Self::Err> {
        Ok(self.nodes)
    }

    fn visit_pre(&mut self, hir: &Hir) -> Result<(), Self::Err> {
        let parent = self.stack.last().copied();
        let index = self.nodes.len();

        self.nodes.push(Node {
            label: label(hir),
            kind: kind(hir.kind()),
            parent,
            children: Vec::new(),
        });

        if let Some(p) = parent {
            self.nodes[p].children.push(index);
        }

        self.stack.push(index);
        Ok(())
    }

    fn visit_post(&mut self, _hir: &Hir) -> Result<(), Self::Err> {
        self.stack.pop();
        Ok(())
    }
}

pub fn build(pattern: &str) -> Result<NodeVec, String> {
    let hir = regex_syntax::Parser::new()
        .parse(pattern)
        .map_err(|e| e.to_string())?;
    visit(&hir, TreeBuilder::new()).map_err(|_| "traversal failed".to_string())
}

fn kind(hir: &HirKind) -> String {
    match hir {
        HirKind::Empty => "empty",
        HirKind::Literal(_) => "literal",
        HirKind::Class(_) => "class",
        HirKind::Look(_) => "look",
        HirKind::Repetition(_) => "repetition",
        HirKind::Capture(_) => "capture",
        HirKind::Concat(_) => "concat",
        HirKind::Alternation(_) => "alternation",
    }
    .to_string()
}

fn printer_label(hir: &Hir) -> String {
    let mut dst = String::new();
    regex_syntax::hir::print::Printer::new()
        .print(hir, &mut dst)
        .unwrap();
    dst
}

fn label(hir: &Hir) -> String {
    match hir.kind() {
        HirKind::Empty => "∅".to_string(),
        HirKind::Literal(lit) => String::from_utf8_lossy(&lit.0).to_string(),
        HirKind::Class(_) => "[…]".to_string(),
        HirKind::Look(_) => printer_label(hir),
        HirKind::Repetition(r) => {
            let s = match (r.min, r.max) {
                (0, Some(1)) => "?".to_string(),
                (0, None) => "*".to_string(),
                (1, None) => "+".to_string(),
                (m, Some(n)) => format!("{{{},{}}}", m, n),
                (m, None) => format!("{{{},}}", m),
            };
            if r.greedy {
                s
            } else {
                format!("{s}?")
            }
        }
        HirKind::Capture(c) => match &c.name {
            Some(name) => format!("({})", name),
            None => format!("({})", c.index),
        },
        HirKind::Concat(_) => "·".to_string(),
        HirKind::Alternation(_) => "|".to_string(),
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

    fn node_count(nodes: &[Node], kind: &str) -> usize {
        nodes.iter().filter(|n| n.kind == kind).count()
    }

    fn has_label(nodes: &[Node], label: &str) -> bool {
        nodes.iter().any(|n| n.label == label)
    }

    // --- basic ---

    #[test]
    fn literal_a() {
        let n = build("a").unwrap();
        assert_eq!(kinds(&n), vec!["literal"]);
        assert_eq!(labels(&n), vec!["a"]);
    }
    #[test]
    fn literal_digit_char() {
        let n = build("5").unwrap();
        assert_eq!(n[0].label, "5");
    }
    #[test]
    fn dot() {
        let n = build(".").unwrap();
        assert_eq!(n[0].kind, "class");
    }
    #[test]
    fn empty_group() {
        let n = build("(?:)").unwrap();
        assert_eq!(n[0].kind, "empty");
    }
    #[test]
    fn invalid() {
        assert!(build("(abc").is_err());
    }
    #[test]
    fn invalid_bracket() {
        assert!(build("[abc").is_err());
    }

    // --- concat ---

    #[test]
    fn concat_two() {
        let n = build(r"a\d").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert_eq!(n[0].children.len(), 2);
    }
    #[test]
    fn concat_three() {
        let n = build(r"a\db").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert_eq!(n[0].children.len(), 3);
    }
    #[test]
    fn concat_children_labels() {
        let n = build(r"a\d").unwrap();
        assert_eq!(children_of(&n, "·"), vec!["a", "[…]"]);
    }
    #[test]
    fn concat_five() {
        let n = build(r"a\db\sc").unwrap();
        assert_eq!(n[0].children.len(), 5);
    }

    // --- alternation ---

    #[test]
    fn alt_two() {
        let n = build("a|b").unwrap();
        assert_eq!(n[0].kind, "class");
    }
    #[test]
    fn alt_three_chars() {
        let n = build("a|b|c").unwrap();
        assert_eq!(n[0].kind, "class");
    }
    #[test]
    fn alt_words() {
        let n = build("foo|bar").unwrap();
        assert_eq!(n[0].kind, "alternation");
    }
    #[test]
    fn alt_children() {
        let n = build("foo|bar").unwrap();
        assert_eq!(n[0].children.len(), 2);
    }
    #[test]
    fn alt_nested() {
        let n = build("a|b|c|d").unwrap();
        assert_eq!(n[0].kind, "class");
    }

    // --- repetition ---

    #[test]
    fn star() {
        let n = build("a*").unwrap();
        assert_eq!(n[0].label, "*");
        assert_eq!(n[0].kind, "repetition");
    }
    #[test]
    fn plus() {
        let n = build("a+").unwrap();
        assert_eq!(n[0].label, "+");
    }
    #[test]
    fn question() {
        let n = build("a?").unwrap();
        assert_eq!(n[0].label, "?");
    }
    #[test]
    fn exact() {
        let n = build("a{3}").unwrap();
        assert_eq!(n[0].label, "{3,3}");
    }
    #[test]
    fn at_least() {
        let n = build("a{2,}").unwrap();
        assert_eq!(n[0].label, "{2,}");
    }
    #[test]
    fn bounded() {
        let n = build("a{2,5}").unwrap();
        assert_eq!(n[0].label, "{2,5}");
    }
    #[test]
    fn lazy_star() {
        let n = build("a*?").unwrap();
        assert_eq!(n[0].label, "*?");
    }
    #[test]
    fn lazy_plus() {
        let n = build("a+?").unwrap();
        assert_eq!(n[0].label, "+?");
    }
    #[test]
    fn lazy_question() {
        let n = build("a??").unwrap();
        assert_eq!(n[0].label, "??");
    }
    #[test]
    fn rep_has_one_child() {
        let n = build("a+").unwrap();
        assert_eq!(n[0].children.len(), 1);
    }

    // --- capture ---

    #[test]
    fn capture_index() {
        let n = build("(a)").unwrap();
        assert_eq!(n[0].kind, "capture");
        assert_eq!(n[0].label, "(1)");
    }
    #[test]
    fn capture_child() {
        let n = build("(a)").unwrap();
        assert_eq!(children_of(&n, "(1)"), vec!["a"]);
    }
    #[test]
    fn named_capture() {
        let n = build("(?P<foo>a)").unwrap();
        assert_eq!(n[0].label, "(foo)");
    }
    #[test]
    fn nested_capture() {
        let n = build("((a))").unwrap();
        assert_eq!(node_count(&n, "capture"), 2);
    }
    #[test]
    fn two_captures() {
        let n = build("(a)(b)").unwrap();
        assert_eq!(node_count(&n, "capture"), 2);
    }

    // --- classes ---

    #[test]
    fn class_d() {
        let n = build(r"\d").unwrap();
        assert_eq!(n[0].kind, "class");
    }
    #[test]
    fn class_w() {
        let n = build(r"\w").unwrap();
        assert_eq!(n[0].kind, "class");
    }
    #[test]
    fn class_s() {
        let n = build(r"\s").unwrap();
        assert_eq!(n[0].kind, "class");
    }
    #[test]
    fn bracketed_class() {
        let n = build("[a-z]").unwrap();
        assert_eq!(n[0].kind, "class");
        assert_eq!(n[0].label, "[…]");
    }
    #[test]
    fn negated_class() {
        let n = build("[^a-z]").unwrap();
        assert_eq!(n[0].kind, "class");
    }
    #[test]
    fn class_digit_plus() {
        let n = build(r"\d+").unwrap();
        assert_eq!(n[0].kind, "repetition");
        assert_eq!(n[1].kind, "class");
    }

    // --- look ---

    #[test]
    fn look_start() {
        let n = build(r"\A").unwrap();
        assert_eq!(n[0].kind, "look");
        assert_eq!(n[0].label, "\\A");
    }
    #[test]
    fn look_end() {
        let n = build(r"\z").unwrap();
        assert_eq!(n[0].kind, "look");
        assert_eq!(n[0].label, "\\z");
    }
    #[test]
    fn look_startlf() {
        let n = build("(?m)^").unwrap();
        assert_eq!(n[0].kind, "look");
    }
    #[test]
    fn look_endlf() {
        let n = build("(?m)$").unwrap();
        assert_eq!(n[0].kind, "look");
    }
    #[test]
    fn word_boundary() {
        let n = build(r"\b").unwrap();
        assert_eq!(n[0].kind, "look");
    }

    // --- complex ---

    #[test]
    fn date_pattern() {
        let n = build(r"(?P<year>\d{4})-(?P<month>0[1-9]|1[0-2])-(?P<day>0[1-9]|[12]\d|3[01])")
            .unwrap();
        assert_eq!(n[0].kind, "concat");
        assert!(has_label(&n, "(year)"));
        assert!(has_label(&n, "(month)"));
        assert!(has_label(&n, "(day)"));
    }

    #[test]
    fn email_like() {
        let n = build(r"[a-z]+@[a-z]+\.[a-z]+").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert!(node_count(&n, "repetition") >= 2);
    }

    #[test]
    fn nested_groups_and_reps() {
        let n = build(r"((a|b)+c)*").unwrap();
        assert_eq!(n[0].kind, "repetition");
        assert!(node_count(&n, "capture") >= 1);
    }

    #[test]
    fn alternation_of_groups() {
        let n = build(r"(foo)|(bar)").unwrap();
        assert_eq!(node_count(&n, "capture"), 2);
    }

    #[test]
    fn word_boundary_word() {
        let n = build(r"\bword\b").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert_eq!(node_count(&n, "look"), 2);
    }

    #[test]
    fn ip_like() {
        let n = build(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
        assert_eq!(node_count(&n, "repetition"), 4);
    }

    #[test]
    fn repeated_group() {
        let n = build(r"(ab)+").unwrap();
        assert_eq!(n[0].kind, "repetition");
        assert_eq!(n[1].kind, "capture");
    }

    #[test]
    fn optional_group() {
        let n = build(r"(abc)?").unwrap();
        assert_eq!(n[0].kind, "repetition");
        assert_eq!(n[0].label, "?");
    }

    #[test]
    fn unicode_class() {
        let n = build(r"\p{L}+").unwrap();
        assert_eq!(n[0].kind, "repetition");
        assert_eq!(n[1].kind, "class");
    }

    #[test]
    fn multiline_anchors() {
        let n = build(r"(?m)^foo$").unwrap();
        assert_eq!(n[0].kind, "concat");
        assert_eq!(node_count(&n, "look"), 2);
    }

    #[test]
    fn hex_literal() {
        let n = build(r"\x61").unwrap();
        assert_eq!(n[0].kind, "literal");
        assert_eq!(n[0].label, "a");
    }

    #[test]
    fn non_greedy_group() {
        let n = build(r"(a+?)").unwrap();
        assert!(has_label(&n, "+?"));
    }

    #[test]
    fn deeply_nested() {
        let n = build(r"((((a))))").unwrap();
        assert_eq!(node_count(&n, "capture"), 4);
    }

    #[test]
    fn alternation_with_empty() {
        let n = build(r"a|").unwrap();
        assert!(n[0].children.len() >= 2);
    }
}
