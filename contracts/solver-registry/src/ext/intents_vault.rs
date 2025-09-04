use near_sdk::{ext_contract, AccountId, PublicKey};

#[allow(dead_code)]
#[ext_contract(ext_intents_vault)]
trait IntentsVaultContract {
    fn add_public_key(intents_contract_id: AccountId, public_key: PublicKey);
    fn remove_public_key(intents_contract_id: AccountId, public_key: PublicKey);
}
