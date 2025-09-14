use miden_objects::Word as NativeWord;
use miden_objects::account::AccountId as NativeAccountId;
use miden_objects::asset::{Asset as NativeAsset, FungibleAsset as FungibleAssetNative};
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::word::Word;

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct FungibleAsset(FungibleAssetNative);

#[wasm_bindgen]
impl FungibleAsset {
    #[wasm_bindgen(constructor)]
    pub fn new(faucet_id: &AccountId, amount: u64) -> FungibleAsset {
        let native_faucet_id: NativeAccountId = faucet_id.into();
        let native_asset = FungibleAssetNative::new(native_faucet_id, amount).unwrap();

        FungibleAsset(native_asset)
    }

    #[wasm_bindgen(js_name = "faucetId")]
    pub fn faucet_id(&self) -> AccountId {
        self.0.faucet_id().into()
    }

    pub fn amount(&self) -> u64 {
        self.0.amount()
    }

    #[wasm_bindgen(js_name = "intoWord")]
    pub fn into_word(&self) -> Word {
        let native_word: NativeWord = self.0.into();
        native_word.into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<FungibleAsset> for NativeAsset {
    fn from(fungible_asset: FungibleAsset) -> Self {
        fungible_asset.0.into()
    }
}

impl From<&FungibleAsset> for NativeAsset {
    fn from(fungible_asset: &FungibleAsset) -> Self {
        fungible_asset.0.into()
    }
}

impl From<FungibleAssetNative> for FungibleAsset {
    fn from(native_asset: FungibleAssetNative) -> Self {
        FungibleAsset(native_asset)
    }
}

impl From<&FungibleAssetNative> for FungibleAsset {
    fn from(native_asset: &FungibleAssetNative) -> Self {
        FungibleAsset(*native_asset)
    }
}
