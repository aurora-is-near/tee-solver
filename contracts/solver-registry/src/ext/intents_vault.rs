use near_sdk::{AccountId, PublicKey, ext_contract};

#[allow(dead_code)]
#[ext_contract(ext_intents_vault)]
trait IntentsVaultContract {
    fn add_public_key(intents_contract_id: AccountId, public_key: PublicKey);
    fn remove_public_key(intents_contract_id: AccountId, public_key: PublicKey);
}
