use miden_client::ClientError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotesErrors {
    #[error(transparent)]
    InternalClientError(#[from] ClientError),
}
