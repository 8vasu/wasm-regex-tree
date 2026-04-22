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

pub struct LayoutConfig {
    pub leaf_spacing: f64,
    pub level_height: f64,
}

pub struct LayoutNode {
    pub x: f64,
    pub y: f64,
}

pub fn layout(nodes: &[Node], cfg: &LayoutConfig) -> Vec<LayoutNode> {
    let n = nodes.len();
    if n == 0 {
        return vec![];
    }

    let root = match nodes.iter().position(|n| n.parent.is_none()) {
        Some(r) => r,
        None => return vec![],
    };

    let mut width = vec![0.0f64; n];
    let mut x = vec![0.0f64; n];
    let mut depth = vec![0usize; n];

    for &v in &postorder(nodes, root) {
        width[v] = if nodes[v].children.is_empty() {
            cfg.leaf_spacing
        } else {
            nodes[v].children.iter().map(|&c| width[c]).sum()
        };
    }

    for &v in &preorder(nodes, root) {
        if let Some(p) = nodes[v].parent {
            depth[v] = depth[p] + 1;
        }
        let mut offset = x[v] - width[v] / 2.0;
        for &c in &nodes[v].children {
            x[c] = offset + width[c] / 2.0;
            offset += width[c];
        }
    }

    (0..n)
        .map(|v| LayoutNode {
            x: x[v],
            y: depth[v] as f64 * cfg.level_height,
        })
        .collect()
}

fn postorder(nodes: &[Node], root: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    let mut visited = vec![false; nodes.len()];
    while let Some(&v) = stack.last() {
        let unvisited = nodes[v].children.iter().find(|&&c| !visited[c]).copied();
        if let Some(c) = unvisited {
            stack.push(c);
        } else {
            visited[v] = true;
            result.push(v);
            stack.pop();
        }
    }
    result
}

fn preorder(nodes: &[Node], root: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(v) = stack.pop() {
        result.push(v);
        for &c in nodes[v].children.iter().rev() {
            stack.push(c);
        }
    }
    result
}
