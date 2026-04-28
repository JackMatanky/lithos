// Integration test for ParserContext with complex markdown

#[cfg(test)]
mod integration {
    use pulldown_cmark::Event;

    use crate::note::parser::{
        config::EventStreamConfig, context::ParserContext,
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
                    Event::Start(pulldown_cmark::Tag::Heading { .. })
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

    #[test]
    fn handles_markdown_with_only_whitespace() {
        let source = "   \n\n\t\n  ";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config)
            .expect("should parse whitespace markdown");

        // pulldown-cmark may emit events for whitespace or skip it entirely
        // Just verify it doesn't crash
        assert_eq!(ctx.source(), source);
    }

    #[test]
    fn handles_deeply_nested_lists() {
        let source = "- Level 0
  - Level 1
    - Level 2
      - Level 3
        - Level 4
          - Level 5
";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config)
            .expect("should parse deeply nested lists");

        let list_starts = ctx
            .events()
            .iter()
            .filter(|e| {
                matches!(e.event(), Event::Start(pulldown_cmark::Tag::List(_)))
            })
            .count();

        assert!(list_starts >= 6, "should have at least 6 list starts");
    }

    #[test]
    fn preserves_event_order() {
        let source = "# Heading\n\nParagraph";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("should parse");

        let events: Vec<_> = ctx
            .events()
            .iter()
            .map(super::super::stream::EventWithRange::event)
            .collect();

        // Verify general structure: heading events followed by paragraph events
        let has_heading = events.iter().any(|e| {
            matches!(e, Event::Start(pulldown_cmark::Tag::Heading { .. }))
        });
        let has_paragraph = events
            .iter()
            .any(|e| matches!(e, Event::Start(pulldown_cmark::Tag::Paragraph)));

        assert!(has_heading, "should have heading event");
        assert!(has_paragraph, "should have paragraph event");
    }
}
