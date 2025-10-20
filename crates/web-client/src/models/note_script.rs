use miden_bridge::notes::bridge::croschain as croschain_script;
use miden_client::note::{NoteScript as NativeNoteScript, WellKnownNote};
use miden_objects::PrettyPrint;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteScript(NativeNoteScript);

#[wasm_bindgen]
impl NoteScript {
    /// Print the MAST source for this script.
    #[wasm_bindgen(js_name = toString)]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_pretty_string()
    }

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<NoteScript, JsValue> {
        deserialize_from_uint8array::<NativeNoteScript>(bytes).map(NoteScript)
    }

    pub fn p2id() -> Self {
        WellKnownNote::P2ID.script().into()
    }

    pub fn p2ide() -> Self {
        WellKnownNote::P2IDE.script().into()
    }

    pub fn swap() -> Self {
        WellKnownNote::SWAP.script().into()
    }

    #[wasm_bindgen(js_name = "rootHex")]
    pub fn root_hex(&self) -> String {
        self.0.root().to_hex()
    }

    pub fn crosschain() -> Self {
        croschain_script().into()
    }

    pub fn root(&self) -> Word {
        self.0.root().into()
    }
}
// CONVERSIONS
// ================================================================================================

impl From<NativeNoteScript> for NoteScript {
    fn from(native_note_script: NativeNoteScript) -> Self {
        NoteScript(native_note_script)
    }
}

impl From<&NativeNoteScript> for NoteScript {
    fn from(native_note_script: &NativeNoteScript) -> Self {
        NoteScript(native_note_script.clone())
    }
}

impl From<NoteScript> for NativeNoteScript {
    fn from(note_script: NoteScript) -> Self {
        note_script.0
    }
}

impl From<&NoteScript> for NativeNoteScript {
    fn from(note_script: &NoteScript) -> Self {
        note_script.0.clone()
    }
}
