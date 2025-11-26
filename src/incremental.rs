use crate::atom::AtomKind;
use crate::language::{Delimiter, Language};
use crate::lexer::lex;
use crate::token::TokenTree;

/// A "Green" token that knows its text and kind, but not its absolute position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GreenToken {
    pub kind: AtomKind,
    pub text: String,
}

impl GreenToken {
    pub fn width(&self) -> usize {
        self.text.len()
    }
}

/// A "Green" tree node that forms the structure of the code.
/// It is immutable and position-independent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GreenTree {
    Token(GreenToken),
    Delimited {
        delimiter: Delimiter,
        children: Vec<GreenTree>,
    },
    Group(Vec<GreenTree>),
    Empty,
}

impl GreenTree {
    pub fn width(&self) -> usize {
        match self {
            GreenTree::Token(t) => t.width(),
            GreenTree::Delimited { delimiter, children } => {
                delimiter.open.len()
                    + children.iter().map(|c| c.width()).sum::<usize>()
                    + delimiter.close.len()
            }
            GreenTree::Group(children) => children.iter().map(|c| c.width()).sum(),
            GreenTree::Empty => 0,
        }
    }

    /// Reconstructs the full text of this node.
    pub fn text(&self) -> String {
        match self {
            GreenTree::Token(t) => t.text.clone(),
            GreenTree::Delimited { delimiter, children } => {
                let mut s = String::new();
                s.push_str(delimiter.open);
                for child in children {
                    s.push_str(&child.text());
                }
                s.push_str(delimiter.close);
                s
            }
            GreenTree::Group(children) => {
                children.iter().map(|c| c.text()).collect()
            }
            GreenTree::Empty => String::new(),
        }
    }

    /// Converts a legacy `TokenTree` (with absolute offsets) to a `GreenTree`.
    pub fn from_token_tree(tt: &TokenTree) -> Self {
        match tt {
            TokenTree::Token(t) => GreenTree::Token(GreenToken {
                kind: t.kind.clone(),
                text: t.text.clone(),
            }),
            TokenTree::Delimited(d, children, _) => GreenTree::Delimited {
                delimiter: d.clone(),
                children: children.iter().map(Self::from_token_tree).collect(),
            },
            TokenTree::Group(children) => GreenTree::Group(
                children.iter().map(Self::from_token_tree).collect(),
            ),
            TokenTree::Empty => GreenTree::Empty,
            TokenTree::Error(_) => GreenTree::Empty, // TODO: Handle errors better
        }
    }
}

/// Represents a text edit: replacing a range of text with new text.
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub start: usize,
    pub end: usize, // Exclusive
    pub new_text: String,
}

impl TextEdit {
    pub fn apply(&self, original: &str) -> String {
        let mut s = String::with_capacity(original.len() + self.new_text.len());
        s.push_str(&original[..self.start]);
        s.push_str(&self.new_text);
        s.push_str(&original[self.end..]);
        s
    }
}

/// The result of an incremental re-lex operation.
#[derive(Debug)]
pub enum RelexResult {
    /// The edit was successfully handled by re-lexing a sub-tree.
    Success(GreenTree),
    /// The edit could not be isolated (e.g., unbalanced delimiters), requiring a full re-parse.
    Failed,
}

/// Attempts to apply an edit to a GreenTree incrementally.
pub fn incremental_relex(
    root: &GreenTree,
    edit: &TextEdit,
    language: &impl Language,
) -> RelexResult {
    // 1. Find the node covering the edit range.
    // We need to track the current offset as we traverse.
    relex_recursive(root, 0, edit, language)
}

fn relex_recursive(
    node: &GreenTree,
    offset: usize,
    edit: &TextEdit,
    language: &impl Language,
) -> RelexResult {
    let width = node.width();
    let node_end = offset + width;

    // Check if the edit is fully contained within this node
    // Note: We want to find the *deepest* container.
    // If the edit overlaps the boundaries, we can't handle it inside this node (unless it's the root).
    
    // Strict containment: start >= offset && end <= node_end
    // But for Delimited nodes, we only want to re-lex if it's inside the *content*, not touching the delimiters.
    
    match node {
        GreenTree::Delimited { delimiter, children } => {
            let open_len = delimiter.open.len();
            let close_len = delimiter.close.len();
            let content_start = offset + open_len;
            let content_end = node_end - close_len;

            // If edit is strictly inside the delimiters
            if edit.start >= content_start && edit.end <= content_end {
                // Try to find a child that contains it
                let mut current_child_offset = content_start;
                for (i, child) in children.iter().enumerate() {
                    let child_width = child.width();
                    if edit.start >= current_child_offset && edit.end <= current_child_offset + child_width {
                        // Recurse into child
                        match relex_recursive(child, current_child_offset, edit, language) {
                            RelexResult::Success(new_child) => {
                                let mut new_children = children.clone();
                                new_children[i] = new_child;
                                return RelexResult::Success(GreenTree::Delimited {
                                    delimiter: delimiter.clone(),
                                    children: new_children,
                                });
                            }
                            RelexResult::Failed => {
                                // Child failed, but maybe we can re-lex this entire block?
                                break; 
                            }
                        }
                    }
                    current_child_offset += child_width;
                }

                // If we are here, either:
                // 1. Edit spans multiple children (but still inside block)
                // 2. Edit is in the "void" between children (if that's possible? No, we have whitespace atoms usually)
                // 3. Child recursion failed.
                
                // Strategy: Re-lex the content of this block.
                // 1. Reconstruct text of the *content* (inner text).
                let mut inner_text = String::new();
                for child in children {
                    inner_text.push_str(&child.text());
                }
                
                // 2. Apply edit to inner text.
                // We need to map the absolute edit offsets to relative offsets within inner_text.
                let rel_start = edit.start - content_start;
                let rel_end = edit.end - content_start;
                
                let new_inner_text = TextEdit {
                    start: rel_start,
                    end: rel_end,
                    new_text: edit.new_text.clone(),
                }.apply(&inner_text);

                // 3. Lex the new inner text.
                let new_tokens = lex(&new_inner_text, language);
                
                // 4. Convert to GreenTrees
                let new_green_children: Vec<GreenTree> = new_tokens.iter().map(GreenTree::from_token_tree).collect();

                // 5. Verify balance?
                // The `lex` function handles delimiters. If `new_tokens` contains unbalanced delimiters, 
                // `lex` might return error nodes or weird structure.
                // But `lex` is designed to be robust.
                // The critical check is: Did the re-lexing consume the entire string without error?
                // And did it produce a list of trees that fits into this block?
                
                // Actually, `lex` returns `Vec<TokenTree>`. If we put that into the block, it's fine.
                // The only risk is if the user typed "}" inside the block, which would close it early.
                // But `lex` on the *inner* text won't see the outer "}".
                // So `lex` will treat "}" as an error or text depending on language.
                
                return RelexResult::Success(GreenTree::Delimited {
                    delimiter: delimiter.clone(),
                    children: new_green_children,
                });
            }
        }
        GreenTree::Group(children) => {
             let mut current_child_offset = offset;
             for (i, child) in children.iter().enumerate() {
                 let child_width = child.width();
                 if edit.start >= current_child_offset 
                    && edit.end <= current_child_offset + child_width 
                    && let RelexResult::Success(new_child) = relex_recursive(child, current_child_offset, edit, language) 
                 {
                     let mut new_children = children.clone();
                     new_children[i] = new_child;
                     return RelexResult::Success(GreenTree::Group(new_children));
                 }
                 current_child_offset += child_width;
             }
        }
        _ => {}
    }

    // If we couldn't handle it in a child, or this is a leaf (Token), we fail.
    // Why fail? Because if we are at a Token, we can't "re-lex" just the token easily without knowing context.
    // Actually, if we are at the Root, we *must* handle it.
    // But `relex_recursive` is called recursively.
    
    // If we are at the top level (offset 0, width = total), we should fall back to full re-lex if we are the root.
    // But the caller `incremental_relex` calls this.
    
    RelexResult::Failed
}

/// A "Red" node is a transient cursor into the Green Tree that knows its absolute position.
#[derive(Debug, Clone)]
pub struct RedNode<'a> {
    pub green: &'a GreenTree,
    pub offset: usize,
}

impl<'a> RedNode<'a> {
    pub fn new(green: &'a GreenTree, offset: usize) -> Self {
        Self { green, offset }
    }

    pub fn children(&self) -> Vec<RedNode<'a>> {
        let mut children = Vec::new();
        let mut current_offset = match self.green {
            GreenTree::Delimited { delimiter, .. } => self.offset + delimiter.open.len(),
            _ => self.offset,
        };

        match self.green {
            GreenTree::Delimited { children: green_children, .. } |
            GreenTree::Group(green_children) => {
                for child in green_children {
                    children.push(RedNode::new(child, current_offset));
                    current_offset += child.width();
                }
            }
            _ => {}
        }
        children
    }

    /// Finds the deepest node that contains the given offset.
    pub fn find_at_offset(&self, target: usize) -> Option<RedNode<'a>> {
        let width = self.green.width();
        if target < self.offset || target >= self.offset + width {
            return None;
        }

        // Check children
        for child in self.children() {
            if let Some(found) = child.find_at_offset(target) {
                return Some(found);
            }
        }

        // If no child contains it (or we are a leaf), return self
        Some(RedNode { green: self.green, offset: self.offset })
    }
}
