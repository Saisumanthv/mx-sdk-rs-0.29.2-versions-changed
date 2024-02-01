mod dns_proxy {
    dharitri_wasm::imports!();

    #[dharitri_wasm::proxy]
    pub trait Dns {
        #[payable("MOAX")]
        #[endpoint]
        fn register(&self, name: &ManagedBuffer);
    }
}

dharitri_wasm::imports!();

/// Standard smart contract module that deals with registering usernames in a DNS contract.
///
/// Dharitri usernames/herotags need to be requested by the beneficiary.
/// For a contract, this means that they need an endpoint via which to request a username from the DNS.
///
#[dharitri_wasm::module]
pub trait DnsModule {
    #[proxy]
    fn dns_proxy(&self, to: ManagedAddress) -> dns_proxy::Proxy<Self::Api>;

    #[payable("MOAX")]
    #[only_owner]
    #[endpoint(dnsRegister)]
    fn dns_register(
        &self,
        dns_address: ManagedAddress,
        name: ManagedBuffer,
        #[payment] payment: BigUint,
    ) {
        self.dns_proxy(dns_address)
            .register(&name)
            .with_moax_transfer(payment)
            .async_call()
            .call_and_exit()
    }
}
