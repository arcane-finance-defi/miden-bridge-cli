use std::sync::Arc;
use miden_client::note::NoteInclusionProof;
use miden_objects::note::NoteId;
use miden_client::Client;
use miden_client::ClientError;
use miden_client::auth::TransactionAuthenticator;
use miden_client::rpc::{NodeRpcClient, RpcError};
use crate::notes::errors::NotesErrors;

pub async fn check_note_existence<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    rpc_api: Arc<dyn NodeRpcClient>,
    note_id: &NoteId,
) -> Result<bool, NotesErrors> {
    let proof = get_fetched_note_proof(rpc_api, note_id.clone())
        .await?;

    match proof {
        None => Ok(false),
        Some(proof) => {
            let sync_height = client.get_sync_height().await?;
            if proof.location().block_num().gt(&sync_height) {
                client.sync_state().await?;
            }
            Ok(true)
        }
    }
}

pub async fn get_fetched_note_proof(
    rpc_api: Arc<dyn NodeRpcClient>,
    note_id: NoteId,
) -> Result<Option<NoteInclusionProof>, ClientError> {
    let result = rpc_api.get_note_by_id(note_id).await;
    
    let note = match result {
        Ok(fetched_note) => Ok(Some(fetched_note)),
        Err(RpcError::NoteNotFound(_)) => Ok(None),
        Err(err) => return Err(ClientError::RpcError(err)),
    };

    match note {
        Ok(Some(note)) => Ok(Some(note.inclusion_proof().clone())),
        Ok(None) => Ok(None),
        Err(err) => return Err(ClientError::RpcError(err)),
    }
}