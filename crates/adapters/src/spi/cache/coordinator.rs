/// Cache reader coordinator for multi-layer caching.
pub struct Reader<K, V> {
    _pd: std::marker::PhantomData<(K, V)>,
}

/// Cache writer coordinator for multi-layer caching.
pub struct Writer<K, V> {
    _pd: std::marker::PhantomData<(K, V)>,
}

#[cfg(test)]
mod tests {
    mod coordinator_init {
        #[test]
        fn verify_linkage() {
            // This test verifies that we can at least see the types
            // proving the module is correctly linked and re-exported.
            let _reader: crate::spi::cache::ReaderCoordinator<String, String>;
            let _writer: crate::spi::cache::WriterCoordinator<String, String>;
        }
    }
}
