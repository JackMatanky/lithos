use std::sync::Arc;

use crate::spi::{
    cache::{CacheReader, CacheWriter},
    errors::CacheError,
};

/// Type alias for the coordinator handle pair.
pub type CoordinatorPair<K, V> = (Reader<K, V>, Writer<K, V>);

/// Internal state shared between Reader and Writer handles.
struct Inner<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[expect(dead_code, reason = "Will be used in Phase 4")]
    memory_reader: Arc<dyn CacheReader<K, V>>,
    #[expect(dead_code, reason = "Will be used in Phase 5")]
    memory_writer: Arc<dyn CacheWriter<K, V>>,
    #[expect(dead_code, reason = "Will be used in Phase 4")]
    disk_reader: Arc<dyn CacheReader<K, V>>,
    #[expect(dead_code, reason = "Will be used in Phase 5")]
    disk_writer: Arc<dyn CacheWriter<K, V>>,
}

/// Cache reader coordinator for multi-layer caching.
pub struct Reader<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Will be used in Phase 4")
    )]
    inner: Arc<Inner<K, V>>,
}

/// Cache writer coordinator for multi-layer caching.
pub struct Writer<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Will be used in Phase 5")
    )]
    inner: Arc<Inner<K, V>>,
}

/// Builder for constructing a `CacheCoordinator` pair.
pub struct Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    disk_reader: Option<Arc<dyn CacheReader<K, V>>>,
    disk_writer: Option<Arc<dyn CacheWriter<K, V>>>,
    memory_reader: Option<Arc<dyn CacheReader<K, V>>>,
    memory_writer: Option<Arc<dyn CacheWriter<K, V>>>,
}

impl<K, V> Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Build the coordinator handles.
    ///
    /// # Errors
    /// Returns `CacheError::BackendError` if any of the required cache ports
    /// are not set.
    #[inline]
    pub fn build(self) -> Result<CoordinatorPair<K, V>, CacheError> {
        let inner = Arc::new(Inner {
            memory_reader: self.memory_reader.ok_or_else(|| {
                CacheError::BackendError {
                    backend: "coordinator",
                    message: "memory_reader is required".into(),
                }
            })?,
            memory_writer: self.memory_writer.ok_or_else(|| {
                CacheError::BackendError {
                    backend: "coordinator",
                    message: "memory_writer is required".into(),
                }
            })?,
            disk_reader: self.disk_reader.ok_or_else(|| {
                CacheError::BackendError {
                    backend: "coordinator",
                    message: "disk_reader is required".into(),
                }
            })?,
            disk_writer: self.disk_writer.ok_or_else(|| {
                CacheError::BackendError {
                    backend: "coordinator",
                    message: "disk_writer is required".into(),
                }
            })?,
        });

        Ok((
            Reader {
                inner: Arc::clone(&inner),
            },
            Writer {
                inner,
            },
        ))
    }

    /// Set the disk reader.
    #[inline]
    #[must_use]
    pub fn disk_reader(mut self, reader: Arc<dyn CacheReader<K, V>>) -> Self {
        self.disk_reader = Some(reader);
        self
    }

    /// Set the disk writer.
    #[inline]
    #[must_use]
    pub fn disk_writer(mut self, writer: Arc<dyn CacheWriter<K, V>>) -> Self {
        self.disk_writer = Some(writer);
        self
    }

    /// Set the memory reader.
    #[inline]
    #[must_use]
    pub fn memory_reader(mut self, reader: Arc<dyn CacheReader<K, V>>) -> Self {
        self.memory_reader = Some(reader);
        self
    }

    /// Set the memory writer.
    #[inline]
    #[must_use]
    pub fn memory_writer(mut self, writer: Arc<dyn CacheWriter<K, V>>) -> Self {
        self.memory_writer = Some(writer);
        self
    }

    /// Create a new builder.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            disk_reader: None,
            disk_writer: None,
            memory_reader: None,
            memory_writer: None,
        }
    }
}

impl<K, V> Default for Builder<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod coordinator_init {
        use super::*;
        use crate::spi::cache::{MockCacheReader, MockCacheWriter};

        #[test]
        fn verify_linkage() {
            let _reader: crate::spi::cache::ReaderCoordinator<String, String>;
            let _writer: crate::spi::cache::WriterCoordinator<String, String>;
        }

        #[test]
        fn shares_inner_state_between_handles() {
            let (reader, writer) = Builder::<String, String>::new()
                .memory_reader(Arc::new(MockCacheReader::new()))
                .memory_writer(Arc::new(MockCacheWriter::new()))
                .disk_reader(Arc::new(MockCacheReader::new()))
                .disk_writer(Arc::new(MockCacheWriter::new()))
                .build()
                .expect("Failed to build coordinator");

            assert!(Arc::ptr_eq(&reader.inner, &writer.inner));
        }
    }
}
