use thiserror::Error;
use miden_client::ClientError;

#[derive(Debug, Error)]
pub enum NotesErrors {
    #[error(transparent)]
    InternalClientError(#[from] ClientError),
}