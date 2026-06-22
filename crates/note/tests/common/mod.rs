#![allow(dead_code)]

use std::sync::Arc;
use trace_db::Store;
use storage::RedbRepository;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn setup_repository(store: &Arc<Store>) -> RedbRepository {
    RedbRepository::new(Arc::clone(store))
}
