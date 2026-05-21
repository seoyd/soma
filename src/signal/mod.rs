pub mod baseline;
pub mod features;
pub mod mock_signal;

pub use baseline::{BaselineSignalConfig, BaselineSignalModel};
pub use features::derive_features;
pub use mock_signal::MockSignalEngine;
