# Comments

Comments are usually handled at the **Atom** level. You define an Atom that matches the comment syntax (e.g., `// ...`) and returns a token of kind `AtomKind::Comment`.

Most Shapes (like `term`, `seq`) automatically skip `AtomKind::Comment` tokens, just like they skip `AtomKind::Whitespace`.

```rust
define_atom! {
    struct LineComment;
    kind = AtomKind::Comment;
    parse(input) {
        if input.rest.starts_with("//") {
            let len = input.rest.find('\n').unwrap_or(input.rest.len());
             Some((Token::new(AtomKind::Comment, &input.rest[..len], input.offset), input.advance(len)))
        } else {
            None
        }
    }
    highlight(token, h) { h.highlight(token, HighlightStyle::Comment); }
}
```
