#[near]
impl Contract {
    #[payable]
    pub fn ft_on_transfer(&mut self, token_id: AccountId, amount: U128) {
        self.require_parent_account();
    }
}
