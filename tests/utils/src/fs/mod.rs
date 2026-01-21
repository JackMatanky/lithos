pub mod temp;
pub mod vault;

// Re-export with File prefix for top-level imports
pub use temp::{
    TempDir as FileTempDir, TestOutput as FileTestOutput, generate_unique_name,
    path_utils,
};
pub use vault::TestVault as FileTestVault;
