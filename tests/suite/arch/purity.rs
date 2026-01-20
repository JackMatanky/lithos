//! Architecture tests for the domain crate.

use lithos_test_utils::arch::assert_no_prohibited_dependencies;

#[cfg(test)]
mod tests {
    use super::*;

    /// 3.6-ARCH-001: `domain_purity`.
    /// Priority: P0.
    #[test]
    fn domain_purity() {
        // GIVEN: the domain crate dependency graph

        // WHEN: checking for prohibited dependencies
        assert_no_prohibited_dependencies("lithos-domain", &[
            "redb", "tokio-fs",
        ]);

        // THEN: the domain crate remains storage and I/O free
    }
}
