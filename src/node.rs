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

pub struct Node {
    pub label: String,
    pub kind: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

pub type NodeVec = Vec<Node>;

pub struct TreeBuilder {
    pub nodes: NodeVec,
    pub stack: Vec<usize>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        TreeBuilder {
            nodes: Vec::new(),
            stack: Vec::new(),
        }
    }
}
