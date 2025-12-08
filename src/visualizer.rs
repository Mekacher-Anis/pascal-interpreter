use crate::ast::{ASTNode, BuiltinNumTypes};
use crate::token::Token;

struct DrawNode {
    #[allow(dead_code)]
    id: usize,
    label: String,
    x: f32,
    y: f32,
    children: Vec<usize>,
}

pub struct Visualizer {
    nodes: Vec<DrawNode>,
    next_id: usize,
    next_x: f32,
    level_height: f32,
    background_color: String,
}

impl Visualizer {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
            next_x: 0.0,
            level_height: 80.0,
            background_color: "#ffffff".to_string(),
        }
    }

    pub fn generate_svg(&mut self, ast: &ASTNode) -> String {
        self.nodes.clear();
        self.next_id = 0;
        self.next_x = 50.0; // Start with some padding

        self.build_tree(ast, 0);

        // Calculate canvas size
        let max_x = self.nodes.iter().map(|n| n.x).fold(0.0f32, f32::max);
        let max_y = self.nodes.iter().map(|n| n.y).fold(0.0f32, f32::max);
        let width = max_x + 100.0;
        let height = max_y + 100.0;

        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            width, height
        ));
        svg.push_str(r#"<style>
            .node { fill: #f0f0f0; stroke: #333; stroke-width: 2; }
            .text { font-family: sans-serif; font-size: 14px; text-anchor: middle; dominant-baseline: middle; fill: #333; }
            .link { stroke: #666; stroke-width: 2; }
        </style>"#);

        // Draw background rect so svg renders with an explicit background color
        svg.push_str(&format!(
            r#"<rect x="0" y="0" width="{}" height="{}" fill="{}" />"#,
            width, height, self.background_color
        ));

        // Draw links
        for node in &self.nodes {
            for &child_id in &node.children {
                let child = &self.nodes[child_id];
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="link" />"#,
                    node.x, node.y, child.x, child.y
                ));
            }
        }

        // Draw nodes
        for node in &self.nodes {
            svg.push_str(&format!(
                r#"<g transform="translate({}, {})">"#,
                node.x, node.y
            ));

            let text_width = node.label.len() as f32 * 9.0;
            let rect_width = text_width.max(50.0);
            let rect_height = 30.0;

            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="5" class="node" />"#,
                -rect_width / 2.0,
                -rect_height / 2.0,
                rect_width,
                rect_height
            ));
            // Escape XML characters in label if necessary
            let safe_label = node.label.replace("<", "&lt;").replace(">", "&gt;");
            svg.push_str(&format!(r#"<text class="text">{}</text>"#, safe_label));
            svg.push_str("</g>");
        }

        svg.push_str("</svg>");
        svg
    }

    fn token_to_string(token: &Token) -> String {
        match token {
            Token::IntegerConst(v) => v.to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Asterisk => "*".to_string(),
            Token::FloatDiv => "/".to_string(),
            Token::LParenthesis => "(".to_string(),
            Token::RParenthesis => ")".to_string(),
            Token::Begin => "BEGIN".to_string(),
            Token::End => "END".to_string(),
            Token::Dot => ".".to_string(),
            Token::Id(s) => s.clone(),
            Token::Assign => ":=".to_string(),
            Token::Semi => ";".to_string(),
            Token::Eof => "EOF".to_string(),
            Token::Program => "PROGRAM".to_string(),
            Token::Var => "var".to_string(),
            Token::Colon => ":".to_string(),
            Token::Comma => ",".to_string(),
            Token::Integer => "INTEGER".to_string(),
            Token::IntegerDiv => "DIV".to_string(),
            Token::RealConst(v) => v.to_string(),
            Token::Real => "REAL".to_string(),
        }
    }

    fn build_tree(&mut self, node: &ASTNode, depth: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        // Placeholder
        self.nodes.push(DrawNode {
            id,
            label: String::new(),
            x: 0.0,
            y: (depth as f32) * self.level_height + 40.0,
            children: Vec::new(),
        });

        let (label, children_indices) = match node {
            ASTNode::Compound { children } => {
                let mut indices = Vec::new();
                for child in children {
                    indices.push(self.build_tree(child, depth + 1));
                }
                ("Compound".to_string(), indices)
            }
            ASTNode::Assign { left, right, token } => {
                let l = self.build_tree(left, depth + 1);
                let r = self.build_tree(right, depth + 1);
                (
                    format!("Assign({})", Self::token_to_string(token)),
                    vec![l, r],
                )
            }
            ASTNode::Var { name: value } => (format!("Var({})", value), vec![]),
            ASTNode::NoOp => ("NoOp".to_string(), vec![]),
            ASTNode::UnaryOpNode { expr, token } => {
                let e = self.build_tree(expr, depth + 1);
                (format!("Unary({})", Self::token_to_string(token)), vec![e])
            }
            ASTNode::BinOpNode { left, right, op } => {
                let l = self.build_tree(left, depth + 1);
                let r = self.build_tree(right, depth + 1);
                (format!("BinOp({})", Self::token_to_string(op)), vec![l, r])
            }
            ASTNode::NumNode { value, .. } => {
                let value_str = match value {
                    BuiltinNumTypes::I32(i) => i.to_string(),
                    BuiltinNumTypes::F32(f) => f.to_string(),
                };
                (format!("Num({})", value_str), vec![])
            }
            ASTNode::Program { name, block } => {
                let child = self.build_tree(block, depth + 1);
                (format!("Program({})", name), vec![child])
            }
            ASTNode::Block {
                declarations,
                compound_statement,
            } => {
                let mut indices = Vec::new();
                for decl in declarations {
                    indices.push(self.build_tree(decl, depth + 1));
                }
                indices.push(self.build_tree(compound_statement, depth + 1));
                ("Block".to_string(), indices)
            }
            ASTNode::VarDecl {
                var_node,
                type_node,
            } => {
                let v = self.build_tree(var_node, depth + 1);
                let t = self.build_tree(type_node, depth + 1);
                ("VarDecl".to_string(), vec![v, t])
            }
            ASTNode::Type { value, .. } => (format!("Type({})", value), vec![]),
        };

        let my_x = if children_indices.is_empty() {
            let x = self.next_x;
            self.next_x += 80.0; // Spacing
            x
        } else {
            let first_child = &self.nodes[children_indices[0]];
            let last_child = &self.nodes[*children_indices.last().unwrap()];
            (first_child.x + last_child.x) / 2.0
        };

        let node_ref = &mut self.nodes[id];
        node_ref.label = label;
        node_ref.children = children_indices;
        node_ref.x = my_x;

        id
    }
}
