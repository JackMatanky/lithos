// Integration test for ParserContext with complex markdown

#[cfg(test)]
mod integration {
    use crate::note::parser::{
        config::EventStreamConfig,
        context::ParserContext,
        stream::{BlockType, ParserEvent},
    };

    #[test]
    fn parses_complex_markdown_with_multiple_features() {
        let source = "# Large Document Test

## Section 1
Paragraph 1 with [ref1][].

[ref1]: /url1

## Section 2
Paragraph 2 with [ref2][].

[ref2]: /url2

- List item 1
  - Nested item 1.1
    - Deep nested 1.1.1

## Section 3
Code block:
```rust
fn test() {}
```

> Blockquote
> - List in quote
";

        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config)
            .expect("should parse complex markdown");

        // Verify events are cached
        assert!(!ctx.events().is_empty(), "should cache events");

        // Verify reference resolution
        assert_eq!(
            ctx.references().resolve("ref1"),
            Some("/url1"),
            "should resolve first reference"
        );
        assert_eq!(
            ctx.references().resolve("ref2"),
            Some("/url2"),
            "should resolve second reference"
        );

        // Verify we have heading events
        let heading_count = ctx
            .events()
            .iter()
            .filter(|e| {
                matches!(
                    e.event(),
                    ParserEvent::BlockStart(BlockType::Heading { .. })
                )
            })
            .count();
        assert!(heading_count >= 3, "should have at least 3 headings");

        // Verify source is preserved
        assert_eq!(ctx.source(), source);
    }

    #[test]
    fn handles_empty_markdown() {
        let source = "";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config)
            .expect("should parse empty markdown");

        assert!(
            ctx.events().is_empty(),
            "empty markdown should have no events"
        );
        assert_eq!(ctx.source(), "");
    }
}
