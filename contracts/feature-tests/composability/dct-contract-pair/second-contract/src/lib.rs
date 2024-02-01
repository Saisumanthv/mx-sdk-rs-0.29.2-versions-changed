#![no_std]

dharitri_wasm::imports!();

#[dharitri_wasm::contract]
pub trait SecondContract {
    #[init]
    fn init(&self, dct_token_identifier: TokenIdentifier) {
        self.set_contract_dct_token_identifier(&dct_token_identifier);
    }

    #[payable("*")]
    #[endpoint(acceptDctPayment)]
    fn accept_dct_payment(&self, #[payment_token] actual_token_identifier: TokenIdentifier) {
        let expected_token_identifier = self.get_contract_dct_token_identifier();
        require!(
            actual_token_identifier == expected_token_identifier,
            "Wrong dct token"
        );
    }

    #[payable("*")]
    #[endpoint(rejectDctPayment)]
    fn reject_dct_payment(&self) {
        sc_panic!("Rejected")
    }

    // storage

    #[storage_set("dctTokenName")]
    fn set_contract_dct_token_identifier(&self, dct_token_identifier: &TokenIdentifier);

    #[view(getdctTokenName)]
    #[storage_get("dctTokenName")]
    fn get_contract_dct_token_identifier(&self) -> TokenIdentifier;
}
