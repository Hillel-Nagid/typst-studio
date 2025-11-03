//! Syntax highlighting using Typst's own parser
//!
//! Phase 3.3: Syntax Highlighting

use typst_syntax::{ parse, SyntaxNode, SyntaxKind };
use std::sync::Arc;
use gpui::rgb;

/// Syntax highlighter using Typst's parser
pub struct SyntaxHighlighter {
    // Typst parser is stateless, no need to store state
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse and highlight Typst text
    pub fn highlight(&self, text: &str) -> Arc<HighlightResult> {
        let root = parse(text);
        let tokens = Self::extract_tokens(&root, text);

        Arc::new(HighlightResult {
            root,
            tokens,
        })
    }

    /// Extract tokens from the syntax tree for highlighting
    /// Uses iterative approach to avoid stack overflow on deep trees
    fn extract_tokens(node: &SyntaxNode, text: &str) -> Vec<HighlightToken> {
        let mut tokens = Vec::new();

        // Use iterative traversal with position tracking
        // Stack holds: (node, current_offset)
        let mut stack: Vec<(SyntaxNode, usize)> = vec![(node.clone(), 0)];

        while let Some((current, current_offset)) = stack.pop() {
            let kind = current.kind();
            let node_text = current.text();
            let node_len = node_text.len();

            // Check if this is a leaf node (no children)
            let children: Vec<_> = current.children().collect();

            if children.is_empty() && node_len > 0 {
                // Leaf node - extract token
                if let Some(token_type) = Self::syntax_kind_to_token_type(kind) {
                    let start = current_offset;
                    let end = start + node_len;

                    // Only add tokens for valid byte ranges
                    if start < text.len() && end <= text.len() && start < end {
                        let color = Self::token_type_to_color(token_type);
                        tokens.push(HighlightToken {
                            start,
                            end,
                            token_type,
                            color,
                        });
                    }
                }
            } else {
                // Non-leaf: push children in reverse order (so they're popped in correct order)
                // Calculate offset for each child
                let mut child_offset = current_offset;
                let children_with_offsets: Vec<_> = children
                    .into_iter()
                    .map(|child| {
                        let offset = child_offset;
                        child_offset += child.text().len();
                        (child, offset)
                    })
                    .collect();

                // Push in reverse order for correct traversal
                for (child, offset) in children_with_offsets.into_iter().rev() {
                    stack.push((child.clone(), offset));
                }
            }
        }

        // Sort tokens by start position
        tokens.sort_by_key(|t| t.start);
        tokens
    }

    /// Map Typst SyntaxKind to our TokenType
    fn syntax_kind_to_token_type(kind: SyntaxKind) -> Option<TokenType> {
        match kind {
            // Keywords
            | SyntaxKind::Let
            | SyntaxKind::Set
            | SyntaxKind::Show
            | SyntaxKind::If
            | SyntaxKind::Else
            | SyntaxKind::For
            | SyntaxKind::While
            | SyntaxKind::Break
            | SyntaxKind::Continue
            | SyntaxKind::Return
            | SyntaxKind::Import
            | SyntaxKind::Include
            | SyntaxKind::As
            | SyntaxKind::In
            | SyntaxKind::Not
            | SyntaxKind::And
            | SyntaxKind::Or => Some(TokenType::Keyword),

            // Functions and identifiers
            SyntaxKind::FuncCall => Some(TokenType::Function),
            SyntaxKind::Ident => Some(TokenType::Variable),

            // Literals
            SyntaxKind::Str | SyntaxKind::RawLang | SyntaxKind::RawTrimmed =>
                Some(TokenType::String),
            SyntaxKind::Int | SyntaxKind::Float | SyntaxKind::Bool => Some(TokenType::Constant),

            // Comments
            SyntaxKind::LineComment | SyntaxKind::BlockComment => Some(TokenType::Comment),

            // Operators
            | SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Star
            | SyntaxKind::Slash
            | SyntaxKind::Eq
            | SyntaxKind::EqEq
            | SyntaxKind::ExclEq
            | SyntaxKind::Lt
            | SyntaxKind::LtEq
            | SyntaxKind::Gt
            | SyntaxKind::GtEq => Some(TokenType::Operator),

            // Math mode
            SyntaxKind::Math | SyntaxKind::MathAlignPoint | SyntaxKind::MathIdent =>
                Some(TokenType::Math),

            // Markup
            SyntaxKind::Markup | SyntaxKind::Strong | SyntaxKind::Emph | SyntaxKind::Heading =>
                Some(TokenType::Markup),

            // Labels and references
            SyntaxKind::Label => Some(TokenType::Label),
            SyntaxKind::Ref => Some(TokenType::Reference),

            _ => None,
        }
    }

    /// Map TokenType to RGB color
    fn token_type_to_color(token_type: TokenType) -> gpui::Rgba {
        match token_type {
            TokenType::Keyword => rgb(0x569cd6), // Blue
            TokenType::Function => rgb(0xdcdcaa), // Yellow
            TokenType::Variable => rgb(0x9cdcfe), // Light blue
            TokenType::Constant => rgb(0xb5cea8), // Green
            TokenType::String => rgb(0xce9178), // Orange
            TokenType::Comment => rgb(0x6a9955), // Green (muted)
            TokenType::Type => rgb(0x4ec9b0), // Teal
            TokenType::Operator => rgb(0xd4d4d4), // Gray
            TokenType::Markup => rgb(0xd7ba7d), // Tan
            TokenType::Math => rgb(0xf8f8f2), // White
            TokenType::Label => rgb(0xc586c0), // Purple
            TokenType::Reference => rgb(0xf8f8f2), // White
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Highlight result with token information
pub struct HighlightResult {
    pub root: SyntaxNode,
    pub tokens: Vec<HighlightToken>,
}

/// A highlighted token
#[derive(Clone)]
pub struct HighlightToken {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
    pub color: gpui::Rgba,
}

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Function,
    Variable,
    Constant,
    String,
    Comment,
    Type,
    Operator,
    Markup,
    Math,
    Label,
    Reference,
}
