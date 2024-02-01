use core::marker::PhantomData;

use dharitri_codec::TopEncodeMulti;

use crate::{
    api::{BlockchainApiImpl, SendApi, SendApiImpl},
    contract_base::ExitCodecErrorHandler,
    err_msg,
    types::{BigUint, CodeMetadata, ManagedAddress, ManagedBuffer, ManagedVec},
};

use super::ManagedArgBuffer;

/// Using max u64 to represent maximum possible gas,
/// so that the value zero is not reserved and can be specified explicitly.
/// Leaving the gas limit unspecified will replace it with `api.get_gas_left()`.
const UNSPECIFIED_GAS_LIMIT: u64 = u64::MAX;

#[must_use]
pub struct ContractDeploy<SA>
where
    SA: SendApi + 'static,
{
    _phantom: PhantomData<SA>,
    to: ManagedAddress<SA>, // only used for Upgrade, ignored for Deploy
    moax_payment: BigUint<SA>,
    explicit_gas_limit: u64,
    arg_buffer: ManagedArgBuffer<SA>,
}

/// Syntactical sugar to help macros to generate code easier.
/// Unlike calling `ContractDeploy::<SA>::new`, here types can be inferred from the context.
pub fn new_contract_deploy<SA>(to: ManagedAddress<SA>) -> ContractDeploy<SA>
where
    SA: SendApi + 'static,
{
    let mut contract_deploy = ContractDeploy::<SA>::new();
    contract_deploy.to = to;
    contract_deploy
}

impl<SA> Default for ContractDeploy<SA>
where
    SA: SendApi + 'static,
{
    fn default() -> Self {
        let zero = BigUint::zero();
        let zero_address = ManagedAddress::zero();
        let arg_buffer = ManagedArgBuffer::new_empty();
        ContractDeploy {
            _phantom: PhantomData,
            to: zero_address,
            moax_payment: zero,
            explicit_gas_limit: UNSPECIFIED_GAS_LIMIT,
            arg_buffer,
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl<SA> ContractDeploy<SA>
where
    SA: SendApi + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_moax_transfer(mut self, payment_amount: BigUint<SA>) -> Self {
        self.moax_payment = payment_amount;
        self
    }

    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.explicit_gas_limit = gas_limit;
        self
    }

    pub fn push_endpoint_arg<T: TopEncodeMulti>(&mut self, endpoint_arg: &T) {
        let h = ExitCodecErrorHandler::<SA>::from(err_msg::CONTRACT_CALL_ENCODE_ERROR);
        let Ok(()) = endpoint_arg.multi_encode_or_handle_err(&mut self.arg_buffer, h);
    }

    // pub fn get_mut_arg_buffer(&mut self) -> &mut ArgBuffer {
    //     &mut self.arg_buffer
    // }

    // /// Provided for cases where we build the contract deploy by hand.
    // pub fn push_argument_raw_bytes(&mut self, bytes: &[u8]) {
    //     self.arg_buffer.push_argument_bytes(bytes);
    // }

    fn resolve_gas_limit(&self) -> u64 {
        if self.explicit_gas_limit == UNSPECIFIED_GAS_LIMIT {
            SA::blockchain_api_impl().get_gas_left()
        } else {
            self.explicit_gas_limit
        }
    }
}

impl<SA> ContractDeploy<SA>
where
    SA: SendApi + 'static,
{
    /// Executes immediately, synchronously, and returns Some(Address) of the deployed contract.  
    /// Will return None if the deploy fails.  
    pub fn deploy_contract(
        self,
        code: &ManagedBuffer<SA>,
        code_metadata: CodeMetadata,
    ) -> (ManagedAddress<SA>, ManagedVec<SA, ManagedBuffer<SA>>) {
        SA::send_api_impl().deploy_contract(
            self.resolve_gas_limit(),
            &self.moax_payment,
            code,
            code_metadata,
            &self.arg_buffer,
        )
    }

    pub fn deploy_from_source(
        self,
        source_address: &ManagedAddress<SA>,
        code_metadata: CodeMetadata,
    ) -> (ManagedAddress<SA>, ManagedVec<SA, ManagedBuffer<SA>>) {
        SA::send_api_impl().deploy_from_source_contract(
            self.resolve_gas_limit(),
            &self.moax_payment,
            source_address,
            code_metadata,
            &self.arg_buffer,
        )
    }

    pub fn upgrade_from_source(
        self,
        source_address: &ManagedAddress<SA>,
        code_metadata: CodeMetadata,
    ) {
        SA::send_api_impl().upgrade_from_source_contract(
            &self.to,
            self.resolve_gas_limit(),
            &self.moax_payment,
            source_address,
            code_metadata,
            &self.arg_buffer,
        )
    }

    pub fn upgrade_contract(self, code: &ManagedBuffer<SA>, code_metadata: CodeMetadata) {
        SA::send_api_impl().upgrade_contract(
            &self.to,
            self.resolve_gas_limit(),
            &self.moax_payment,
            code,
            code_metadata,
            &self.arg_buffer,
        );
    }
}