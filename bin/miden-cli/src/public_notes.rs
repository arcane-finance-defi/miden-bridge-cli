use miden_bridge::accounts::token_wrapper::bridge_note_tag;
use miden_bridge::notes::bridge::{bridge, croschain};
use miden_client::store::InputNoteRecord;
use miden_client::{Felt, ZERO};
use miden_objects::asset::Asset;
use miden_objects::note::{
    Note,
    NoteAssets,
    NoteExecutionHint,
    NoteInputs,
    NoteMetadata,
    NoteRecipient,
    NoteType,
};
use miden_objects::transaction::OutputNote;
use miden_objects::{NoteError, Word};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublicNoteConstructorError {
    #[error("Crosschain note haven't fungible asset in vault")]
    FungibleAssetNotFound,
    #[error("Note metadata construction fails")]
    NoteMetadataCreationError(#[source] NoteError),
    #[error("Note inputs too much")]
    NoteInputsCreationError(#[source] NoteError),
    #[error("Malformed serial number")]
    MalformedSerialNumber,
}

pub fn is_crosschain_note(input: InputNoteRecord) -> bool {
    input.details().script().root().to_hex() == croschain().root().to_hex()
}

pub fn get_public_bridge_output_note(
    input_note: InputNoteRecord,
) -> Result<OutputNote, PublicNoteConstructorError> {
    let crosschain_asset = input_note
        .details()
        .assets()
        .iter()
        .last()
        .ok_or(PublicNoteConstructorError::FungibleAssetNotFound)?;
    let crosschain_asset = match crosschain_asset {
        Asset::Fungible(asset) => Ok(asset),
        _ => Err(PublicNoteConstructorError::FungibleAssetNotFound),
    }?;
    let script = bridge();
    let assets = NoteAssets::default();
    let metadata = NoteMetadata::new(
        crosschain_asset.faucet_id(),
        NoteType::Public,
        bridge_note_tag(),
        NoteExecutionHint::Always,
        ZERO,
    )
    .map_err(PublicNoteConstructorError::NoteMetadataCreationError)?;

    let serial_num = <&[Felt; 4]>::try_from(&input_note.details().inputs().values()[..4])
        .expect("from slice to array");
    let serial_num = Word::try_from(serial_num)
        .map_err(|_| PublicNoteConstructorError::MalformedSerialNumber)?;

    let inputs = NoteInputs::new(
        vec![
            Word::from(Asset::Fungible(crosschain_asset.clone())).to_vec(),
            input_note.details().inputs().values()[4..].to_vec(),
        ]
        .concat(),
    )
    .map_err(PublicNoteConstructorError::NoteInputsCreationError)?;

    let recipient = NoteRecipient::new(serial_num, script, inputs);

    Ok(OutputNote::Full(Note::new(assets, metadata, recipient)))
}
