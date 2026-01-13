// Library exports for testing
// This allows integration tests to import modules from the main crate

pub mod db;
pub mod services;

// Re-export commonly used types for convenience
pub use db::DbPool;
pub use services::plugins::PluginSupervisor;
