use alloc::string::ToString;

use miden_objects::block::BlockHeader;
use miden_objects::note::{NoteId, NoteInclusionProof, NoteMetadata};
use miden_objects::transaction::TransactionId;

use super::{InputNoteState, NoteStateHandler};
use crate::store::NoteRecordError;

/// Information related to notes in the [`InputNoteState::ConsumedExternal`] state.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumedExternalNoteState {
    /// Block height at which the note was nullified.
    pub nullifier_block_height: u32,
}

impl NoteStateHandler for ConsumedExternalNoteState {
    fn inclusion_proof_received(
        &self,
        _inclusion_proof: NoteInclusionProof,
        _metadata: NoteMetadata,
    ) -> Result<Option<InputNoteState>, NoteRecordError> {
        Ok(None)
    }

    fn consumed_externally(
        &self,
        _nullifier_block_height: u32,
    ) -> Result<Option<InputNoteState>, NoteRecordError> {
        Ok(None)
    }

    fn block_header_received(
        &self,
        _note_id: NoteId,
        _block_header: &BlockHeader,
    ) -> Result<Option<InputNoteState>, NoteRecordError> {
        Ok(None)
    }

    fn consumed_locally(
        &self,
        _consumer_account: miden_objects::account::AccountId,
        _consumer_transaction: miden_objects::transaction::TransactionId,
        _current_timestamp: Option<u64>,
    ) -> Result<Option<InputNoteState>, NoteRecordError> {
        Err(NoteRecordError::NoteNotConsumable("Note already consumed".to_string()))
    }

    fn transaction_committed(
        &self,
        _transaction_id: TransactionId,
        _block_height: u32,
    ) -> Result<Option<InputNoteState>, NoteRecordError> {
        Err(NoteRecordError::InvalidStateTransition(
            "Only processing notes can be committed in a local transaction".to_string(),
        ))
    }

    fn metadata(&self) -> Option<&NoteMetadata> {
        None
    }

    fn inclusion_proof(&self) -> Option<&NoteInclusionProof> {
        None
    }

    fn consumer_transaction_id(&self) -> Option<&TransactionId> {
        None
    }
}

impl miden_tx::utils::Serializable for ConsumedExternalNoteState {
    fn write_into<W: miden_tx::utils::ByteWriter>(&self, target: &mut W) {
        self.nullifier_block_height.write_into(target);
    }
}

impl miden_tx::utils::Deserializable for ConsumedExternalNoteState {
    fn read_from<R: miden_tx::utils::ByteReader>(
        source: &mut R,
    ) -> Result<Self, miden_tx::utils::DeserializationError> {
        let nullifier_block_height = u32::read_from(source)?;
        Ok(ConsumedExternalNoteState { nullifier_block_height })
    }
}

impl From<ConsumedExternalNoteState> for InputNoteState {
    fn from(state: ConsumedExternalNoteState) -> Self {
        InputNoteState::ConsumedExternal(state)
    }
}
