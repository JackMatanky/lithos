# Rust TDD Validation Checklist

- [ ] Story or requirement source identified (or explicitly not provided)
- [ ] Scope detected (file/module/multi-module/project) and validated
- [ ] Scope expansion documented when dependencies require it
- [ ] Red phase tests written with GWT comments (no doc comments)
- [ ] Green phase minimal implementation documented
- [ ] Refactor phase preserves passing tests
- [ ] Public components covered with tests
- [ ] Error paths and edge cases covered
- [ ] Rust-specific invariants validated
- [ ] Rust style and linting standards adhered to (fmt, lint)
- [ ] Quality gates identified (fmt, lint, test, verify)
