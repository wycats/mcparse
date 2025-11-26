use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mcparse::{
    atom::{Atom, AtomKind},
    define_atom, define_language,
    highlighter::{HighlightStyle, Highlighter},
    language::{Delimiter, Language},
    lexer::lex,
    shape::{CompletionItem, MatchContext, MatchResult, Matcher, Shape, seq, term},
    token::{Cursor, SourceLocation, Token, TokenStream, TokenTree},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::{error::Error, io};

// --- Highlighter for Ratatui ---

#[derive(Debug)]
struct RatatuiHighlighter {
    spans: Vec<Span<'static>>,
}

impl RatatuiHighlighter {
    fn new() -> Self {
        Self { spans: Vec::new() }
    }

    fn into_lines(self) -> Vec<Line<'static>> {
        // Split spans by newline characters in their content
        let mut lines = Vec::new();
        let mut current_line_spans = Vec::new();

        for span in self.spans {
            let content = span.content;
            let style = span.style;
            let parts: Vec<&str> = content.split('\n').collect();

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    lines.push(Line::from(current_line_spans));
                    current_line_spans = Vec::new();
                }
                if !part.is_empty() {
                    current_line_spans.push(Span::styled(part.to_string(), style));
                }
            }
        }
        lines.push(Line::from(current_line_spans));
        lines
    }
}

impl Highlighter for RatatuiHighlighter {
    fn highlight(&mut self, token: &Token, style: HighlightStyle) {
        let color = match style {
            HighlightStyle::Keyword => Color::Blue,
            HighlightStyle::String => Color::Green,
            HighlightStyle::Number => Color::Yellow,
            HighlightStyle::Comment => Color::DarkGray,
            HighlightStyle::Operator => Color::Red,
            HighlightStyle::Punctuation => Color::White,
            HighlightStyle::Variable => Color::Cyan,
            HighlightStyle::Function => Color::Magenta,
            HighlightStyle::None => Color::Reset,
        };

        // Handle Unknown tokens specially if needed, though they might come in as None style
        let final_color = if let AtomKind::Other(ref s) = token.kind {
            if s == "Unknown" { Color::Red } else { color }
        } else {
            color
        };

        self.spans.push(Span::styled(
            token.text.clone(),
            Style::default().fg(final_color),
        ));
    }
}

// --- Language Definition (Miniscript) ---

define_atom! {
    struct Whitespace;
    kind = AtomKind::Whitespace;
    parse(input) {
        let mut len = 0;
        for c in input.rest.chars() {
            if c.is_whitespace() {
                len += c.len_utf8();
            } else {
                break;
            }
        }
        if len > 0 {
            Some((
                Token {
                    kind: AtomKind::Whitespace,
                    text: input.rest[..len].to_string(),
                    location: SourceLocation {
                        span: (input.offset, len).into(),
                    },
                    atom_index: None,
                    binding: None,
                },
                input.advance(len),
            ))
        } else {
            None
        }
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::None);
    }
}

#[derive(Debug)]
struct Punctuation(String);
impl Atom for Punctuation {
    fn kind(&self) -> AtomKind {
        AtomKind::Operator
    }
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        if input.rest.starts_with(&self.0) {
            Some((
                Token {
                    kind: AtomKind::Operator,
                    text: self.0.clone(),
                    location: SourceLocation {
                        span: (input.offset, self.0.len()).into(),
                    },
                    atom_index: None,
                    binding: None,
                },
                input.advance(self.0.len()),
            ))
        } else {
            None
        }
    }
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Operator);
    }
}

define_atom! {
    struct Identifier;
    kind = AtomKind::Identifier;
    parse(input) {
        let mut chars = input.rest.chars();
        if let Some(c) = chars.next() {
            if c.is_alphabetic() || c == '_' {
                let mut len = c.len_utf8();
                for c in chars {
                    if c.is_alphanumeric() || c == '_' {
                        len += c.len_utf8();
                    } else {
                        break;
                    }
                }
                return Some((
                    Token {
                        kind: AtomKind::Identifier,
                        text: input.rest[..len].to_string(),
                        location: SourceLocation {
                            span: (input.offset, len).into(),
                        },
                        atom_index: None,
                        binding: None,
                    },
                    input.advance(len),
                ));
            }
        }
        None
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::Variable);
    }
}

define_atom! {
    struct NumberLiteral;
    kind = AtomKind::Number;
    parse(input) {
        let mut len = 0;
        for c in input.rest.chars() {
            if c.is_ascii_digit() {
                len += c.len_utf8();
            } else {
                break;
            }
        }
        if len > 0 {
            Some((
                Token {
                    kind: AtomKind::Number,
                    text: input.rest[..len].to_string(),
                    location: SourceLocation {
                        span: (input.offset, len).into(),
                    },
                    atom_index: None,
                    binding: None,
                },
                input.advance(len),
            ))
        } else {
            None
        }
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::Number);
    }
}

define_language! {
    struct MiniScriptLang;
    atoms = [
        Whitespace,
        Punctuation("=".into()),
        Punctuation(";".into()),
        Punctuation("(".into()),
        Punctuation(")".into()),
        Punctuation("{".into()),
        Punctuation("}".into()),
        Identifier,
        NumberLiteral
    ];
    delimiters = [
        Delimiter {
            kind: "brace",
            open: "{",
            close: "}",
        },
        Delimiter {
            kind: "paren",
            open: "(",
            close: ")",
        },
    ];
}

// --- Matchers ---

#[derive(Debug, Clone)]
struct AnyIdentifier;
impl Matcher for AnyIdentifier {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Token(token) => matches!(token.kind, AtomKind::Identifier),
            _ => false,
        }
    }

    fn describe(&self) -> String {
        "Identifier".to_string()
    }
}

// --- Shape ---

#[derive(Clone, Copy, Debug)]
struct MiniScriptShape;

impl Shape for MiniScriptShape {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        // let <ident> = <number>
        let let_stmt = seq(
            term("let"),
            seq(term(AnyIdentifier), seq(term("="), term(AtomKind::Number))),
        );

        // For now, just match one statement
        let_stmt.match_shape(stream, context)
    }

    fn complete<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
        cursor: usize,
    ) -> Vec<CompletionItem> {
        // let <ident> = <number>
        let let_stmt = seq(
            term("let"),
            seq(term(AnyIdentifier), seq(term("="), term(AtomKind::Number))),
        );

        let_stmt.complete(stream, context, cursor)
    }
}

// --- App Logic ---

struct App {
    input: String,
    cursor_pos: usize,
    completions: Vec<CompletionItem>,
    completion_state: ListState,
    show_completions: bool,
    lang: MiniScriptLang,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            completions: vec![],
            completion_state: ListState::default(),
            show_completions: false,
            lang: MiniScriptLang::new(),
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.update_completions();
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    self.update_completions();
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.update_completions();
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                    self.update_completions();
                }
            }
            KeyCode::Tab => {
                if !self.show_completions {
                    self.show_completions = true;
                    self.update_completions();
                } else if !self.completions.is_empty() {
                    // Cycle selection
                    let i = match self.completion_state.selected() {
                        Some(i) => {
                            if i >= self.completions.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.completion_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                if self.show_completions {
                    if let Some(i) = self.completion_state.selected() {
                        if let Some(item) = self.completions.get(i).cloned() {
                            self.apply_completion(&item);
                        }
                    }
                    self.show_completions = false;
                } else {
                    self.input.insert(self.cursor_pos, '\n');
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Esc => {
                self.show_completions = false;
            }
            _ => {}
        }
    }

    fn update_completions(&mut self) {
        if !self.show_completions {
            return;
        }

        let trees = lex(&self.input, &self.lang);
        let stream = TokenStream::new(&trees);
        use mcparse::shape::NoOpMatchContext;
        let mut context = NoOpMatchContext;

        self.completions = MiniScriptShape.complete(stream, &mut context, self.cursor_pos);
        if self.completions.is_empty() {
            self.completion_state.select(None);
        } else {
            self.completion_state.select(Some(0));
        }
    }

    fn apply_completion(&mut self, item: &CompletionItem) {
        // Remove the part of the token that was already typed
        if item.delete_backwards > 0 {
            let start = self.cursor_pos.saturating_sub(item.delete_backwards);
            self.input.replace_range(start..self.cursor_pos, "");
            self.cursor_pos = start;
        }

        self.input.insert_str(self.cursor_pos, &item.label);
        self.cursor_pos += item.label.len();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    break;
                }
                app.on_key(key.code);
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Editor Area
    // Use Highlighter to generate Spans.
    let mut highlighter = RatatuiHighlighter::new();
    let trees = lex(&app.input, &app.lang);

    // Flatten tokens for highlighting
    fn highlight_tree(
        tree: &TokenTree,
        highlighter: &mut RatatuiHighlighter,
        lang: &impl mcparse::language::Language,
    ) {
        match tree {
            TokenTree::Token(t) => {
                // Use the atom index to find the correct atom for highlighting
                if let Some(index) = t.atom_index {
                    if let Some(atom) = lang.atoms().get(index) {
                        atom.highlight(t, highlighter);
                        return;
                    }
                }

                // Fallback if index is missing or invalid
                highlighter.highlight(t, HighlightStyle::None);
            }
            TokenTree::Delimited(d, children, _) => {
                // Highlight open delimiter
                highlighter.highlight(
                    &Token {
                        kind: AtomKind::Operator,
                        text: d.open.to_string(),
                        location: SourceLocation {
                            span: (0, 0).into(),
                        }, // Dummy location
                        atom_index: None,
                        binding: None,
                    },
                    HighlightStyle::Punctuation,
                );

                for child in children {
                    highlight_tree(child, highlighter, lang);
                }

                // Highlight close delimiter
                highlighter.highlight(
                    &Token {
                        kind: AtomKind::Operator,
                        text: d.close.to_string(),
                        location: SourceLocation {
                            span: (0, 0).into(),
                        }, // Dummy location
                        atom_index: None,
                        binding: None,
                    },
                    HighlightStyle::Punctuation,
                );
            }
            TokenTree::Group(children) => {
                for child in children {
                    highlight_tree(child, highlighter, lang);
                }
            }
            TokenTree::Error(_msg) => {
                // How to represent error text? The error token doesn't carry the text it skipped easily unless we change TokenTree::Error
                // But wait, lexer now produces Unknown tokens for skipped text!
                // So TokenTree::Error might not contain text we want to display.
                // If we have Unknown tokens, they are handled in Token case.
            }
            TokenTree::Empty => {}
        }
    }

    for tree in &trees {
        highlight_tree(tree, &mut highlighter, &app.lang);
    }

    let lines = highlighter.into_lines();

    let input_paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Editor (Ctrl+C to quit)"),
    );

    f.render_widget(input_paragraph, chunks[0]);

    // Cursor
    // Calculate cursor position (row, col)
    let (cursor_x, cursor_y) = {
        let mut x = 0;
        let mut y = 0;
        for (i, c) in app.input.chars().enumerate() {
            if i == app.cursor_pos {
                break;
            }
            if c == '\n' {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
        }
        (x, y)
    };

    f.set_cursor_position((
        chunks[0].x + 1 + cursor_x as u16,
        chunks[0].y + 1 + cursor_y as u16,
    ));

    // Status Bar
    // Find token at cursor
    let mut status_text = format!("Cursor: {} ({}, {})", app.cursor_pos, cursor_x, cursor_y);

    fn find_token_at(trees: &[TokenTree], pos: usize) -> Option<&Token> {
        for tree in trees {
            match tree {
                TokenTree::Token(t) => {
                    if t.location.contains(pos) {
                        return Some(t);
                    }
                }
                TokenTree::Delimited(_, children, loc) => {
                    if loc.contains(pos) {
                        if let Some(found) = find_token_at(children, pos) {
                            return Some(found);
                        }
                        // If not in children, maybe on delimiters?
                        // Delimiter tokens are not stored explicitly with location in TokenTree::Delimited
                        // But the group location covers them.
                    }
                }
                TokenTree::Group(children) => {
                    if let Some(found) = find_token_at(children, pos) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    if let Some(token) = find_token_at(&trees, app.cursor_pos.saturating_sub(1)) {
        status_text.push_str(&format!(" | Token: {:?} ({:?})", token.kind, token.text));
    }

    let status =
        Paragraph::new(status_text).block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[1]);

    // Completion Popup
    if app.show_completions && !app.completions.is_empty() {
        let items: Vec<ListItem> = app
            .completions
            .iter()
            .map(|i| ListItem::new(i.label.clone()))
            .collect();

        let completions_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Completions"))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol(">> ");

        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area); // Clear background
        f.render_stateful_widget(completions_list, area, &mut app.completion_state);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
