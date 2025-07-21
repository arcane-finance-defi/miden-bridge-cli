use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use miden_objects::{Word, account::Account};
use miden_tx::utils::Serializable;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

use super::flattened_vec::FlattenedU8Vec;
use crate::store::web_store::{
    note::utils::{SerializedInputNoteData, SerializedOutputNoteData},
    transaction::utils::SerializedTransactionData,
};

// Sync IndexedDB Operations
#[wasm_bindgen(module = "/src/store/web_store/js/sync.js")]

extern "C" {
    // GETS
    // ================================================================================================

    #[wasm_bindgen(js_name = getSyncHeight)]
    pub fn idxdb_get_sync_height() -> js_sys::Promise;

    #[wasm_bindgen(js_name = getNoteTags)]
    pub fn idxdb_get_note_tags() -> js_sys::Promise;

    // INSERTS
    // ================================================================================================

    #[wasm_bindgen(js_name = addNoteTag)]
    pub fn idxdb_add_note_tag(
        tag: Vec<u8>,
        source_note_id: Option<String>,
        source_account_id: Option<String>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = applyStateSync)]
    pub fn idxdb_apply_state_sync(state_update: JsStateSyncUpdate) -> js_sys::Promise;

    // DELETES
    // ================================================================================================
    #[wasm_bindgen(js_name = removeNoteTag)]
    pub fn idxdb_remove_note_tag(
        tag: Vec<u8>,
        source_note_id: Option<String>,
        source_account_id: Option<String>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = discardTransactions)]
    pub fn idxdb_discard_transactions(transactions: Vec<String>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = receiveStateSync)]
    pub fn idxdb_receive_state_sync(state_update: JsStateSyncUpdate) -> js_sys::Promise;
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct JsStateSyncUpdate {
    #[wasm_bindgen(js_name = "blockNum")]
    pub block_num: String,
    #[wasm_bindgen(js_name = "flattenedNewBlockHeaders")]
    pub flattened_new_block_headers: FlattenedU8Vec,
    #[wasm_bindgen(js_name = "newBlockNums")]
    pub new_block_nums: Vec<String>,
    #[wasm_bindgen(js_name = "flattenedPartialBlockChainPeaks")]
    pub flattened_partial_blockchain_peaks: FlattenedU8Vec,
    #[wasm_bindgen(js_name = "blockHasRelevantNotes")]
    pub block_has_relevant_notes: Vec<u8>,
    #[wasm_bindgen(js_name = "serializedNodeIds")]
    pub serialized_node_ids: Vec<String>,
    #[wasm_bindgen(js_name = "serializedNodes")]
    pub serialized_nodes: Vec<String>,
    #[wasm_bindgen(js_name = "noteTagsToRemove")]
    pub note_tags_to_remove: Vec<String>,
    #[wasm_bindgen(js_name = "serializedInputNotes")]
    pub serialized_input_notes: Vec<SerializedInputNoteData>,
    #[wasm_bindgen(js_name = "serializedOutputNotes")]
    pub serialized_output_notes: Vec<SerializedOutputNoteData>,
    #[wasm_bindgen(js_name = "accountUpdates")]
    pub account_updates: Vec<JsAccountUpdate>,
    #[wasm_bindgen(js_name = "transactionUpdates")]
    pub transaction_updates: Vec<SerializedTransactionData>,
}

#[wasm_bindgen(getter_with_clone, inspectable)]
#[derive(Clone)]
pub struct JsAccountUpdate {
    #[wasm_bindgen(js_name = "storageRoot")]
    pub storage_root: String,
    #[wasm_bindgen(js_name = "storageSlots")]
    pub storage_slots: Vec<u8>,
    #[wasm_bindgen(js_name = "assetVaultRoot")]
    pub asset_vault_root: String,
    #[wasm_bindgen(js_name = "assetBytes")]
    pub asset_bytes: Vec<u8>,
    #[wasm_bindgen(js_name = "accountId")]
    pub account_id: String,
    #[wasm_bindgen(js_name = "codeRoot")]
    pub code_root: String,
    #[wasm_bindgen(js_name = "committed")]
    pub committed: bool,
    #[wasm_bindgen(js_name = "nonce")]
    pub nonce: String,
    #[wasm_bindgen(js_name = "accountCommitment")]
    pub account_commitment: String,
    #[wasm_bindgen(js_name = "accountSeed")]
    pub account_seed: Option<Vec<u8>>,
}

impl JsAccountUpdate {
    pub fn from_account(account: &Account, account_seed: Option<Word>) -> Self {
        let asset_vault = account.vault();
        Self {
            storage_root: account.storage().commitment().to_string(),
            storage_slots: account.storage().to_bytes(),
            asset_vault_root: asset_vault.root().to_string(),
            asset_bytes: asset_vault.assets().collect::<Vec<_>>().to_bytes(),
            account_id: account.id().to_string(),
            code_root: account.code().commitment().to_string(),
            committed: account.is_public(),
            nonce: account.nonce().to_string(),
            account_commitment: account.commitment().to_string(),
            account_seed: account_seed.map(|seed| seed.to_bytes()),
        }
    }
}
