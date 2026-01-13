//! Architecture tests for the domain crate.

use lithos_test_utils::arch::assert_no_prohibited_dependencies;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_purity() {
        // Domain should not depend on storage or IO-heavy async crates
        assert_no_prohibited_dependencies(
            "lithos-domain",
            &["redb", "tokio-fs"],
        );
    }
}
