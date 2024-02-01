#![no_std]

dharitri_wasm::imports!();
dharitri_wasm::derive_imports!();

const MOAX_NUM_DECIMALS: usize = 18;

/// Converts between MOAX and a wrapped MOAX DCT token.
///	1 MOAX = 1 wrapped MOAX and is interchangeable at all times.
/// Also manages the supply of wrapped MOAX tokens.
#[dharitri_wasm::contract]
pub trait MoaxDctSwap {
    #[init]
    fn init(&self) {}

    // endpoints - owner-only

    #[only_owner]
    #[payable("MOAX")]
    #[endpoint(issueWrappedMoax)]
    fn issue_wrapped_moax(&self, token_display_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        require!(
            self.wrapped_moax_token_id().is_empty(),
            "wrapped moax was already issued"
        );

        let issue_cost = self.call_value().moax_value();
        let caller = self.blockchain().get_caller();
        let initial_supply = BigUint::zero();

        self.issue_started_event(&caller, &token_ticker, &initial_supply);

        self.send()
            .dct_system_sc_proxy()
            .issue_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
                &initial_supply,
                FungibleTokenProperties {
                    num_decimals: MOAX_NUM_DECIMALS,
                    can_freeze: false,
                    can_wipe: false,
                    can_pause: false,
                    can_mint: true,
                    can_burn: false,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: false,
                },
            )
            .async_call()
            .with_callback(self.callbacks().dct_issue_callback(&caller))
            .call_and_exit()
    }

    #[callback]
    fn dct_issue_callback(
        &self,
        caller: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let (returned_tokens, token_identifier) = self.call_value().payment_token_pair();

        // callback is called with DCTTransfer of the newly issued token, with the amount requested,
        // so we can get the token identifier and amount from the call data
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.issue_success_event(caller, &token_identifier, &returned_tokens);
                self.wrapped_moax_token_id().set(&token_identifier);
            },
            ManagedAsyncCallResult::Err(message) => {
                self.issue_failure_event(caller, &message.err_msg);

                // return issue cost to the owner
                // TODO: test that it works
                if token_identifier.is_moax() && returned_tokens > 0 {
                    self.send().direct_moax(caller, &returned_tokens, &[]);
                }
            },
        }
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        require!(
            !self.wrapped_moax_token_id().is_empty(),
            "Must issue token first"
        );

        let roles = [DctLocalRole::Mint, DctLocalRole::Burn];
        self.send()
            .dct_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.wrapped_moax_token_id().get(),
                roles[..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    // endpoints

    #[payable("MOAX")]
    #[endpoint(wrapMoax)]
    fn wrap_moax(&self) {
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_token.is_moax(), "Only MOAX accepted");
        require!(payment_amount > 0u32, "Payment must be more than 0");

        let wrapped_moax_token_id = self.wrapped_moax_token_id().get();
        self.send()
            .dct_local_mint(&wrapped_moax_token_id, 0, &payment_amount);

        let caller = self.blockchain().get_caller();
        self.send()
            .direct(&caller, &wrapped_moax_token_id, 0, &payment_amount, &[]);
    }

    #[payable("*")]
    #[endpoint(unwrapMoax)]
    fn unwrap_moax(&self) {
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        let wrapped_moax_token_id = self.wrapped_moax_token_id().get();

        require!(payment_token == wrapped_moax_token_id, "Wrong dct token");
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
        // this should never happen, but we'll check anyway
        require!(
            payment_amount <= self.get_locked_moax_balance(),
            "Contract does not have enough funds"
        );

        self.send()
            .dct_local_burn(&wrapped_moax_token_id, 0, &payment_amount);

        // 1 wrapped MOAX = 1 MOAX, so we pay back the same amount
        let caller = self.blockchain().get_caller();
        self.send().direct_moax(&caller, &payment_amount, &[]);
    }

    #[view(getLockedMoaxBalance)]
    fn get_locked_moax_balance(&self) -> BigUint {
        self.blockchain()
            .get_sc_balance(&TokenIdentifier::moax(), 0)
    }

    // storage

    #[view(getWrappedMoaxTokenIdentifier)]
    #[storage_mapper("wrappedMoaxTokenId")]
    fn wrapped_moax_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    // events

    #[event("issue-started")]
    fn issue_started_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] token_ticker: &ManagedBuffer,
        initial_supply: &BigUint,
    );

    #[event("issue-success")]
    fn issue_success_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] token_identifier: &TokenIdentifier,
        initial_supply: &BigUint,
    );

    #[event("issue-failure")]
    fn issue_failure_event(&self, #[indexed] caller: &ManagedAddress, message: &ManagedBuffer);

    #[event("wrap-moax")]
    fn wrap_moax_event(&self, #[indexed] user: &ManagedAddress, amount: &BigUint);

    #[event("unwrap-moax")]
    fn unwrap_moax_event(&self, #[indexed] user: &ManagedAddress, amount: &BigUint);
}
