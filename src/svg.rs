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

use crate::layout::LayoutNode;
use crate::node::Node;

pub struct SvgConfig {
    pub rect_width: f64,
    pub rect_height: f64,
    pub padding: f64,
    pub edge_width: f64,
    pub label_font_size: f64,
    pub corner_radius: f64,
}

fn kind_color(kind: &str) -> &'static str {
    match kind {
        "literal" => "#d4eac8",
        "concat" => "#cfe0f5",
        "alternation" => "#f5e0cf",
        "repetition" => "#f5cfcf",
        "group" | "capture" => "#e8d4f0",
        "class_perl" | "class_unicode" | "class_bracketed" | "class" => "#fdf0c2",
        "assertion" | "look" => "#d4f0ee",
        _ => "#e8e8e8",
    }
}

pub fn render(nodes: &[Node], layout: &[LayoutNode], cfg: &SvgConfig) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let min_x = layout.iter().map(|l| l.x).fold(f64::INFINITY, f64::min);
    let max_x = layout.iter().map(|l| l.x).fold(f64::NEG_INFINITY, f64::max);
    let max_y = layout.iter().map(|l| l.y).fold(f64::NEG_INFINITY, f64::max);

    let offset_x = -min_x + cfg.padding;
    let width = max_x - min_x + cfg.padding * 2.0 + cfg.rect_width;
    let height = max_y + cfg.padding * 2.0 + cfg.rect_height;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0}" height="{height:.0}">"#
    );

    for (i, node) in nodes.iter().enumerate() {
        if let Some(p) = node.parent {
            let x1 = layout[p].x + offset_x + cfg.rect_width / 2.0;
            let y1 = layout[p].y + cfg.padding + cfg.rect_height;
            let x2 = layout[i].x + offset_x + cfg.rect_width / 2.0;
            let y2 = layout[i].y + cfg.padding;
            svg.push_str(&format!(
                r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="#bbb" stroke-width="{:.1}"/>"##,
                cfg.edge_width
            ));
        }
    }

    for (i, node) in nodes.iter().enumerate() {
        let x = layout[i].x + offset_x;
        let y = layout[i].y + cfg.padding;
        let color = kind_color(&node.kind);

        svg.push_str(&format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{:.1}" height="{:.1}" rx="{:.1}" fill="{color}"/>"#,
            cfg.rect_width, cfg.rect_height, cfg.corner_radius
        ));
        svg.push_str(&format!(
            r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="monospace" font-size="{:.0}" fill="#333">{}</text>"##,
            x + cfg.rect_width / 2.0,
            y + cfg.rect_height / 2.0,
            cfg.label_font_size,
            escape_xml(&node.label)
        ));
    }

    svg.push_str("</svg>");
    svg
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
