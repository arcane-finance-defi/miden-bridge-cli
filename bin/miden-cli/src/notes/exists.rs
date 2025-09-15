use miden_objects::note::NoteId;
use miden_client::auth::TransactionAuthenticator;
use miden_client::Client;
use crate::notes::errors::NotesErrors;

pub async fn check_note_existence<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    note_id: &NoteId,
) -> Result<bool, NotesErrors> {
    let proof = client.get_note_inclusion_proof(note_id.clone())
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