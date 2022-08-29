
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to convert {0} to {1}, {2}")]
    CaseConvert(String, String, String),
    #[error("Failed to convert {0} reason: {1}")]
    CaseDerive(String, String),
}
