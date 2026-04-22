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

mod ast;
mod hir;
mod layout;
mod node;
mod svg;
mod utils;

use layout::LayoutConfig;
use svg::SvgConfig;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn visualize(
    pattern: &str,
    leaf_spacing: f64,
    level_height: f64,
    rect_width: f64,
    rect_height: f64,
    padding: f64,
    edge_width: f64,
    label_font_size: f64,
    corner_radius: f64,
) -> String {
    utils::set_panic_hook();

    let layout_cfg = LayoutConfig {
        leaf_spacing,
        level_height,
    };
    let svg_cfg = SvgConfig {
        rect_width,
        rect_height,
        padding,
        edge_width,
        label_font_size,
        corner_radius,
    };

    let ast_nodes = match ast::build(pattern) {
        Ok(n) => n,
        Err(e) => return format!("AST error: {e}"),
    };
    let hir_nodes = match hir::build(pattern) {
        Ok(n) => n,
        Err(e) => return format!("HIR error: {e}"),
    };

    let ast_layout = layout::layout(&ast_nodes, &layout_cfg);
    let hir_layout = layout::layout(&hir_nodes, &layout_cfg);

    let ast_svg = svg::render(&ast_nodes, &ast_layout, &svg_cfg);
    let hir_svg = svg::render(&hir_nodes, &hir_layout, &svg_cfg);

    format!("{ast_svg}||{hir_svg}")
}
