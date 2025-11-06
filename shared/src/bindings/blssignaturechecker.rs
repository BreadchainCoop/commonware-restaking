///Module containing a contract's types and functions.
/**

```solidity
library BN254 {
    struct G1Point { uint256 X; uint256 Y; }
    struct G2Point { uint256[2] X; uint256[2] Y; }
}
```*/
#[allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::style,
    clippy::empty_structs_with_brackets
)]
pub mod BN254 {

    use alloy::sol_types as alloy_sol_types;
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
    struct G1Point { uint256 X; uint256 Y; }
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct G1Point {
        #[allow(missing_docs)]
        pub X: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub Y: alloy::sol_types::private::primitives::aliases::U256,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
        );
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (
            alloy::sol_types::private::primitives::aliases::U256,
            alloy::sol_types::private::primitives::aliases::U256,
        );
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<G1Point> for UnderlyingRustTuple<'_> {
            fn from(value: G1Point) -> Self {
                (value.X, value.Y)
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for G1Point {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {
                    X: tuple.0,
                    Y: tuple.1,
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for G1Point {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for G1Point {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Uint<256> as alloy_sol_types::SolType>::tokenize(
                        &self.X,
                    ),
                    <alloy::sol_types::sol_data::Uint<256> as alloy_sol_types::SolType>::tokenize(
                        &self.Y,
                    ),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encode_packed_to(
                    &tuple, out,
                )
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_packed_encoded_size(
                    &tuple,
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for G1Point {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for G1Point {
            const NAME: &'static str = "G1Point";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed("G1Point(uint256 X,uint256 Y)")
            }
            #[inline]
            fn eip712_components()
            -> alloy_sol_types::private::Vec<alloy_sol_types::private::Cow<'static, str>>
            {
                alloy_sol_types::private::Vec::new()
            }
            #[inline]
            fn eip712_encode_type() -> alloy_sol_types::private::Cow<'static, str> {
                <Self as alloy_sol_types::SolStruct>::eip712_root_type()
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.X)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.Y)
                        .0,
                ]
                    .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for G1Point {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.X)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.Y)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(<Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust));
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.X, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.Y, out);
            }
            #[inline]
            fn encode_topic(rust: &Self::RustType) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, &mut out);
                alloy_sol_types::abi::token::WordToken(alloy_sol_types::private::keccak256(out))
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
    struct G2Point { uint256[2] X; uint256[2] Y; }
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct G2Point {
        #[allow(missing_docs)]
        pub X: [alloy::sol_types::private::primitives::aliases::U256; 2usize],
        #[allow(missing_docs)]
        pub Y: [alloy::sol_types::private::primitives::aliases::U256; 2usize],
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (
            alloy::sol_types::sol_data::FixedArray<alloy::sol_types::sol_data::Uint<256>, 2usize>,
            alloy::sol_types::sol_data::FixedArray<alloy::sol_types::sol_data::Uint<256>, 2usize>,
        );
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (
            [alloy::sol_types::private::primitives::aliases::U256; 2usize],
            [alloy::sol_types::private::primitives::aliases::U256; 2usize],
        );
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<G2Point> for UnderlyingRustTuple<'_> {
            fn from(value: G2Point) -> Self {
                (value.X, value.Y)
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for G2Point {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {
                    X: tuple.0,
                    Y: tuple.1,
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for G2Point {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for G2Point {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <alloy::sol_types::sol_data::FixedArray<
                        alloy::sol_types::sol_data::Uint<256>,
                        2usize,
                    > as alloy_sol_types::SolType>::tokenize(&self.X),
                    <alloy::sol_types::sol_data::FixedArray<
                        alloy::sol_types::sol_data::Uint<256>,
                        2usize,
                    > as alloy_sol_types::SolType>::tokenize(&self.Y),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encode_packed_to(
                    &tuple, out,
                )
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_packed_encoded_size(
                    &tuple,
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for G2Point {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for G2Point {
            const NAME: &'static str = "G2Point";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed("G2Point(uint256[2] X,uint256[2] Y)")
            }
            #[inline]
            fn eip712_components()
            -> alloy_sol_types::private::Vec<alloy_sol_types::private::Cow<'static, str>>
            {
                alloy_sol_types::private::Vec::new()
            }
            #[inline]
            fn eip712_encode_type() -> alloy_sol_types::private::Cow<'static, str> {
                <Self as alloy_sol_types::SolStruct>::eip712_root_type()
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <alloy::sol_types::sol_data::FixedArray<
                        alloy::sol_types::sol_data::Uint<256>,
                        2usize,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.X)
                    .0,
                    <alloy::sol_types::sol_data::FixedArray<
                        alloy::sol_types::sol_data::Uint<256>,
                        2usize,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.Y)
                    .0,
                ]
                .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for G2Point {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::FixedArray<
                    alloy::sol_types::sol_data::Uint<256>,
                    2usize,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.X)
                    + <alloy::sol_types::sol_data::FixedArray<
                        alloy::sol_types::sol_data::Uint<256>,
                        2usize,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.Y
                    )
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(<Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust));
                <alloy::sol_types::sol_data::FixedArray<
                    alloy::sol_types::sol_data::Uint<256>,
                    2usize,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.X, out
                );
                <alloy::sol_types::sol_data::FixedArray<
                    alloy::sol_types::sol_data::Uint<256>,
                    2usize,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.Y, out
                );
            }
            #[inline]
            fn encode_topic(rust: &Self::RustType) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, &mut out);
                alloy_sol_types::abi::token::WordToken(alloy_sol_types::private::keccak256(out))
            }
        }
    };
    use alloy::contract as alloy_contract;
    /**Creates a new wrapper around an on-chain [`BN254`](self) contract instance.

    See the [wrapper's documentation](`BN254Instance`) for more details.*/
    #[inline]
    pub const fn new<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    >(
        address: alloy_sol_types::private::Address,
        provider: P,
    ) -> BN254Instance<T, P, N> {
        BN254Instance::<T, P, N>::new(address, provider)
    }
    /**A [`BN254`](self) instance.

    Contains type-safe methods for interacting with an on-chain instance of the
    [`BN254`](self) contract located at a given `address`, using a given
    provider `P`.

    If the contract bytecode is available (see the [`sol!`](alloy_sol_types::sol!)
    documentation on how to provide it), the `deploy` and `deploy_builder` methods can
    be used to deploy a new instance of the contract.

    See the [module-level documentation](self) for all the available methods.*/
    #[derive(Clone)]
    pub struct BN254Instance<T, P, N = alloy_contract::private::Ethereum> {
        address: alloy_sol_types::private::Address,
        provider: P,
        _network_transport: ::core::marker::PhantomData<(N, T)>,
    }
    #[automatically_derived]
    impl<T, P, N> ::core::fmt::Debug for BN254Instance<T, P, N> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple("BN254Instance").field(&self.address).finish()
        }
    }
    /// Instantiation and getters/setters.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > BN254Instance<T, P, N>
    {
        /**Creates a new wrapper around an on-chain [`BN254`](self) contract instance.

        See the [wrapper's documentation](`BN254Instance`) for more details.*/
        #[inline]
        pub const fn new(address: alloy_sol_types::private::Address, provider: P) -> Self {
            Self {
                address,
                provider,
                _network_transport: ::core::marker::PhantomData,
            }
        }
        /// Returns a reference to the address.
        #[inline]
        pub const fn address(&self) -> &alloy_sol_types::private::Address {
            &self.address
        }
        /// Sets the address.
        #[inline]
        pub fn set_address(&mut self, address: alloy_sol_types::private::Address) {
            self.address = address;
        }
        /// Sets the address and returns `self`.
        pub fn at(mut self, address: alloy_sol_types::private::Address) -> Self {
            self.set_address(address);
            self
        }
        /// Returns a reference to the provider.
        #[inline]
        pub const fn provider(&self) -> &P {
            &self.provider
        }
    }
    impl<T, P: ::core::clone::Clone, N> BN254Instance<T, &P, N> {
        /// Clones the provider and returns a new instance with the cloned provider.
        #[inline]
        pub fn with_cloned_provider(self) -> BN254Instance<T, P, N> {
            BN254Instance {
                address: self.address,
                provider: ::core::clone::Clone::clone(&self.provider),
                _network_transport: ::core::marker::PhantomData,
            }
        }
    }
    /// Function calls.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > BN254Instance<T, P, N>
    {
        /// Creates a new call builder using this contract instance's provider and address.
        ///
        /// Note that the call can be any function call, not just those defined in this
        /// contract. Prefer using the other methods for building type-safe contract calls.
        pub fn call_builder<C: alloy_sol_types::SolCall>(
            &self,
            call: &C,
        ) -> alloy_contract::SolCallBuilder<T, &P, C, N> {
            alloy_contract::SolCallBuilder::new_sol(&self.provider, &self.address, call)
        }
    }
    /// Event filters.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > BN254Instance<T, P, N>
    {
        /// Creates a new event filter using this contract instance's provider and address.
        ///
        /// Note that the type can be any event, not just those defined in this contract.
        /// Prefer using the other methods for building type-safe event filters.
        pub fn event_filter<E: alloy_sol_types::SolEvent>(
            &self,
        ) -> alloy_contract::Event<T, &P, E, N> {
            alloy_contract::Event::new_sol(&self.provider, &self.address)
        }
    }
}
///Module containing a contract's types and functions.
/**

```solidity
library IBLSSignatureCheckerTypes {
    struct NonSignerStakesAndSignature { uint32[] nonSignerQuorumBitmapIndices; BN254.G1Point[] nonSignerPubkeys; BN254.G1Point[] quorumApks; BN254.G2Point apkG2; BN254.G1Point sigma; uint32[] quorumApkIndices; uint32[] totalStakeIndices; uint32[][] nonSignerStakeIndices; }
    struct QuorumStakeTotals { uint96[] signedStakeForQuorum; uint96[] totalStakeForQuorum; }
}
```*/
#[allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::style,
    clippy::empty_structs_with_brackets
)]
pub mod IBLSSignatureCheckerTypes {
    use super::*;
    use alloy::sol_types as alloy_sol_types;
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
    struct NonSignerStakesAndSignature { uint32[] nonSignerQuorumBitmapIndices; BN254.G1Point[] nonSignerPubkeys; BN254.G1Point[] quorumApks; BN254.G2Point apkG2; BN254.G1Point sigma; uint32[] quorumApkIndices; uint32[] totalStakeIndices; uint32[][] nonSignerStakeIndices; }
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct NonSignerStakesAndSignature {
        #[allow(missing_docs)]
        pub nonSignerQuorumBitmapIndices: alloy::sol_types::private::Vec<u32>,
        #[allow(missing_docs)]
        pub nonSignerPubkeys:
            alloy::sol_types::private::Vec<<BN254::G1Point as alloy::sol_types::SolType>::RustType>,
        #[allow(missing_docs)]
        pub quorumApks:
            alloy::sol_types::private::Vec<<BN254::G1Point as alloy::sol_types::SolType>::RustType>,
        #[allow(missing_docs)]
        pub apkG2: <BN254::G2Point as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub sigma: <BN254::G1Point as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub quorumApkIndices: alloy::sol_types::private::Vec<u32>,
        #[allow(missing_docs)]
        pub totalStakeIndices: alloy::sol_types::private::Vec<u32>,
        #[allow(missing_docs)]
        pub nonSignerStakeIndices:
            alloy::sol_types::private::Vec<alloy::sol_types::private::Vec<u32>>,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (
            alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Uint<32>>,
            alloy::sol_types::sol_data::Array<BN254::G1Point>,
            alloy::sol_types::sol_data::Array<BN254::G1Point>,
            BN254::G2Point,
            BN254::G1Point,
            alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Uint<32>>,
            alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Uint<32>>,
            alloy::sol_types::sol_data::Array<
                alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Uint<32>>,
            >,
        );
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (
            alloy::sol_types::private::Vec<u32>,
            alloy::sol_types::private::Vec<<BN254::G1Point as alloy::sol_types::SolType>::RustType>,
            alloy::sol_types::private::Vec<<BN254::G1Point as alloy::sol_types::SolType>::RustType>,
            <BN254::G2Point as alloy::sol_types::SolType>::RustType,
            <BN254::G1Point as alloy::sol_types::SolType>::RustType,
            alloy::sol_types::private::Vec<u32>,
            alloy::sol_types::private::Vec<u32>,
            alloy::sol_types::private::Vec<alloy::sol_types::private::Vec<u32>>,
        );
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<NonSignerStakesAndSignature> for UnderlyingRustTuple<'_> {
            fn from(value: NonSignerStakesAndSignature) -> Self {
                (
                    value.nonSignerQuorumBitmapIndices,
                    value.nonSignerPubkeys,
                    value.quorumApks,
                    value.apkG2,
                    value.sigma,
                    value.quorumApkIndices,
                    value.totalStakeIndices,
                    value.nonSignerStakeIndices,
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for NonSignerStakesAndSignature {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {
                    nonSignerQuorumBitmapIndices: tuple.0,
                    nonSignerPubkeys: tuple.1,
                    quorumApks: tuple.2,
                    apkG2: tuple.3,
                    sigma: tuple.4,
                    quorumApkIndices: tuple.5,
                    totalStakeIndices: tuple.6,
                    nonSignerStakeIndices: tuple.7,
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for NonSignerStakesAndSignature {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for NonSignerStakesAndSignature {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::SolType>::tokenize(
                        &self.nonSignerQuorumBitmapIndices,
                    ),
                    <alloy::sol_types::sol_data::Array<
                        BN254::G1Point,
                    > as alloy_sol_types::SolType>::tokenize(&self.nonSignerPubkeys),
                    <alloy::sol_types::sol_data::Array<
                        BN254::G1Point,
                    > as alloy_sol_types::SolType>::tokenize(&self.quorumApks),
                    <BN254::G2Point as alloy_sol_types::SolType>::tokenize(&self.apkG2),
                    <BN254::G1Point as alloy_sol_types::SolType>::tokenize(&self.sigma),
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::SolType>::tokenize(&self.quorumApkIndices),
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::SolType>::tokenize(&self.totalStakeIndices),
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Array<
                            alloy::sol_types::sol_data::Uint<32>,
                        >,
                    > as alloy_sol_types::SolType>::tokenize(&self.nonSignerStakeIndices),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encode_packed_to(
                    &tuple, out,
                )
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_packed_encoded_size(
                    &tuple,
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for NonSignerStakesAndSignature {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for NonSignerStakesAndSignature {
            const NAME: &'static str = "NonSignerStakesAndSignature";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed(
                    "NonSignerStakesAndSignature(uint32[] nonSignerQuorumBitmapIndices,BN254.G1Point[] nonSignerPubkeys,BN254.G1Point[] quorumApks,BN254.G2Point apkG2,BN254.G1Point sigma,uint32[] quorumApkIndices,uint32[] totalStakeIndices,uint32[][] nonSignerStakeIndices)",
                )
            }
            #[inline]
            fn eip712_components()
            -> alloy_sol_types::private::Vec<alloy_sol_types::private::Cow<'static, str>>
            {
                let mut components = alloy_sol_types::private::Vec::with_capacity(4);
                components.push(<BN254::G1Point as alloy_sol_types::SolStruct>::eip712_root_type());
                components
                    .extend(<BN254::G1Point as alloy_sol_types::SolStruct>::eip712_components());
                components.push(<BN254::G1Point as alloy_sol_types::SolStruct>::eip712_root_type());
                components
                    .extend(<BN254::G1Point as alloy_sol_types::SolStruct>::eip712_components());
                components.push(<BN254::G2Point as alloy_sol_types::SolStruct>::eip712_root_type());
                components
                    .extend(<BN254::G2Point as alloy_sol_types::SolStruct>::eip712_components());
                components.push(<BN254::G1Point as alloy_sol_types::SolStruct>::eip712_root_type());
                components
                    .extend(<BN254::G1Point as alloy_sol_types::SolStruct>::eip712_components());
                components
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::SolType>::eip712_data_word(
                            &self.nonSignerQuorumBitmapIndices,
                        )
                        .0,
                    <alloy::sol_types::sol_data::Array<
                        BN254::G1Point,
                    > as alloy_sol_types::SolType>::eip712_data_word(
                            &self.nonSignerPubkeys,
                        )
                        .0,
                    <alloy::sol_types::sol_data::Array<
                        BN254::G1Point,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.quorumApks)
                        .0,
                    <BN254::G2Point as alloy_sol_types::SolType>::eip712_data_word(
                            &self.apkG2,
                        )
                        .0,
                    <BN254::G1Point as alloy_sol_types::SolType>::eip712_data_word(
                            &self.sigma,
                        )
                        .0,
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::SolType>::eip712_data_word(
                            &self.quorumApkIndices,
                        )
                        .0,
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::SolType>::eip712_data_word(
                            &self.totalStakeIndices,
                        )
                        .0,
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Array<
                            alloy::sol_types::sol_data::Uint<32>,
                        >,
                    > as alloy_sol_types::SolType>::eip712_data_word(
                            &self.nonSignerStakeIndices,
                        )
                        .0,
                ]
                    .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for NonSignerStakesAndSignature {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.nonSignerQuorumBitmapIndices,
                    )
                    + <alloy::sol_types::sol_data::Array<
                        BN254::G1Point,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.nonSignerPubkeys,
                    )
                    + <alloy::sol_types::sol_data::Array<
                        BN254::G1Point,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.quorumApks,
                    )
                    + <BN254::G2Point as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.apkG2,
                    )
                    + <BN254::G1Point as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.sigma,
                    )
                    + <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.quorumApkIndices,
                    )
                    + <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<32>,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.totalStakeIndices,
                    )
                    + <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Array<
                            alloy::sol_types::sol_data::Uint<32>,
                        >,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.nonSignerStakeIndices,
                    )
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(<Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust));
                <alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::Uint<32>,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.nonSignerQuorumBitmapIndices,
                    out,
                );
                <alloy::sol_types::sol_data::Array<
                    BN254::G1Point,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.nonSignerPubkeys,
                    out,
                );
                <alloy::sol_types::sol_data::Array<
                    BN254::G1Point,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.quorumApks,
                    out,
                );
                <BN254::G2Point as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.apkG2,
                    out,
                );
                <BN254::G1Point as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.sigma,
                    out,
                );
                <alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::Uint<32>,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.quorumApkIndices,
                    out,
                );
                <alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::Uint<32>,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.totalStakeIndices,
                    out,
                );
                <alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Uint<32>>,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.nonSignerStakeIndices,
                    out,
                );
            }
            #[inline]
            fn encode_topic(rust: &Self::RustType) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, &mut out);
                alloy_sol_types::abi::token::WordToken(alloy_sol_types::private::keccak256(out))
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
    struct QuorumStakeTotals { uint96[] signedStakeForQuorum; uint96[] totalStakeForQuorum; }
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct QuorumStakeTotals {
        #[allow(missing_docs)]
        pub signedStakeForQuorum:
            alloy::sol_types::private::Vec<alloy::sol_types::private::primitives::aliases::U96>,
        #[allow(missing_docs)]
        pub totalStakeForQuorum:
            alloy::sol_types::private::Vec<alloy::sol_types::private::primitives::aliases::U96>,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (
            alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Uint<96>>,
            alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Uint<96>>,
        );
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (
            alloy::sol_types::private::Vec<alloy::sol_types::private::primitives::aliases::U96>,
            alloy::sol_types::private::Vec<alloy::sol_types::private::primitives::aliases::U96>,
        );
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<QuorumStakeTotals> for UnderlyingRustTuple<'_> {
            fn from(value: QuorumStakeTotals) -> Self {
                (value.signedStakeForQuorum, value.totalStakeForQuorum)
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for QuorumStakeTotals {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {
                    signedStakeForQuorum: tuple.0,
                    totalStakeForQuorum: tuple.1,
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for QuorumStakeTotals {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for QuorumStakeTotals {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<96>,
                    > as alloy_sol_types::SolType>::tokenize(&self.signedStakeForQuorum),
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<96>,
                    > as alloy_sol_types::SolType>::tokenize(&self.totalStakeForQuorum),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_encode_packed_to(
                    &tuple, out,
                )
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple =
                    <UnderlyingRustTuple<'_> as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::abi_packed_encoded_size(
                    &tuple,
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for QuorumStakeTotals {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> =
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for QuorumStakeTotals {
            const NAME: &'static str = "QuorumStakeTotals";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed(
                    "QuorumStakeTotals(uint96[] signedStakeForQuorum,uint96[] totalStakeForQuorum)",
                )
            }
            #[inline]
            fn eip712_components()
            -> alloy_sol_types::private::Vec<alloy_sol_types::private::Cow<'static, str>>
            {
                alloy_sol_types::private::Vec::new()
            }
            #[inline]
            fn eip712_encode_type() -> alloy_sol_types::private::Cow<'static, str> {
                <Self as alloy_sol_types::SolStruct>::eip712_root_type()
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<96>,
                    > as alloy_sol_types::SolType>::eip712_data_word(
                            &self.signedStakeForQuorum,
                        )
                        .0,
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<96>,
                    > as alloy_sol_types::SolType>::eip712_data_word(
                            &self.totalStakeForQuorum,
                        )
                        .0,
                ]
                    .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for QuorumStakeTotals {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<96>,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.signedStakeForQuorum,
                    )
                    + <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Uint<96>,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.totalStakeForQuorum,
                    )
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(<Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust));
                <alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::Uint<96>,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.signedStakeForQuorum,
                    out,
                );
                <alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::Uint<96>,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.totalStakeForQuorum,
                    out,
                );
            }
            #[inline]
            fn encode_topic(rust: &Self::RustType) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, &mut out);
                alloy_sol_types::abi::token::WordToken(alloy_sol_types::private::keccak256(out))
            }
        }
    };
    use alloy::contract as alloy_contract;
    /**Creates a new wrapper around an on-chain [`IBLSSignatureCheckerTypes`](self) contract instance.

    See the [wrapper's documentation](`IBLSSignatureCheckerTypesInstance`) for more details.*/
    #[inline]
    pub const fn new<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    >(
        address: alloy_sol_types::private::Address,
        provider: P,
    ) -> IBLSSignatureCheckerTypesInstance<T, P, N> {
        IBLSSignatureCheckerTypesInstance::<T, P, N>::new(address, provider)
    }
    /**A [`IBLSSignatureCheckerTypes`](self) instance.

    Contains type-safe methods for interacting with an on-chain instance of the
    [`IBLSSignatureCheckerTypes`](self) contract located at a given `address`, using a given
    provider `P`.

    If the contract bytecode is available (see the [`sol!`](alloy_sol_types::sol!)
    documentation on how to provide it), the `deploy` and `deploy_builder` methods can
    be used to deploy a new instance of the contract.

    See the [module-level documentation](self) for all the available methods.*/
    #[derive(Clone)]
    pub struct IBLSSignatureCheckerTypesInstance<T, P, N = alloy_contract::private::Ethereum> {
        address: alloy_sol_types::private::Address,
        provider: P,
        _network_transport: ::core::marker::PhantomData<(N, T)>,
    }
    #[automatically_derived]
    impl<T, P, N> ::core::fmt::Debug for IBLSSignatureCheckerTypesInstance<T, P, N> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple("IBLSSignatureCheckerTypesInstance")
                .field(&self.address)
                .finish()
        }
    }
    /// Instantiation and getters/setters.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > IBLSSignatureCheckerTypesInstance<T, P, N>
    {
        /**Creates a new wrapper around an on-chain [`IBLSSignatureCheckerTypes`](self) contract instance.

        See the [wrapper's documentation](`IBLSSignatureCheckerTypesInstance`) for more details.*/
        #[inline]
        pub const fn new(address: alloy_sol_types::private::Address, provider: P) -> Self {
            Self {
                address,
                provider,
                _network_transport: ::core::marker::PhantomData,
            }
        }
        /// Returns a reference to the address.
        #[inline]
        pub const fn address(&self) -> &alloy_sol_types::private::Address {
            &self.address
        }
        /// Sets the address.
        #[inline]
        pub fn set_address(&mut self, address: alloy_sol_types::private::Address) {
            self.address = address;
        }
        /// Sets the address and returns `self`.
        pub fn at(mut self, address: alloy_sol_types::private::Address) -> Self {
            self.set_address(address);
            self
        }
        /// Returns a reference to the provider.
        #[inline]
        pub const fn provider(&self) -> &P {
            &self.provider
        }
    }
    impl<T, P: ::core::clone::Clone, N> IBLSSignatureCheckerTypesInstance<T, &P, N> {
        /// Clones the provider and returns a new instance with the cloned provider.
        #[inline]
        pub fn with_cloned_provider(self) -> IBLSSignatureCheckerTypesInstance<T, P, N> {
            IBLSSignatureCheckerTypesInstance {
                address: self.address,
                provider: ::core::clone::Clone::clone(&self.provider),
                _network_transport: ::core::marker::PhantomData,
            }
        }
    }
    /// Function calls.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > IBLSSignatureCheckerTypesInstance<T, P, N>
    {
        /// Creates a new call builder using this contract instance's provider and address.
        ///
        /// Note that the call can be any function call, not just those defined in this
        /// contract. Prefer using the other methods for building type-safe contract calls.
        pub fn call_builder<C: alloy_sol_types::SolCall>(
            &self,
            call: &C,
        ) -> alloy_contract::SolCallBuilder<T, &P, C, N> {
            alloy_contract::SolCallBuilder::new_sol(&self.provider, &self.address, call)
        }
    }
    /// Event filters.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > IBLSSignatureCheckerTypesInstance<T, P, N>
    {
        /// Creates a new event filter using this contract instance's provider and address.
        ///
        /// Note that the type can be any event, not just those defined in this contract.
        /// Prefer using the other methods for building type-safe event filters.
        pub fn event_filter<E: alloy_sol_types::SolEvent>(
            &self,
        ) -> alloy_contract::Event<T, &P, E, N> {
            alloy_contract::Event::new_sol(&self.provider, &self.address)
        }
    }
}
/**

Generated by the following Solidity interface...
```solidity
library BN254 {
    struct G1Point {
        uint256 X;
        uint256 Y;
    }
    struct G2Point {
        uint256[2] X;
        uint256[2] Y;
    }
}

library IBLSSignatureCheckerTypes {
    struct NonSignerStakesAndSignature {
        uint32[] nonSignerQuorumBitmapIndices;
        BN254.G1Point[] nonSignerPubkeys;
        BN254.G1Point[] quorumApks;
        BN254.G2Point apkG2;
        BN254.G1Point sigma;
        uint32[] quorumApkIndices;
        uint32[] totalStakeIndices;
        uint32[][] nonSignerStakeIndices;
    }
    struct QuorumStakeTotals {
        uint96[] signedStakeForQuorum;
        uint96[] totalStakeForQuorum;
    }
}

interface BLSSignatureChecker {
    error BitmapValueTooLarge();
    error BytesArrayLengthTooLong();
    error BytesArrayNotOrdered();
    error ECAddFailed();
    error ECMulFailed();
    error ExpModFailed();
    error InputArrayLengthMismatch();
    error InputEmptyQuorumNumbers();
    error InputNonSignerLengthMismatch();
    error InvalidBLSPairingKey();
    error InvalidBLSSignature();
    error InvalidQuorumApkHash();
    error InvalidReferenceBlocknumber();
    error NonSignerPubkeysNotSorted();
    error OnlyRegistryCoordinatorOwner();
    error ScalarTooLarge();

    constructor(address _registryCoordinator);

    function blsApkRegistry() external view returns (address);
    function checkSignatures(bytes32 msgHash, bytes memory quorumNumbers, uint32 referenceBlockNumber, IBLSSignatureCheckerTypes.NonSignerStakesAndSignature memory params) external view returns (IBLSSignatureCheckerTypes.QuorumStakeTotals memory, bytes32);
    function delegation() external view returns (address);
    function registryCoordinator() external view returns (address);
    function stakeRegistry() external view returns (address);
    function trySignatureAndApkVerification(bytes32 msgHash, BN254.G1Point memory apk, BN254.G2Point memory apkG2, BN254.G1Point memory sigma) external view returns (bool pairingSuccessful, bool siganatureIsValid);
}
```

...which was generated by the following JSON ABI:
```json
[
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "_registryCoordinator",
        "type": "address",
        "internalType": "contract ISlashingRegistryCoordinator"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "blsApkRegistry",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IBLSApkRegistry"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "checkSignatures",
    "inputs": [
      {
        "name": "msgHash",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "quorumNumbers",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "referenceBlockNumber",
        "type": "uint32",
        "internalType": "uint32"
      },
      {
        "name": "params",
        "type": "tuple",
        "internalType": "struct IBLSSignatureCheckerTypes.NonSignerStakesAndSignature",
        "components": [
          {
            "name": "nonSignerQuorumBitmapIndices",
            "type": "uint32[]",
            "internalType": "uint32[]"
          },
          {
            "name": "nonSignerPubkeys",
            "type": "tuple[]",
            "internalType": "struct BN254.G1Point[]",
            "components": [
              {
                "name": "X",
                "type": "uint256",
                "internalType": "uint256"
              },
              {
                "name": "Y",
                "type": "uint256",
                "internalType": "uint256"
              }
            ]
          },
          {
            "name": "quorumApks",
            "type": "tuple[]",
            "internalType": "struct BN254.G1Point[]",
            "components": [
              {
                "name": "X",
                "type": "uint256",
                "internalType": "uint256"
              },
              {
                "name": "Y",
                "type": "uint256",
                "internalType": "uint256"
              }
            ]
          },
          {
            "name": "apkG2",
            "type": "tuple",
            "internalType": "struct BN254.G2Point",
            "components": [
              {
                "name": "X",
                "type": "uint256[2]",
                "internalType": "uint256[2]"
              },
              {
                "name": "Y",
                "type": "uint256[2]",
                "internalType": "uint256[2]"
              }
            ]
          },
          {
            "name": "sigma",
            "type": "tuple",
            "internalType": "struct BN254.G1Point",
            "components": [
              {
                "name": "X",
                "type": "uint256",
                "internalType": "uint256"
              },
              {
                "name": "Y",
                "type": "uint256",
                "internalType": "uint256"
              }
            ]
          },
          {
            "name": "quorumApkIndices",
            "type": "uint32[]",
            "internalType": "uint32[]"
          },
          {
            "name": "totalStakeIndices",
            "type": "uint32[]",
            "internalType": "uint32[]"
          },
          {
            "name": "nonSignerStakeIndices",
            "type": "uint32[][]",
            "internalType": "uint32[][]"
          }
        ]
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct IBLSSignatureCheckerTypes.QuorumStakeTotals",
        "components": [
          {
            "name": "signedStakeForQuorum",
            "type": "uint96[]",
            "internalType": "uint96[]"
          },
          {
            "name": "totalStakeForQuorum",
            "type": "uint96[]",
            "internalType": "uint96[]"
          }
        ]
      },
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "delegation",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IDelegationManager"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "registryCoordinator",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract ISlashingRegistryCoordinator"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "stakeRegistry",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IStakeRegistry"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "trySignatureAndApkVerification",
    "inputs": [
      {
        "name": "msgHash",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "apk",
        "type": "tuple",
        "internalType": "struct BN254.G1Point",
        "components": [
          {
            "name": "X",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "Y",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      },
      {
        "name": "apkG2",
        "type": "tuple",
        "internalType": "struct BN254.G2Point",
        "components": [
          {
            "name": "X",
            "type": "uint256[2]",
            "internalType": "uint256[2]"
          },
          {
            "name": "Y",
            "type": "uint256[2]",
            "internalType": "uint256[2]"
          }
        ]
      },
      {
        "name": "sigma",
        "type": "tuple",
        "internalType": "struct BN254.G1Point",
        "components": [
          {
            "name": "X",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "Y",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      }
    ],
    "outputs": [
      {
        "name": "pairingSuccessful",
        "type": "bool",
        "internalType": "bool"
      },
      {
        "name": "siganatureIsValid",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "BitmapValueTooLarge",
    "inputs": []
  },
  {
    "type": "error",
    "name": "BytesArrayLengthTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "BytesArrayNotOrdered",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ECAddFailed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ECMulFailed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ExpModFailed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InputArrayLengthMismatch",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InputEmptyQuorumNumbers",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InputNonSignerLengthMismatch",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidBLSPairingKey",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidBLSSignature",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidQuorumApkHash",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidReferenceBlocknumber",
    "inputs": []
  },
  {
    "type": "error",
    "name": "NonSignerPubkeysNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "OnlyRegistryCoordinatorOwner",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ScalarTooLarge",
    "inputs": []
  }
]
```*/
#[allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::style,
    clippy::empty_structs_with_brackets
)]
pub mod BLSSignatureChecker {
    use super::*;
    use alloy::sol_types as alloy_sol_types;
    /// The creation / init bytecode of the contract.
    ///
    /// ```text
    ///0x610100604052348015610010575f5ffd5b506040516131d33803806131d3833981810160405281019061003291906102bf565b808073ffffffffffffffffffffffffffffffffffffffff1660808173ffffffffffffffffffffffffffffffffffffffff16815250508073ffffffffffffffffffffffffffffffffffffffff1663683048356040518163ffffffff1660e01b8152600401602060405180830381865afa1580156100b0573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906100d49190610325565b73ffffffffffffffffffffffffffffffffffffffff1660a08173ffffffffffffffffffffffffffffffffffffffff16815250508073ffffffffffffffffffffffffffffffffffffffff16635df459466040518163ffffffff1660e01b8152600401602060405180830381865afa158015610150573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610174919061038b565b73ffffffffffffffffffffffffffffffffffffffff1660c08173ffffffffffffffffffffffffffffffffffffffff168152505060a05173ffffffffffffffffffffffffffffffffffffffff1663df5cf7236040518163ffffffff1660e01b8152600401602060405180830381865afa1580156101f2573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061021691906103f1565b73ffffffffffffffffffffffffffffffffffffffff1660e08173ffffffffffffffffffffffffffffffffffffffff1681525050505061041c565b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f61027d82610254565b9050919050565b5f61028e82610273565b9050919050565b61029e81610284565b81146102a8575f5ffd5b50565b5f815190506102b981610295565b92915050565b5f602082840312156102d4576102d3610250565b5b5f6102e1848285016102ab565b91505092915050565b5f6102f482610273565b9050919050565b610304816102ea565b811461030e575f5ffd5b50565b5f8151905061031f816102fb565b92915050565b5f6020828403121561033a57610339610250565b5b5f61034784828501610311565b91505092915050565b5f61035a82610273565b9050919050565b61036a81610350565b8114610374575f5ffd5b50565b5f8151905061038581610361565b92915050565b5f602082840312156103a05761039f610250565b5b5f6103ad84828501610377565b91505092915050565b5f6103c082610273565b9050919050565b6103d0816103b6565b81146103da575f5ffd5b50565b5f815190506103eb816103c7565b92915050565b5f6020828403121561040657610405610250565b5b5f610413848285016103dd565b91505092915050565b60805160a05160c05160e051612d616104725f395f610eeb01525f81816102a3015261093d01525f81816102c701528181610abd0152610c9701525f81816102eb0152818161061701526107940152612d615ff3fe608060405234801561000f575f5ffd5b5060043610610060575f3560e01c8063171f1d5b146100645780635df459461461009557806368304835146100b35780636d14a987146100d15780636efb4636146100ef578063df5cf72314610120575b5f5ffd5b61007e60048036038101906100799190611e8f565b61013e565b60405161008c929190611f0e565b60405180910390f35b61009d6102a1565b6040516100aa9190611faf565b60405180910390f35b6100bb6102c5565b6040516100c89190611fe8565b60405180910390f35b6100d96102e9565b6040516100e69190612021565b60405180910390f35b6101096004803603810190610104919061249c565b61030d565b60405161011792919061265a565b60405180910390f35b610128610ee9565b60405161013591906126a8565b60405180910390f35b5f5f5f7f30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f000000187875f01518860200151885f01515f60028110610182576101816126c1565b5b6020020151895f015160016002811061019e5761019d6126c1565b5b60200201518a602001515f600281106101ba576101b96126c1565b5b60200201518b602001516001600281106101d7576101d66126c1565b5b60200201518b5f01518c602001516040516020016101fd9998979695949392919061272e565b604051602081830303815290604052805190602001205f1c61021f91906127fd565b905061028f61024961023a8389610f0d90919063ffffffff16565b86610fd890919063ffffffff16565b6102516110c8565b61028561026e85610260611192565b610f0d90919063ffffffff16565b6102778c6111b6565b610fd890919063ffffffff16565b886201d4c06112c1565b80935081945050505094509492505050565b7f000000000000000000000000000000000000000000000000000000000000000081565b7f000000000000000000000000000000000000000000000000000000000000000081565b7f000000000000000000000000000000000000000000000000000000000000000081565b610315611af9565b5f5f8686905003610352576040517f1f0405a000000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b8260400151518686905014801561037057508260a001515186869050145b801561038357508260c001515186869050145b801561039657508260e001515186869050145b6103cc576040517f43714afd00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b825f0151518360200151511461040e576040517f5f832f4100000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b4363ffffffff168463ffffffff1610610453576040517f4b874f4500000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b5f60405180604001604052805f81526020015f8152509050610473611af9565b8787905067ffffffffffffffff8111156104905761048f611c90565b5b6040519080825280602002602001820160405280156104be5781602001602082028036833780820191505090505b5081602001819052508787905067ffffffffffffffff8111156104e4576104e3611c90565b5b6040519080825280602002602001820160405280156105125781602001602082028036833780820191505090505b50815f0181905250610522611b13565b85602001515167ffffffffffffffff81111561054157610540611c90565b5b60405190808252806020026020018201604052801561056f5781602001602082028036833780820191505090505b50815f018190525085602001515167ffffffffffffffff81111561059657610595611c90565b5b6040519080825280602002602001820160405280156105c45781602001602082028036833780820191505090505b5081602001819052505f6106a78a8a8080601f0160208091040260200160405190810160405280939291908181526020018383808284375f81840152601f19601f820116905080830192505050505050507f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff16639aa1653d6040518163ffffffff1660e01b8152600401602060405180830381865afa15801561067e573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906106a29190612863565b611561565b90505f5f90505b87602001515181101561091e576106e2886020015182815181106106d5576106d46126c1565b5b60200260200101516115b8565b836020015182815181106106f9576106f86126c1565b5b6020026020010181815250505f811461079257826020015160018261071e91906128bb565b8151811061072f5761072e6126c1565b5b60200260200101515f1c836020015182815181106107505761074f6126c1565b5b60200260200101515f1c11610791576040517fff71941400000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b5b7f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff166304ec6351846020015183815181106107e5576107e46126c1565b5b60200260200101518b8b5f01518581518110610804576108036126c1565b5b60200260200101516040518463ffffffff1660e01b815260040161082a9392919061292d565b602060405180830381865afa158015610845573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061086991906129af565b77ffffffffffffffffffffffffffffffffffffffffffffffff16835f01518281518110610899576108986126c1565b5b60200260200101818152505061090f6109006108d384865f015185815181106108c5576108c46126c1565b5b6020026020010151166115d0565b8a6020015184815181106108ea576108e96126c1565b5b602002602001015161160b90919063ffffffff16565b86610fd890919063ffffffff16565b945080806001019150506106ae565b5050610929836116f0565b92505f5f90505b89899050811015610e1a577f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff166368bccaac8b8b8481811061098a576109896126c1565b5b9050013560f81c60f81b60f81c8a8a60a0015185815181106109af576109ae6126c1565b5b60200260200101516040518463ffffffff1660e01b81526004016109d5939291906129e9565b602060405180830381865afa1580156109f0573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610a149190612a73565b67ffffffffffffffff1916610a4688604001518381518110610a3957610a386126c1565b5b60200260200101516115b8565b67ffffffffffffffff191614610a88576040517fe1310aed00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b610ab987604001518281518110610aa257610aa16126c1565b5b602002602001015185610fd890919063ffffffff16565b93507f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff1663c8294c568b8b84818110610b0a57610b096126c1565b5b9050013560f81c60f81b60f81c8a8a60c001518581518110610b2f57610b2e6126c1565b5b60200260200101516040518463ffffffff1660e01b8152600401610b55939291906129e9565b602060405180830381865afa158015610b70573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610b949190612ac8565b83602001518281518110610bab57610baa6126c1565b5b60200260200101906bffffffffffffffffffffffff1690816bffffffffffffffffffffffff168152505082602001518181518110610bec57610beb6126c1565b5b6020026020010151835f01518281518110610c0a57610c096126c1565b5b60200260200101906bffffffffffffffffffffffff1690816bffffffffffffffffffffffff16815250505f5f90505f5f90505b886020015151811015610e0b57610c90845f01518281518110610c6357610c626126c1565b5b60200260200101518d8d86818110610c7e57610c7d6126c1565b5b9050013560f81c60f81b60f81c6117a8565b15610dfe577f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff1663f2be94ae8d8d86818110610ce457610ce36126c1565b5b9050013560f81c60f81b60f81c8c87602001518581518110610d0957610d086126c1565b5b60200260200101518d60e001518881518110610d2857610d276126c1565b5b60200260200101518781518110610d4257610d416126c1565b5b60200260200101516040518563ffffffff1660e01b8152600401610d699493929190612af3565b602060405180830381865afa158015610d84573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610da89190612ac8565b855f01518481518110610dbe57610dbd6126c1565b5b60200260200101818151610dd29190612b36565b9150906bffffffffffffffffffffffff1690816bffffffffffffffffffffffff16815250508160010191505b8080600101915050610c3d565b50508080600101915050610930565b505f5f610e318c868a606001518b6080015161013e565b9150915081610e6c576040517f67988d3300000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b80610ea3576040517fab1b236b00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b50505f878260200151604051602001610ebd929190612c5a565b604051602081830303815290604052805190602001209050828195509550505050509550959350505050565b7f000000000000000000000000000000000000000000000000000000000000000081565b610f15611b2d565b610f1d611b45565b835f0151815f60038110610f3457610f336126c1565b5b602002018181525050836020015181600160038110610f5657610f556126c1565b5b6020020181815250508281600260038110610f7457610f736126c1565b5b6020020181815250505f60408360608460076107d05a03fa9050805f8103610f9857fe5b5080610fd0576040517f4633be3200000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b505092915050565b610fe0611b2d565b610fe8611b67565b835f0151815f60048110610fff57610ffe6126c1565b5b602002018181525050836020015181600160048110611021576110206126c1565b5b602002018181525050825f015181600260048110611042576110416126c1565b5b602002018181525050826020015181600360048110611064576110636126c1565b5b6020020181815250505f60408360808460066107d05a03fa9050805f810361108857fe5b50806110c0576040517fd4b68fd700000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b505092915050565b6110d0611b89565b604051806040016040528060405180604001604052807f198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c281526020017f1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed815250815260200160405180604001604052807f275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec81526020017f1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d815250815250905090565b61119a611b2d565b6040518060400160405280600181526020016002815250905090565b6111be611b2d565b5f5f90505f5f90505f7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47855f1c6111f591906127fd565b90505b6001156112a157611208816117be565b80935081945050507f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd478061123f5761123e6127d0565b5b82830983036112675760405180604001604052808281526020018381525093505050506112bc565b7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4780611296576112956127d0565b5b6001820890506111f8565b60405180604001604052805f81526020015f81525093505050505b919050565b5f5f5f60405180604001604052808981526020018781525090505f60405180604001604052808981526020018781525090506112fb611baf565b5f5f90505b6002811015611519575f6006826113179190612c81565b905084826002811061132c5761132b6126c1565b5b60200201515f0151835f836113419190612cc2565b600c8110611352576113516126c1565b5b60200201818152505084826002811061136e5761136d6126c1565b5b602002015160200151836001836113859190612cc2565b600c8110611396576113956126c1565b5b6020020181815250508382600281106113b2576113b16126c1565b5b60200201515f01515f600281106113cc576113cb6126c1565b5b6020020151836002836113df9190612cc2565b600c81106113f0576113ef6126c1565b5b60200201818152505083826002811061140c5761140b6126c1565b5b60200201515f0151600160028110611427576114266126c1565b5b60200201518360038361143a9190612cc2565b600c811061144b5761144a6126c1565b5b602002018181525050838260028110611467576114666126c1565b5b6020020151602001515f60028110611482576114816126c1565b5b6020020151836004836114959190612cc2565b600c81106114a6576114a56126c1565b5b6020020181815250508382600281106114c2576114c16126c1565b5b6020020151602001516001600281106114de576114dd6126c1565b5b6020020151836005836114f19190612cc2565b600c8110611502576115016126c1565b5b602002018181525050508080600101915050611300565b50611522611bd2565b5f6020826020600c028560088cfa9050805f835f60018110611547576115466126c1565b5b602002015114159650965050505050509550959350505050565b5f5f61156c846118b3565b9050808360ff166001901b116115ae576040517fca95733300000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b8091505092915050565b5f81515f52816020015160205260405f209050919050565b5f5f5f90505b5f831115611602576001836115eb91906128bb565b8316925080806115fa90612d02565b9150506115d6565b80915050919050565b611613611b2d565b6102008261ffff1610611652576040517fff89d4fa00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b60018261ffff1603611666578290506116ea565b5f60405180604001604052805f81526020015f81525090505f8490505f600190505f5f90505b8161ffff168661ffff16106116e2576001808260ff168861ffff16901c1661ffff16036116c0576116bd8484610fd8565b93505b6116ca8384610fd8565b925060018261ffff16901b915080600101905061168c565b839450505050505b92915050565b6116f8611b2d565b5f825f015114801561170d57505f8260200151145b1561172e5760405180604001604052805f81526020015f81525090506117a3565b6040518060400160405280835f015181526020017f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47846020015161177291906127fd565b7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4761179d91906128bb565b81525090505b919050565b5f60018260ff1684901c16600114905092915050565b5f5f5f7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47806117f0576117ef6127d0565b5b60037f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4780611821576118206127d0565b5b867f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4780611851576118506127d0565b5b888909090890505f6118a4827f0c19139cb84c680a6e14116da060561765e05aa45a1c72a34f082305b61f3f527f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd476119bb565b90508181935093505050915091565b5f610100825111156118f1576040517ffb4a9c8e00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b5f825103611901575f90506119b6565b5f5f835f81518110611916576119156126c1565b5b602001015160f81c60f81b60f81c60ff166001901b91505f600190505b84518110156119af5784818151811061194f5761194e6126c1565b5b602001015160f81c60f81b60f81c60ff166001901b915082821161199f576040517f80c8834800000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b8183179250806001019050611933565b5081925050505b919050565b5f5f6119c5611bd2565b6119cd611bf4565b6020815f600681106119e2576119e16126c1565b5b602002018181525050602081600160068110611a0157611a006126c1565b5b602002018181525050602081600260068110611a2057611a1f6126c1565b5b6020020181815250508681600360068110611a3e57611a3d6126c1565b5b6020020181815250508581600460068110611a5c57611a5b6126c1565b5b6020020181815250508481600560068110611a7a57611a796126c1565b5b60200201818152505060208260c08360056107d05a03fa9250825f8103611a9d57fe5b5082611ad5576040517fd51edae300000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b815f60018110611ae857611ae76126c1565b5b602002015193505050509392505050565b604051806040016040528060608152602001606081525090565b604051806040016040528060608152602001606081525090565b60405180604001604052805f81526020015f81525090565b6040518060600160405280600390602082028036833780820191505090505090565b6040518060800160405280600490602082028036833780820191505090505090565b6040518060400160405280611b9c611c16565b8152602001611ba9611c16565b81525090565b604051806101800160405280600c90602082028036833780820191505090505090565b6040518060200160405280600190602082028036833780820191505090505090565b6040518060c00160405280600690602082028036833780820191505090505090565b6040518060400160405280600290602082028036833780820191505090505090565b5f604051905090565b5f5ffd5b5f5ffd5b5f819050919050565b611c5b81611c49565b8114611c65575f5ffd5b50565b5f81359050611c7681611c52565b92915050565b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b611cc682611c80565b810181811067ffffffffffffffff82111715611ce557611ce4611c90565b5b80604052505050565b5f611cf7611c38565b9050611d038282611cbd565b919050565b5f5ffd5b5f819050919050565b611d1e81611d0c565b8114611d28575f5ffd5b50565b5f81359050611d3981611d15565b92915050565b5f60408284031215611d5457611d53611c7c565b5b611d5e6040611cee565b90505f611d6d84828501611d2b565b5f830152506020611d8084828501611d2b565b60208301525092915050565b5f5ffd5b5f67ffffffffffffffff821115611daa57611da9611c90565b5b602082029050919050565b5f5ffd5b5f611dcb611dc684611d90565b611cee565b90508060208402830185811115611de557611de4611db5565b5b835b81811015611e0e5780611dfa8882611d2b565b845260208401935050602081019050611de7565b5050509392505050565b5f82601f830112611e2c57611e2b611d8c565b5b6002611e39848285611db9565b91505092915050565b5f60808284031215611e5757611e56611c7c565b5b611e616040611cee565b90505f611e7084828501611e18565b5f830152506040611e8384828501611e18565b60208301525092915050565b5f5f5f5f6101208587031215611ea857611ea7611c41565b5b5f611eb587828801611c68565b9450506020611ec687828801611d3f565b9350506060611ed787828801611e42565b92505060e0611ee887828801611d3f565b91505092959194509250565b5f8115159050919050565b611f0881611ef4565b82525050565b5f604082019050611f215f830185611eff565b611f2e6020830184611eff565b9392505050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f819050919050565b5f611f77611f72611f6d84611f35565b611f54565b611f35565b9050919050565b5f611f8882611f5d565b9050919050565b5f611f9982611f7e565b9050919050565b611fa981611f8f565b82525050565b5f602082019050611fc25f830184611fa0565b92915050565b5f611fd282611f7e565b9050919050565b611fe281611fc8565b82525050565b5f602082019050611ffb5f830184611fd9565b92915050565b5f61200b82611f7e565b9050919050565b61201b81612001565b82525050565b5f6020820190506120345f830184612012565b92915050565b5f5ffd5b5f5f83601f84011261205357612052611d8c565b5b8235905067ffffffffffffffff8111156120705761206f61203a565b5b60208301915083600182028301111561208c5761208b611db5565b5b9250929050565b5f63ffffffff82169050919050565b6120ab81612093565b81146120b5575f5ffd5b50565b5f813590506120c6816120a2565b92915050565b5f67ffffffffffffffff8211156120e6576120e5611c90565b5b602082029050602081019050919050565b5f612109612104846120cc565b611cee565b9050808382526020820190506020840283018581111561212c5761212b611db5565b5b835b81811015612155578061214188826120b8565b84526020840193505060208101905061212e565b5050509392505050565b5f82601f83011261217357612172611d8c565b5b81356121838482602086016120f7565b91505092915050565b5f67ffffffffffffffff8211156121a6576121a5611c90565b5b602082029050602081019050919050565b5f6121c96121c48461218c565b611cee565b905080838252602082019050604084028301858111156121ec576121eb611db5565b5b835b8181101561221557806122018882611d3f565b8452602084019350506040810190506121ee565b5050509392505050565b5f82601f83011261223357612232611d8c565b5b81356122438482602086016121b7565b91505092915050565b5f67ffffffffffffffff82111561226657612265611c90565b5b602082029050602081019050919050565b5f6122896122848461224c565b611cee565b905080838252602082019050602084028301858111156122ac576122ab611db5565b5b835b818110156122f357803567ffffffffffffffff8111156122d1576122d0611d8c565b5b8086016122de898261215f565b855260208501945050506020810190506122ae565b5050509392505050565b5f82601f83011261231157612310611d8c565b5b8135612321848260208601612277565b91505092915050565b5f61018082840312156123405761233f611c7c565b5b61234b610100611cee565b90505f82013567ffffffffffffffff81111561236a57612369611d08565b5b6123768482850161215f565b5f83015250602082013567ffffffffffffffff81111561239957612398611d08565b5b6123a58482850161221f565b602083015250604082013567ffffffffffffffff8111156123c9576123c8611d08565b5b6123d58482850161221f565b60408301525060606123e984828501611e42565b60608301525060e06123fd84828501611d3f565b60808301525061012082013567ffffffffffffffff81111561242257612421611d08565b5b61242e8482850161215f565b60a08301525061014082013567ffffffffffffffff81111561245357612452611d08565b5b61245f8482850161215f565b60c08301525061016082013567ffffffffffffffff81111561248457612483611d08565b5b612490848285016122fd565b60e08301525092915050565b5f5f5f5f5f608086880312156124b5576124b4611c41565b5b5f6124c288828901611c68565b955050602086013567ffffffffffffffff8111156124e3576124e2611c45565b5b6124ef8882890161203e565b94509450506040612502888289016120b8565b925050606086013567ffffffffffffffff81111561252357612522611c45565b5b61252f8882890161232a565b9150509295509295909350565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b5f6bffffffffffffffffffffffff82169050919050565b61258581612565565b82525050565b5f612596838361257c565b60208301905092915050565b5f602082019050919050565b5f6125b88261253c565b6125c28185612546565b93506125cd83612556565b805f5b838110156125fd5781516125e4888261258b565b97506125ef836125a2565b9250506001810190506125d0565b5085935050505092915050565b5f604083015f8301518482035f86015261262482826125ae565b9150506020830151848203602086015261263e82826125ae565b9150508091505092915050565b61265481611c49565b82525050565b5f6040820190508181035f830152612672818561260a565b9050612681602083018461264b565b9392505050565b5f61269282611f7e565b9050919050565b6126a281612688565b82525050565b5f6020820190506126bb5f830184612699565b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b5f819050919050565b61270861270382611c49565b6126ee565b82525050565b5f819050919050565b61272861272382611d0c565b61270e565b82525050565b5f612739828c6126f7565b602082019150612749828b612717565b602082019150612759828a612717565b6020820191506127698289612717565b6020820191506127798288612717565b6020820191506127898287612717565b6020820191506127998286612717565b6020820191506127a98285612717565b6020820191506127b98284612717565b6020820191508190509a9950505050505050505050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b5f61280782611d0c565b915061281283611d0c565b925082612822576128216127d0565b5b828206905092915050565b5f60ff82169050919050565b6128428161282d565b811461284c575f5ffd5b50565b5f8151905061285d81612839565b92915050565b5f6020828403121561287857612877611c41565b5b5f6128858482850161284f565b91505092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6128c582611d0c565b91506128d083611d0c565b92508282039050818111156128e8576128e761288e565b5b92915050565b6128f781612093565b82525050565b5f61291761291261290d84612093565b611f54565b611d0c565b9050919050565b612927816128fd565b82525050565b5f6060820190506129405f83018661264b565b61294d60208301856128ee565b61295a604083018461291e565b949350505050565b5f77ffffffffffffffffffffffffffffffffffffffffffffffff82169050919050565b61298e81612962565b8114612998575f5ffd5b50565b5f815190506129a981612985565b92915050565b5f602082840312156129c4576129c3611c41565b5b5f6129d18482850161299b565b91505092915050565b6129e38161282d565b82525050565b5f6060820190506129fc5f8301866129da565b612a0960208301856128ee565b612a16604083018461291e565b949350505050565b5f7fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000082169050919050565b612a5281612a1e565b8114612a5c575f5ffd5b50565b5f81519050612a6d81612a49565b92915050565b5f60208284031215612a8857612a87611c41565b5b5f612a9584828501612a5f565b91505092915050565b612aa781612565565b8114612ab1575f5ffd5b50565b5f81519050612ac281612a9e565b92915050565b5f60208284031215612add57612adc611c41565b5b5f612aea84828501612ab4565b91505092915050565b5f608082019050612b065f8301876129da565b612b1360208301866128ee565b612b20604083018561264b565b612b2d606083018461291e565b95945050505050565b5f612b4082612565565b9150612b4b83612565565b925082820390506bffffffffffffffffffffffff811115612b6f57612b6e61288e565b5b92915050565b5f8160e01b9050919050565b5f612b8b82612b75565b9050919050565b612ba3612b9e82612093565b612b81565b82525050565b5f81519050919050565b5f81905092915050565b5f819050602082019050919050565b612bd581611c49565b82525050565b5f612be68383612bcc565b60208301905092915050565b5f602082019050919050565b5f612c0882612ba9565b612c128185612bb3565b9350612c1d83612bbd565b805f5b83811015612c4d578151612c348882612bdb565b9750612c3f83612bf2565b925050600181019050612c20565b5085935050505092915050565b5f612c658285612b92565b600482019150612c758284612bfe565b91508190509392505050565b5f612c8b82611d0c565b9150612c9683611d0c565b9250828202612ca481611d0c565b91508282048414831517612cbb57612cba61288e565b5b5092915050565b5f612ccc82611d0c565b9150612cd783611d0c565b9250828201905080821115612cef57612cee61288e565b5b92915050565b5f61ffff82169050919050565b5f612d0c82612cf5565b915061ffff8203612d2057612d1f61288e565b5b60018201905091905056fea264697066735822122009fafd6a9cb72c083d065be8b674687b4e0fc68c06baf5234eac5e2e4975cdc864736f6c634300081d0033
    /// ```
    #[rustfmt::skip]
    #[allow(clippy::all)]
    pub static BYTECODE: alloy_sol_types::private::Bytes = alloy_sol_types::private::Bytes::from_static(
        b"a\x01\0`@R4\x80\x15a\0\x10W__\xFD[P`@Qa1\xD38\x03\x80a1\xD3\x839\x81\x81\x01`@R\x81\x01\x90a\x002\x91\x90a\x02\xBFV[\x80\x80s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`\x80\x81s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP\x80s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16ch0H5`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\0\xB0W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\0\xD4\x91\x90a\x03%V[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`\xA0\x81s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP\x80s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c]\xF4YF`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x01PW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x01t\x91\x90a\x03\x8BV[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`\xC0\x81s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP`\xA0Qs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\xDF\\\xF7#`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x01\xF2W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x02\x16\x91\x90a\x03\xF1V[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`\xE0\x81s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPPPPa\x04\x1CV[__\xFD[_s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[_a\x02}\x82a\x02TV[\x90P\x91\x90PV[_a\x02\x8E\x82a\x02sV[\x90P\x91\x90PV[a\x02\x9E\x81a\x02\x84V[\x81\x14a\x02\xA8W__\xFD[PV[_\x81Q\x90Pa\x02\xB9\x81a\x02\x95V[\x92\x91PPV[_` \x82\x84\x03\x12\x15a\x02\xD4Wa\x02\xD3a\x02PV[[_a\x02\xE1\x84\x82\x85\x01a\x02\xABV[\x91PP\x92\x91PPV[_a\x02\xF4\x82a\x02sV[\x90P\x91\x90PV[a\x03\x04\x81a\x02\xEAV[\x81\x14a\x03\x0EW__\xFD[PV[_\x81Q\x90Pa\x03\x1F\x81a\x02\xFBV[\x92\x91PPV[_` \x82\x84\x03\x12\x15a\x03:Wa\x039a\x02PV[[_a\x03G\x84\x82\x85\x01a\x03\x11V[\x91PP\x92\x91PPV[_a\x03Z\x82a\x02sV[\x90P\x91\x90PV[a\x03j\x81a\x03PV[\x81\x14a\x03tW__\xFD[PV[_\x81Q\x90Pa\x03\x85\x81a\x03aV[\x92\x91PPV[_` \x82\x84\x03\x12\x15a\x03\xA0Wa\x03\x9Fa\x02PV[[_a\x03\xAD\x84\x82\x85\x01a\x03wV[\x91PP\x92\x91PPV[_a\x03\xC0\x82a\x02sV[\x90P\x91\x90PV[a\x03\xD0\x81a\x03\xB6V[\x81\x14a\x03\xDAW__\xFD[PV[_\x81Q\x90Pa\x03\xEB\x81a\x03\xC7V[\x92\x91PPV[_` \x82\x84\x03\x12\x15a\x04\x06Wa\x04\x05a\x02PV[[_a\x04\x13\x84\x82\x85\x01a\x03\xDDV[\x91PP\x92\x91PPV[`\x80Q`\xA0Q`\xC0Q`\xE0Qa-aa\x04r_9_a\x0E\xEB\x01R_\x81\x81a\x02\xA3\x01Ra\t=\x01R_\x81\x81a\x02\xC7\x01R\x81\x81a\n\xBD\x01Ra\x0C\x97\x01R_\x81\x81a\x02\xEB\x01R\x81\x81a\x06\x17\x01Ra\x07\x94\x01Ra-a_\xF3\xFE`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\0`W_5`\xE0\x1C\x80c\x17\x1F\x1D[\x14a\0dW\x80c]\xF4YF\x14a\0\x95W\x80ch0H5\x14a\0\xB3W\x80cm\x14\xA9\x87\x14a\0\xD1W\x80cn\xFBF6\x14a\0\xEFW\x80c\xDF\\\xF7#\x14a\x01 W[__\xFD[a\0~`\x04\x806\x03\x81\x01\x90a\0y\x91\x90a\x1E\x8FV[a\x01>V[`@Qa\0\x8C\x92\x91\x90a\x1F\x0EV[`@Q\x80\x91\x03\x90\xF3[a\0\x9Da\x02\xA1V[`@Qa\0\xAA\x91\x90a\x1F\xAFV[`@Q\x80\x91\x03\x90\xF3[a\0\xBBa\x02\xC5V[`@Qa\0\xC8\x91\x90a\x1F\xE8V[`@Q\x80\x91\x03\x90\xF3[a\0\xD9a\x02\xE9V[`@Qa\0\xE6\x91\x90a !V[`@Q\x80\x91\x03\x90\xF3[a\x01\t`\x04\x806\x03\x81\x01\x90a\x01\x04\x91\x90a$\x9CV[a\x03\rV[`@Qa\x01\x17\x92\x91\x90a&ZV[`@Q\x80\x91\x03\x90\xF3[a\x01(a\x0E\xE9V[`@Qa\x015\x91\x90a&\xA8V[`@Q\x80\x91\x03\x90\xF3[___\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X](3\xE8Hy\xB9p\x91C\xE1\xF5\x93\xF0\0\0\x01\x87\x87_\x01Q\x88` \x01Q\x88_\x01Q_`\x02\x81\x10a\x01\x82Wa\x01\x81a&\xC1V[[` \x02\x01Q\x89_\x01Q`\x01`\x02\x81\x10a\x01\x9EWa\x01\x9Da&\xC1V[[` \x02\x01Q\x8A` \x01Q_`\x02\x81\x10a\x01\xBAWa\x01\xB9a&\xC1V[[` \x02\x01Q\x8B` \x01Q`\x01`\x02\x81\x10a\x01\xD7Wa\x01\xD6a&\xC1V[[` \x02\x01Q\x8B_\x01Q\x8C` \x01Q`@Q` \x01a\x01\xFD\x99\x98\x97\x96\x95\x94\x93\x92\x91\x90a'.V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 _\x1Ca\x02\x1F\x91\x90a'\xFDV[\x90Pa\x02\x8Fa\x02Ia\x02:\x83\x89a\x0F\r\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x86a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[a\x02Qa\x10\xC8V[a\x02\x85a\x02n\x85a\x02`a\x11\x92V[a\x0F\r\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[a\x02w\x8Ca\x11\xB6V[a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x88b\x01\xD4\xC0a\x12\xC1V[\x80\x93P\x81\x94PPPP\x94P\x94\x92PPPV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03\x15a\x1A\xF9V[__\x86\x86\x90P\x03a\x03RW`@Q\x7F\x1F\x04\x05\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82`@\x01QQ\x86\x86\x90P\x14\x80\x15a\x03pWP\x82`\xA0\x01QQ\x86\x86\x90P\x14[\x80\x15a\x03\x83WP\x82`\xC0\x01QQ\x86\x86\x90P\x14[\x80\x15a\x03\x96WP\x82`\xE0\x01QQ\x86\x86\x90P\x14[a\x03\xCCW`@Q\x7FCqJ\xFD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82_\x01QQ\x83` \x01QQ\x14a\x04\x0EW`@Q\x7F_\x83/A\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[Cc\xFF\xFF\xFF\xFF\x16\x84c\xFF\xFF\xFF\xFF\x16\x10a\x04SW`@Q\x7FK\x87OE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90Pa\x04sa\x1A\xF9V[\x87\x87\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x04\x90Wa\x04\x8Fa\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x04\xBEW\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81` \x01\x81\x90RP\x87\x87\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x04\xE4Wa\x04\xE3a\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05\x12W\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81_\x01\x81\x90RPa\x05\"a\x1B\x13V[\x85` \x01QQg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x05AWa\x05@a\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05oW\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81_\x01\x81\x90RP\x85` \x01QQg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x05\x96Wa\x05\x95a\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05\xC4W\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81` \x01\x81\x90RP_a\x06\xA7\x8A\x8A\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847_\x81\x84\x01R`\x1F\x19`\x1F\x82\x01\x16\x90P\x80\x83\x01\x92PPPPPPP\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\x9A\xA1e=`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x06~W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x06\xA2\x91\x90a(cV[a\x15aV[\x90P__\x90P[\x87` \x01QQ\x81\x10\x15a\t\x1EWa\x06\xE2\x88` \x01Q\x82\x81Q\x81\x10a\x06\xD5Wa\x06\xD4a&\xC1V[[` \x02` \x01\x01Qa\x15\xB8V[\x83` \x01Q\x82\x81Q\x81\x10a\x06\xF9Wa\x06\xF8a&\xC1V[[` \x02` \x01\x01\x81\x81RPP_\x81\x14a\x07\x92W\x82` \x01Q`\x01\x82a\x07\x1E\x91\x90a(\xBBV[\x81Q\x81\x10a\x07/Wa\x07.a&\xC1V[[` \x02` \x01\x01Q_\x1C\x83` \x01Q\x82\x81Q\x81\x10a\x07PWa\x07Oa&\xC1V[[` \x02` \x01\x01Q_\x1C\x11a\x07\x91W`@Q\x7F\xFFq\x94\x14\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\x04\xECcQ\x84` \x01Q\x83\x81Q\x81\x10a\x07\xE5Wa\x07\xE4a&\xC1V[[` \x02` \x01\x01Q\x8B\x8B_\x01Q\x85\x81Q\x81\x10a\x08\x04Wa\x08\x03a&\xC1V[[` \x02` \x01\x01Q`@Q\x84c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x08*\x93\x92\x91\x90a)-V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x08EW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x08i\x91\x90a)\xAFV[w\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x83_\x01Q\x82\x81Q\x81\x10a\x08\x99Wa\x08\x98a&\xC1V[[` \x02` \x01\x01\x81\x81RPPa\t\x0Fa\t\0a\x08\xD3\x84\x86_\x01Q\x85\x81Q\x81\x10a\x08\xC5Wa\x08\xC4a&\xC1V[[` \x02` \x01\x01Q\x16a\x15\xD0V[\x8A` \x01Q\x84\x81Q\x81\x10a\x08\xEAWa\x08\xE9a&\xC1V[[` \x02` \x01\x01Qa\x16\x0B\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x86a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x94P\x80\x80`\x01\x01\x91PPa\x06\xAEV[PPa\t)\x83a\x16\xF0V[\x92P__\x90P[\x89\x89\x90P\x81\x10\x15a\x0E\x1AW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16ch\xBC\xCA\xAC\x8B\x8B\x84\x81\x81\x10a\t\x8AWa\t\x89a&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1C\x8A\x8A`\xA0\x01Q\x85\x81Q\x81\x10a\t\xAFWa\t\xAEa&\xC1V[[` \x02` \x01\x01Q`@Q\x84c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\t\xD5\x93\x92\x91\x90a)\xE9V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\t\xF0W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\n\x14\x91\x90a*sV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16a\nF\x88`@\x01Q\x83\x81Q\x81\x10a\n9Wa\n8a&\xC1V[[` \x02` \x01\x01Qa\x15\xB8V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16\x14a\n\x88W`@Q\x7F\xE11\n\xED\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\n\xB9\x87`@\x01Q\x82\x81Q\x81\x10a\n\xA2Wa\n\xA1a&\xC1V[[` \x02` \x01\x01Q\x85a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x93P\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\xC8)LV\x8B\x8B\x84\x81\x81\x10a\x0B\nWa\x0B\ta&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1C\x8A\x8A`\xC0\x01Q\x85\x81Q\x81\x10a\x0B/Wa\x0B.a&\xC1V[[` \x02` \x01\x01Q`@Q\x84c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x0BU\x93\x92\x91\x90a)\xE9V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0BpW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0B\x94\x91\x90a*\xC8V[\x83` \x01Q\x82\x81Q\x81\x10a\x0B\xABWa\x0B\xAAa&\xC1V[[` \x02` \x01\x01\x90k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x81k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP\x82` \x01Q\x81\x81Q\x81\x10a\x0B\xECWa\x0B\xEBa&\xC1V[[` \x02` \x01\x01Q\x83_\x01Q\x82\x81Q\x81\x10a\x0C\nWa\x0C\ta&\xC1V[[` \x02` \x01\x01\x90k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x81k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP__\x90P__\x90P[\x88` \x01QQ\x81\x10\x15a\x0E\x0BWa\x0C\x90\x84_\x01Q\x82\x81Q\x81\x10a\x0CcWa\x0Cba&\xC1V[[` \x02` \x01\x01Q\x8D\x8D\x86\x81\x81\x10a\x0C~Wa\x0C}a&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1Ca\x17\xA8V[\x15a\r\xFEW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\xF2\xBE\x94\xAE\x8D\x8D\x86\x81\x81\x10a\x0C\xE4Wa\x0C\xE3a&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1C\x8C\x87` \x01Q\x85\x81Q\x81\x10a\r\tWa\r\x08a&\xC1V[[` \x02` \x01\x01Q\x8D`\xE0\x01Q\x88\x81Q\x81\x10a\r(Wa\r'a&\xC1V[[` \x02` \x01\x01Q\x87\x81Q\x81\x10a\rBWa\rAa&\xC1V[[` \x02` \x01\x01Q`@Q\x85c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\ri\x94\x93\x92\x91\x90a*\xF3V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\r\x84W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\r\xA8\x91\x90a*\xC8V[\x85_\x01Q\x84\x81Q\x81\x10a\r\xBEWa\r\xBDa&\xC1V[[` \x02` \x01\x01\x81\x81Qa\r\xD2\x91\x90a+6V[\x91P\x90k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x81k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP\x81`\x01\x01\x91P[\x80\x80`\x01\x01\x91PPa\x0C=V[PP\x80\x80`\x01\x01\x91PPa\t0V[P__a\x0E1\x8C\x86\x8A``\x01Q\x8B`\x80\x01Qa\x01>V[\x91P\x91P\x81a\x0ElW`@Q\x7Fg\x98\x8D3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80a\x0E\xA3W`@Q\x7F\xAB\x1B#k\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PP_\x87\x82` \x01Q`@Q` \x01a\x0E\xBD\x92\x91\x90a,ZV[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P\x82\x81\x95P\x95PPPPP\x95P\x95\x93PPPPV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x0F\x15a\x1B-V[a\x0F\x1Da\x1BEV[\x83_\x01Q\x81_`\x03\x81\x10a\x0F4Wa\x0F3a&\xC1V[[` \x02\x01\x81\x81RPP\x83` \x01Q\x81`\x01`\x03\x81\x10a\x0FVWa\x0FUa&\xC1V[[` \x02\x01\x81\x81RPP\x82\x81`\x02`\x03\x81\x10a\x0FtWa\x0Fsa&\xC1V[[` \x02\x01\x81\x81RPP_`@\x83``\x84`\x07a\x07\xD0Z\x03\xFA\x90P\x80_\x81\x03a\x0F\x98W\xFE[P\x80a\x0F\xD0W`@Q\x7FF3\xBE2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PP\x92\x91PPV[a\x0F\xE0a\x1B-V[a\x0F\xE8a\x1BgV[\x83_\x01Q\x81_`\x04\x81\x10a\x0F\xFFWa\x0F\xFEa&\xC1V[[` \x02\x01\x81\x81RPP\x83` \x01Q\x81`\x01`\x04\x81\x10a\x10!Wa\x10 a&\xC1V[[` \x02\x01\x81\x81RPP\x82_\x01Q\x81`\x02`\x04\x81\x10a\x10BWa\x10Aa&\xC1V[[` \x02\x01\x81\x81RPP\x82` \x01Q\x81`\x03`\x04\x81\x10a\x10dWa\x10ca&\xC1V[[` \x02\x01\x81\x81RPP_`@\x83`\x80\x84`\x06a\x07\xD0Z\x03\xFA\x90P\x80_\x81\x03a\x10\x88W\xFE[P\x80a\x10\xC0W`@Q\x7F\xD4\xB6\x8F\xD7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PP\x92\x91PPV[a\x10\xD0a\x1B\x89V[`@Q\x80`@\x01`@R\x80`@Q\x80`@\x01`@R\x80\x7F\x19\x8E\x93\x93\x92\rH:r`\xBF\xB71\xFB]%\xF1\xAAI35\xA9\xE7\x12\x97\xE4\x85\xB7\xAE\xF3\x12\xC2\x81R` \x01\x7F\x18\0\xDE\xEF\x12\x1F\x1EvBj\0f^\\DygC\"\xD4\xF7^\xDA\xDDF\xDE\xBD\\\xD9\x92\xF6\xED\x81RP\x81R` \x01`@Q\x80`@\x01`@R\x80\x7F']\xC4\xA2\x88\xD1\xAF\xB3\xCB\xB1\xAC\t\x18u$\xC7\xDB69]\xF7\xBE;\x99\xE6s\xB1:\x07Ze\xEC\x81R` \x01\x7F\x1D\x9B\xEF\xCD\x05\xA52>m\xA4\xD45\xF3\xB6\x17\xCD\xB3\xAF\x83(\\-\xF7\x11\xEF9\xC0\x15q\x82\x7F\x9D\x81RP\x81RP\x90P\x90V[a\x11\x9Aa\x1B-V[`@Q\x80`@\x01`@R\x80`\x01\x81R` \x01`\x02\x81RP\x90P\x90V[a\x11\xBEa\x1B-V[__\x90P__\x90P_\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x85_\x1Ca\x11\xF5\x91\x90a'\xFDV[\x90P[`\x01\x15a\x12\xA1Wa\x12\x08\x81a\x17\xBEV[\x80\x93P\x81\x94PPP\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x12?Wa\x12>a'\xD0V[[\x82\x83\t\x83\x03a\x12gW`@Q\x80`@\x01`@R\x80\x82\x81R` \x01\x83\x81RP\x93PPPPa\x12\xBCV[\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x12\x96Wa\x12\x95a'\xD0V[[`\x01\x82\x08\x90Pa\x11\xF8V[`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x93PPPP[\x91\x90PV[___`@Q\x80`@\x01`@R\x80\x89\x81R` \x01\x87\x81RP\x90P_`@Q\x80`@\x01`@R\x80\x89\x81R` \x01\x87\x81RP\x90Pa\x12\xFBa\x1B\xAFV[__\x90P[`\x02\x81\x10\x15a\x15\x19W_`\x06\x82a\x13\x17\x91\x90a,\x81V[\x90P\x84\x82`\x02\x81\x10a\x13,Wa\x13+a&\xC1V[[` \x02\x01Q_\x01Q\x83_\x83a\x13A\x91\x90a,\xC2V[`\x0C\x81\x10a\x13RWa\x13Qa&\xC1V[[` \x02\x01\x81\x81RPP\x84\x82`\x02\x81\x10a\x13nWa\x13ma&\xC1V[[` \x02\x01Q` \x01Q\x83`\x01\x83a\x13\x85\x91\x90a,\xC2V[`\x0C\x81\x10a\x13\x96Wa\x13\x95a&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x13\xB2Wa\x13\xB1a&\xC1V[[` \x02\x01Q_\x01Q_`\x02\x81\x10a\x13\xCCWa\x13\xCBa&\xC1V[[` \x02\x01Q\x83`\x02\x83a\x13\xDF\x91\x90a,\xC2V[`\x0C\x81\x10a\x13\xF0Wa\x13\xEFa&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x14\x0CWa\x14\x0Ba&\xC1V[[` \x02\x01Q_\x01Q`\x01`\x02\x81\x10a\x14'Wa\x14&a&\xC1V[[` \x02\x01Q\x83`\x03\x83a\x14:\x91\x90a,\xC2V[`\x0C\x81\x10a\x14KWa\x14Ja&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x14gWa\x14fa&\xC1V[[` \x02\x01Q` \x01Q_`\x02\x81\x10a\x14\x82Wa\x14\x81a&\xC1V[[` \x02\x01Q\x83`\x04\x83a\x14\x95\x91\x90a,\xC2V[`\x0C\x81\x10a\x14\xA6Wa\x14\xA5a&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x14\xC2Wa\x14\xC1a&\xC1V[[` \x02\x01Q` \x01Q`\x01`\x02\x81\x10a\x14\xDEWa\x14\xDDa&\xC1V[[` \x02\x01Q\x83`\x05\x83a\x14\xF1\x91\x90a,\xC2V[`\x0C\x81\x10a\x15\x02Wa\x15\x01a&\xC1V[[` \x02\x01\x81\x81RPPP\x80\x80`\x01\x01\x91PPa\x13\0V[Pa\x15\"a\x1B\xD2V[_` \x82` `\x0C\x02\x85`\x08\x8C\xFA\x90P\x80_\x83_`\x01\x81\x10a\x15GWa\x15Fa&\xC1V[[` \x02\x01Q\x14\x15\x96P\x96PPPPPP\x95P\x95\x93PPPPV[__a\x15l\x84a\x18\xB3V[\x90P\x80\x83`\xFF\x16`\x01\x90\x1B\x11a\x15\xAEW`@Q\x7F\xCA\x95s3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80\x91PP\x92\x91PPV[_\x81Q_R\x81` \x01Q` R`@_ \x90P\x91\x90PV[___\x90P[_\x83\x11\x15a\x16\x02W`\x01\x83a\x15\xEB\x91\x90a(\xBBV[\x83\x16\x92P\x80\x80a\x15\xFA\x90a-\x02V[\x91PPa\x15\xD6V[\x80\x91PP\x91\x90PV[a\x16\x13a\x1B-V[a\x02\0\x82a\xFF\xFF\x16\x10a\x16RW`@Q\x7F\xFF\x89\xD4\xFA\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x82a\xFF\xFF\x16\x03a\x16fW\x82\x90Pa\x16\xEAV[_`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90P_\x84\x90P_`\x01\x90P__\x90P[\x81a\xFF\xFF\x16\x86a\xFF\xFF\x16\x10a\x16\xE2W`\x01\x80\x82`\xFF\x16\x88a\xFF\xFF\x16\x90\x1C\x16a\xFF\xFF\x16\x03a\x16\xC0Wa\x16\xBD\x84\x84a\x0F\xD8V[\x93P[a\x16\xCA\x83\x84a\x0F\xD8V[\x92P`\x01\x82a\xFF\xFF\x16\x90\x1B\x91P\x80`\x01\x01\x90Pa\x16\x8CV[\x83\x94PPPPP[\x92\x91PPV[a\x16\xF8a\x1B-V[_\x82_\x01Q\x14\x80\x15a\x17\rWP_\x82` \x01Q\x14[\x15a\x17.W`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90Pa\x17\xA3V[`@Q\x80`@\x01`@R\x80\x83_\x01Q\x81R` \x01\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x84` \x01Qa\x17r\x91\x90a'\xFDV[\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDGa\x17\x9D\x91\x90a(\xBBV[\x81RP\x90P[\x91\x90PV[_`\x01\x82`\xFF\x16\x84\x90\x1C\x16`\x01\x14\x90P\x92\x91PPV[___\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x17\xF0Wa\x17\xEFa'\xD0V[[`\x03\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x18!Wa\x18 a'\xD0V[[\x86\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x18QWa\x18Pa'\xD0V[[\x88\x89\t\t\x08\x90P_a\x18\xA4\x82\x7F\x0C\x19\x13\x9C\xB8Lh\nn\x14\x11m\xA0`V\x17e\xE0Z\xA4Z\x1Cr\xA3O\x08#\x05\xB6\x1F?R\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDGa\x19\xBBV[\x90P\x81\x81\x93P\x93PPP\x91P\x91V[_a\x01\0\x82Q\x11\x15a\x18\xF1W`@Q\x7F\xFBJ\x9C\x8E\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_\x82Q\x03a\x19\x01W_\x90Pa\x19\xB6V[__\x83_\x81Q\x81\x10a\x19\x16Wa\x19\x15a&\xC1V[[` \x01\x01Q`\xF8\x1C`\xF8\x1B`\xF8\x1C`\xFF\x16`\x01\x90\x1B\x91P_`\x01\x90P[\x84Q\x81\x10\x15a\x19\xAFW\x84\x81\x81Q\x81\x10a\x19OWa\x19Na&\xC1V[[` \x01\x01Q`\xF8\x1C`\xF8\x1B`\xF8\x1C`\xFF\x16`\x01\x90\x1B\x91P\x82\x82\x11a\x19\x9FW`@Q\x7F\x80\xC8\x83H\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x81\x83\x17\x92P\x80`\x01\x01\x90Pa\x193V[P\x81\x92PPP[\x91\x90PV[__a\x19\xC5a\x1B\xD2V[a\x19\xCDa\x1B\xF4V[` \x81_`\x06\x81\x10a\x19\xE2Wa\x19\xE1a&\xC1V[[` \x02\x01\x81\x81RPP` \x81`\x01`\x06\x81\x10a\x1A\x01Wa\x1A\0a&\xC1V[[` \x02\x01\x81\x81RPP` \x81`\x02`\x06\x81\x10a\x1A Wa\x1A\x1Fa&\xC1V[[` \x02\x01\x81\x81RPP\x86\x81`\x03`\x06\x81\x10a\x1A>Wa\x1A=a&\xC1V[[` \x02\x01\x81\x81RPP\x85\x81`\x04`\x06\x81\x10a\x1A\\Wa\x1A[a&\xC1V[[` \x02\x01\x81\x81RPP\x84\x81`\x05`\x06\x81\x10a\x1AzWa\x1Aya&\xC1V[[` \x02\x01\x81\x81RPP` \x82`\xC0\x83`\x05a\x07\xD0Z\x03\xFA\x92P\x82_\x81\x03a\x1A\x9DW\xFE[P\x82a\x1A\xD5W`@Q\x7F\xD5\x1E\xDA\xE3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x81_`\x01\x81\x10a\x1A\xE8Wa\x1A\xE7a&\xC1V[[` \x02\x01Q\x93PPPP\x93\x92PPPV[`@Q\x80`@\x01`@R\x80``\x81R` \x01``\x81RP\x90V[`@Q\x80`@\x01`@R\x80``\x81R` \x01``\x81RP\x90V[`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90V[`@Q\x80``\x01`@R\x80`\x03\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`\x80\x01`@R\x80`\x04\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`@\x01`@R\x80a\x1B\x9Ca\x1C\x16V[\x81R` \x01a\x1B\xA9a\x1C\x16V[\x81RP\x90V[`@Q\x80a\x01\x80\x01`@R\x80`\x0C\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80` \x01`@R\x80`\x01\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`\xC0\x01`@R\x80`\x06\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`@\x01`@R\x80`\x02\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[_`@Q\x90P\x90V[__\xFD[__\xFD[_\x81\x90P\x91\x90PV[a\x1C[\x81a\x1CIV[\x81\x14a\x1CeW__\xFD[PV[_\x815\x90Pa\x1Cv\x81a\x1CRV[\x92\x91PPV[__\xFD[_`\x1F\x19`\x1F\x83\x01\x16\x90P\x91\x90PV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`A`\x04R`$_\xFD[a\x1C\xC6\x82a\x1C\x80V[\x81\x01\x81\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17\x15a\x1C\xE5Wa\x1C\xE4a\x1C\x90V[[\x80`@RPPPV[_a\x1C\xF7a\x1C8V[\x90Pa\x1D\x03\x82\x82a\x1C\xBDV[\x91\x90PV[__\xFD[_\x81\x90P\x91\x90PV[a\x1D\x1E\x81a\x1D\x0CV[\x81\x14a\x1D(W__\xFD[PV[_\x815\x90Pa\x1D9\x81a\x1D\x15V[\x92\x91PPV[_`@\x82\x84\x03\x12\x15a\x1DTWa\x1DSa\x1C|V[[a\x1D^`@a\x1C\xEEV[\x90P_a\x1Dm\x84\x82\x85\x01a\x1D+V[_\x83\x01RP` a\x1D\x80\x84\x82\x85\x01a\x1D+V[` \x83\x01RP\x92\x91PPV[__\xFD[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a\x1D\xAAWa\x1D\xA9a\x1C\x90V[[` \x82\x02\x90P\x91\x90PV[__\xFD[_a\x1D\xCBa\x1D\xC6\x84a\x1D\x90V[a\x1C\xEEV[\x90P\x80` \x84\x02\x83\x01\x85\x81\x11\x15a\x1D\xE5Wa\x1D\xE4a\x1D\xB5V[[\x83[\x81\x81\x10\x15a\x1E\x0EW\x80a\x1D\xFA\x88\x82a\x1D+V[\x84R` \x84\x01\x93PP` \x81\x01\x90Pa\x1D\xE7V[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a\x1E,Wa\x1E+a\x1D\x8CV[[`\x02a\x1E9\x84\x82\x85a\x1D\xB9V[\x91PP\x92\x91PPV[_`\x80\x82\x84\x03\x12\x15a\x1EWWa\x1EVa\x1C|V[[a\x1Ea`@a\x1C\xEEV[\x90P_a\x1Ep\x84\x82\x85\x01a\x1E\x18V[_\x83\x01RP`@a\x1E\x83\x84\x82\x85\x01a\x1E\x18V[` \x83\x01RP\x92\x91PPV[____a\x01 \x85\x87\x03\x12\x15a\x1E\xA8Wa\x1E\xA7a\x1CAV[[_a\x1E\xB5\x87\x82\x88\x01a\x1ChV[\x94PP` a\x1E\xC6\x87\x82\x88\x01a\x1D?V[\x93PP``a\x1E\xD7\x87\x82\x88\x01a\x1EBV[\x92PP`\xE0a\x1E\xE8\x87\x82\x88\x01a\x1D?V[\x91PP\x92\x95\x91\x94P\x92PV[_\x81\x15\x15\x90P\x91\x90PV[a\x1F\x08\x81a\x1E\xF4V[\x82RPPV[_`@\x82\x01\x90Pa\x1F!_\x83\x01\x85a\x1E\xFFV[a\x1F.` \x83\x01\x84a\x1E\xFFV[\x93\x92PPPV[_s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[_\x81\x90P\x91\x90PV[_a\x1Fwa\x1Fra\x1Fm\x84a\x1F5V[a\x1FTV[a\x1F5V[\x90P\x91\x90PV[_a\x1F\x88\x82a\x1F]V[\x90P\x91\x90PV[_a\x1F\x99\x82a\x1F~V[\x90P\x91\x90PV[a\x1F\xA9\x81a\x1F\x8FV[\x82RPPV[_` \x82\x01\x90Pa\x1F\xC2_\x83\x01\x84a\x1F\xA0V[\x92\x91PPV[_a\x1F\xD2\x82a\x1F~V[\x90P\x91\x90PV[a\x1F\xE2\x81a\x1F\xC8V[\x82RPPV[_` \x82\x01\x90Pa\x1F\xFB_\x83\x01\x84a\x1F\xD9V[\x92\x91PPV[_a \x0B\x82a\x1F~V[\x90P\x91\x90PV[a \x1B\x81a \x01V[\x82RPPV[_` \x82\x01\x90Pa 4_\x83\x01\x84a \x12V[\x92\x91PPV[__\xFD[__\x83`\x1F\x84\x01\x12a SWa Ra\x1D\x8CV[[\x825\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a pWa oa :V[[` \x83\x01\x91P\x83`\x01\x82\x02\x83\x01\x11\x15a \x8CWa \x8Ba\x1D\xB5V[[\x92P\x92\x90PV[_c\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[a \xAB\x81a \x93V[\x81\x14a \xB5W__\xFD[PV[_\x815\x90Pa \xC6\x81a \xA2V[\x92\x91PPV[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a \xE6Wa \xE5a\x1C\x90V[[` \x82\x02\x90P` \x81\x01\x90P\x91\x90PV[_a!\ta!\x04\x84a \xCCV[a\x1C\xEEV[\x90P\x80\x83\x82R` \x82\x01\x90P` \x84\x02\x83\x01\x85\x81\x11\x15a!,Wa!+a\x1D\xB5V[[\x83[\x81\x81\x10\x15a!UW\x80a!A\x88\x82a \xB8V[\x84R` \x84\x01\x93PP` \x81\x01\x90Pa!.V[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a!sWa!ra\x1D\x8CV[[\x815a!\x83\x84\x82` \x86\x01a \xF7V[\x91PP\x92\x91PPV[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a!\xA6Wa!\xA5a\x1C\x90V[[` \x82\x02\x90P` \x81\x01\x90P\x91\x90PV[_a!\xC9a!\xC4\x84a!\x8CV[a\x1C\xEEV[\x90P\x80\x83\x82R` \x82\x01\x90P`@\x84\x02\x83\x01\x85\x81\x11\x15a!\xECWa!\xEBa\x1D\xB5V[[\x83[\x81\x81\x10\x15a\"\x15W\x80a\"\x01\x88\x82a\x1D?V[\x84R` \x84\x01\x93PP`@\x81\x01\x90Pa!\xEEV[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a\"3Wa\"2a\x1D\x8CV[[\x815a\"C\x84\x82` \x86\x01a!\xB7V[\x91PP\x92\x91PPV[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a\"fWa\"ea\x1C\x90V[[` \x82\x02\x90P` \x81\x01\x90P\x91\x90PV[_a\"\x89a\"\x84\x84a\"LV[a\x1C\xEEV[\x90P\x80\x83\x82R` \x82\x01\x90P` \x84\x02\x83\x01\x85\x81\x11\x15a\"\xACWa\"\xABa\x1D\xB5V[[\x83[\x81\x81\x10\x15a\"\xF3W\x805g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\"\xD1Wa\"\xD0a\x1D\x8CV[[\x80\x86\x01a\"\xDE\x89\x82a!_V[\x85R` \x85\x01\x94PPP` \x81\x01\x90Pa\"\xAEV[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a#\x11Wa#\x10a\x1D\x8CV[[\x815a#!\x84\x82` \x86\x01a\"wV[\x91PP\x92\x91PPV[_a\x01\x80\x82\x84\x03\x12\x15a#@Wa#?a\x1C|V[[a#Ka\x01\0a\x1C\xEEV[\x90P_\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a#jWa#ia\x1D\x08V[[a#v\x84\x82\x85\x01a!_V[_\x83\x01RP` \x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a#\x99Wa#\x98a\x1D\x08V[[a#\xA5\x84\x82\x85\x01a\"\x1FV[` \x83\x01RP`@\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a#\xC9Wa#\xC8a\x1D\x08V[[a#\xD5\x84\x82\x85\x01a\"\x1FV[`@\x83\x01RP``a#\xE9\x84\x82\x85\x01a\x1EBV[``\x83\x01RP`\xE0a#\xFD\x84\x82\x85\x01a\x1D?V[`\x80\x83\x01RPa\x01 \x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$\"Wa$!a\x1D\x08V[[a$.\x84\x82\x85\x01a!_V[`\xA0\x83\x01RPa\x01@\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$SWa$Ra\x1D\x08V[[a$_\x84\x82\x85\x01a!_V[`\xC0\x83\x01RPa\x01`\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$\x84Wa$\x83a\x1D\x08V[[a$\x90\x84\x82\x85\x01a\"\xFDV[`\xE0\x83\x01RP\x92\x91PPV[_____`\x80\x86\x88\x03\x12\x15a$\xB5Wa$\xB4a\x1CAV[[_a$\xC2\x88\x82\x89\x01a\x1ChV[\x95PP` \x86\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$\xE3Wa$\xE2a\x1CEV[[a$\xEF\x88\x82\x89\x01a >V[\x94P\x94PP`@a%\x02\x88\x82\x89\x01a \xB8V[\x92PP``\x86\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a%#Wa%\"a\x1CEV[[a%/\x88\x82\x89\x01a#*V[\x91PP\x92\x95P\x92\x95\x90\x93PV[_\x81Q\x90P\x91\x90PV[_\x82\x82R` \x82\x01\x90P\x92\x91PPV[_\x81\x90P` \x82\x01\x90P\x91\x90PV[_k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[a%\x85\x81a%eV[\x82RPPV[_a%\x96\x83\x83a%|V[` \x83\x01\x90P\x92\x91PPV[_` \x82\x01\x90P\x91\x90PV[_a%\xB8\x82a%<V[a%\xC2\x81\x85a%FV[\x93Pa%\xCD\x83a%VV[\x80_[\x83\x81\x10\x15a%\xFDW\x81Qa%\xE4\x88\x82a%\x8BV[\x97Pa%\xEF\x83a%\xA2V[\x92PP`\x01\x81\x01\x90Pa%\xD0V[P\x85\x93PPPP\x92\x91PPV[_`@\x83\x01_\x83\x01Q\x84\x82\x03_\x86\x01Ra&$\x82\x82a%\xAEV[\x91PP` \x83\x01Q\x84\x82\x03` \x86\x01Ra&>\x82\x82a%\xAEV[\x91PP\x80\x91PP\x92\x91PPV[a&T\x81a\x1CIV[\x82RPPV[_`@\x82\x01\x90P\x81\x81\x03_\x83\x01Ra&r\x81\x85a&\nV[\x90Pa&\x81` \x83\x01\x84a&KV[\x93\x92PPPV[_a&\x92\x82a\x1F~V[\x90P\x91\x90PV[a&\xA2\x81a&\x88V[\x82RPPV[_` \x82\x01\x90Pa&\xBB_\x83\x01\x84a&\x99V[\x92\x91PPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`2`\x04R`$_\xFD[_\x81\x90P\x91\x90PV[a'\x08a'\x03\x82a\x1CIV[a&\xEEV[\x82RPPV[_\x81\x90P\x91\x90PV[a'(a'#\x82a\x1D\x0CV[a'\x0EV[\x82RPPV[_a'9\x82\x8Ca&\xF7V[` \x82\x01\x91Pa'I\x82\x8Ba'\x17V[` \x82\x01\x91Pa'Y\x82\x8Aa'\x17V[` \x82\x01\x91Pa'i\x82\x89a'\x17V[` \x82\x01\x91Pa'y\x82\x88a'\x17V[` \x82\x01\x91Pa'\x89\x82\x87a'\x17V[` \x82\x01\x91Pa'\x99\x82\x86a'\x17V[` \x82\x01\x91Pa'\xA9\x82\x85a'\x17V[` \x82\x01\x91Pa'\xB9\x82\x84a'\x17V[` \x82\x01\x91P\x81\x90P\x9A\x99PPPPPPPPPPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x12`\x04R`$_\xFD[_a(\x07\x82a\x1D\x0CV[\x91Pa(\x12\x83a\x1D\x0CV[\x92P\x82a(\"Wa(!a'\xD0V[[\x82\x82\x06\x90P\x92\x91PPV[_`\xFF\x82\x16\x90P\x91\x90PV[a(B\x81a(-V[\x81\x14a(LW__\xFD[PV[_\x81Q\x90Pa(]\x81a(9V[\x92\x91PPV[_` \x82\x84\x03\x12\x15a(xWa(wa\x1CAV[[_a(\x85\x84\x82\x85\x01a(OV[\x91PP\x92\x91PPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x11`\x04R`$_\xFD[_a(\xC5\x82a\x1D\x0CV[\x91Pa(\xD0\x83a\x1D\x0CV[\x92P\x82\x82\x03\x90P\x81\x81\x11\x15a(\xE8Wa(\xE7a(\x8EV[[\x92\x91PPV[a(\xF7\x81a \x93V[\x82RPPV[_a)\x17a)\x12a)\r\x84a \x93V[a\x1FTV[a\x1D\x0CV[\x90P\x91\x90PV[a)'\x81a(\xFDV[\x82RPPV[_``\x82\x01\x90Pa)@_\x83\x01\x86a&KV[a)M` \x83\x01\x85a(\xEEV[a)Z`@\x83\x01\x84a)\x1EV[\x94\x93PPPPV[_w\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[a)\x8E\x81a)bV[\x81\x14a)\x98W__\xFD[PV[_\x81Q\x90Pa)\xA9\x81a)\x85V[\x92\x91PPV[_` \x82\x84\x03\x12\x15a)\xC4Wa)\xC3a\x1CAV[[_a)\xD1\x84\x82\x85\x01a)\x9BV[\x91PP\x92\x91PPV[a)\xE3\x81a(-V[\x82RPPV[_``\x82\x01\x90Pa)\xFC_\x83\x01\x86a)\xDAV[a*\t` \x83\x01\x85a(\xEEV[a*\x16`@\x83\x01\x84a)\x1EV[\x94\x93PPPPV[_\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x82\x16\x90P\x91\x90PV[a*R\x81a*\x1EV[\x81\x14a*\\W__\xFD[PV[_\x81Q\x90Pa*m\x81a*IV[\x92\x91PPV[_` \x82\x84\x03\x12\x15a*\x88Wa*\x87a\x1CAV[[_a*\x95\x84\x82\x85\x01a*_V[\x91PP\x92\x91PPV[a*\xA7\x81a%eV[\x81\x14a*\xB1W__\xFD[PV[_\x81Q\x90Pa*\xC2\x81a*\x9EV[\x92\x91PPV[_` \x82\x84\x03\x12\x15a*\xDDWa*\xDCa\x1CAV[[_a*\xEA\x84\x82\x85\x01a*\xB4V[\x91PP\x92\x91PPV[_`\x80\x82\x01\x90Pa+\x06_\x83\x01\x87a)\xDAV[a+\x13` \x83\x01\x86a(\xEEV[a+ `@\x83\x01\x85a&KV[a+-``\x83\x01\x84a)\x1EV[\x95\x94PPPPPV[_a+@\x82a%eV[\x91Pa+K\x83a%eV[\x92P\x82\x82\x03\x90Pk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a+oWa+na(\x8EV[[\x92\x91PPV[_\x81`\xE0\x1B\x90P\x91\x90PV[_a+\x8B\x82a+uV[\x90P\x91\x90PV[a+\xA3a+\x9E\x82a \x93V[a+\x81V[\x82RPPV[_\x81Q\x90P\x91\x90PV[_\x81\x90P\x92\x91PPV[_\x81\x90P` \x82\x01\x90P\x91\x90PV[a+\xD5\x81a\x1CIV[\x82RPPV[_a+\xE6\x83\x83a+\xCCV[` \x83\x01\x90P\x92\x91PPV[_` \x82\x01\x90P\x91\x90PV[_a,\x08\x82a+\xA9V[a,\x12\x81\x85a+\xB3V[\x93Pa,\x1D\x83a+\xBDV[\x80_[\x83\x81\x10\x15a,MW\x81Qa,4\x88\x82a+\xDBV[\x97Pa,?\x83a+\xF2V[\x92PP`\x01\x81\x01\x90Pa, V[P\x85\x93PPPP\x92\x91PPV[_a,e\x82\x85a+\x92V[`\x04\x82\x01\x91Pa,u\x82\x84a+\xFEV[\x91P\x81\x90P\x93\x92PPPV[_a,\x8B\x82a\x1D\x0CV[\x91Pa,\x96\x83a\x1D\x0CV[\x92P\x82\x82\x02a,\xA4\x81a\x1D\x0CV[\x91P\x82\x82\x04\x84\x14\x83\x15\x17a,\xBBWa,\xBAa(\x8EV[[P\x92\x91PPV[_a,\xCC\x82a\x1D\x0CV[\x91Pa,\xD7\x83a\x1D\x0CV[\x92P\x82\x82\x01\x90P\x80\x82\x11\x15a,\xEFWa,\xEEa(\x8EV[[\x92\x91PPV[_a\xFF\xFF\x82\x16\x90P\x91\x90PV[_a-\x0C\x82a,\xF5V[\x91Pa\xFF\xFF\x82\x03a- Wa-\x1Fa(\x8EV[[`\x01\x82\x01\x90P\x91\x90PV\xFE\xA2dipfsX\"\x12 \t\xFA\xFDj\x9C\xB7,\x08=\x06[\xE8\xB6th{N\x0F\xC6\x8C\x06\xBA\xF5#N\xAC^.Iu\xCD\xC8dsolcC\0\x08\x1D\x003",
    );
    /// The runtime bytecode of the contract, as deployed on the network.
    ///
    /// ```text
    ///0x608060405234801561000f575f5ffd5b5060043610610060575f3560e01c8063171f1d5b146100645780635df459461461009557806368304835146100b35780636d14a987146100d15780636efb4636146100ef578063df5cf72314610120575b5f5ffd5b61007e60048036038101906100799190611e8f565b61013e565b60405161008c929190611f0e565b60405180910390f35b61009d6102a1565b6040516100aa9190611faf565b60405180910390f35b6100bb6102c5565b6040516100c89190611fe8565b60405180910390f35b6100d96102e9565b6040516100e69190612021565b60405180910390f35b6101096004803603810190610104919061249c565b61030d565b60405161011792919061265a565b60405180910390f35b610128610ee9565b60405161013591906126a8565b60405180910390f35b5f5f5f7f30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f000000187875f01518860200151885f01515f60028110610182576101816126c1565b5b6020020151895f015160016002811061019e5761019d6126c1565b5b60200201518a602001515f600281106101ba576101b96126c1565b5b60200201518b602001516001600281106101d7576101d66126c1565b5b60200201518b5f01518c602001516040516020016101fd9998979695949392919061272e565b604051602081830303815290604052805190602001205f1c61021f91906127fd565b905061028f61024961023a8389610f0d90919063ffffffff16565b86610fd890919063ffffffff16565b6102516110c8565b61028561026e85610260611192565b610f0d90919063ffffffff16565b6102778c6111b6565b610fd890919063ffffffff16565b886201d4c06112c1565b80935081945050505094509492505050565b7f000000000000000000000000000000000000000000000000000000000000000081565b7f000000000000000000000000000000000000000000000000000000000000000081565b7f000000000000000000000000000000000000000000000000000000000000000081565b610315611af9565b5f5f8686905003610352576040517f1f0405a000000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b8260400151518686905014801561037057508260a001515186869050145b801561038357508260c001515186869050145b801561039657508260e001515186869050145b6103cc576040517f43714afd00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b825f0151518360200151511461040e576040517f5f832f4100000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b4363ffffffff168463ffffffff1610610453576040517f4b874f4500000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b5f60405180604001604052805f81526020015f8152509050610473611af9565b8787905067ffffffffffffffff8111156104905761048f611c90565b5b6040519080825280602002602001820160405280156104be5781602001602082028036833780820191505090505b5081602001819052508787905067ffffffffffffffff8111156104e4576104e3611c90565b5b6040519080825280602002602001820160405280156105125781602001602082028036833780820191505090505b50815f0181905250610522611b13565b85602001515167ffffffffffffffff81111561054157610540611c90565b5b60405190808252806020026020018201604052801561056f5781602001602082028036833780820191505090505b50815f018190525085602001515167ffffffffffffffff81111561059657610595611c90565b5b6040519080825280602002602001820160405280156105c45781602001602082028036833780820191505090505b5081602001819052505f6106a78a8a8080601f0160208091040260200160405190810160405280939291908181526020018383808284375f81840152601f19601f820116905080830192505050505050507f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff16639aa1653d6040518163ffffffff1660e01b8152600401602060405180830381865afa15801561067e573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906106a29190612863565b611561565b90505f5f90505b87602001515181101561091e576106e2886020015182815181106106d5576106d46126c1565b5b60200260200101516115b8565b836020015182815181106106f9576106f86126c1565b5b6020026020010181815250505f811461079257826020015160018261071e91906128bb565b8151811061072f5761072e6126c1565b5b60200260200101515f1c836020015182815181106107505761074f6126c1565b5b60200260200101515f1c11610791576040517fff71941400000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b5b7f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff166304ec6351846020015183815181106107e5576107e46126c1565b5b60200260200101518b8b5f01518581518110610804576108036126c1565b5b60200260200101516040518463ffffffff1660e01b815260040161082a9392919061292d565b602060405180830381865afa158015610845573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061086991906129af565b77ffffffffffffffffffffffffffffffffffffffffffffffff16835f01518281518110610899576108986126c1565b5b60200260200101818152505061090f6109006108d384865f015185815181106108c5576108c46126c1565b5b6020026020010151166115d0565b8a6020015184815181106108ea576108e96126c1565b5b602002602001015161160b90919063ffffffff16565b86610fd890919063ffffffff16565b945080806001019150506106ae565b5050610929836116f0565b92505f5f90505b89899050811015610e1a577f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff166368bccaac8b8b8481811061098a576109896126c1565b5b9050013560f81c60f81b60f81c8a8a60a0015185815181106109af576109ae6126c1565b5b60200260200101516040518463ffffffff1660e01b81526004016109d5939291906129e9565b602060405180830381865afa1580156109f0573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610a149190612a73565b67ffffffffffffffff1916610a4688604001518381518110610a3957610a386126c1565b5b60200260200101516115b8565b67ffffffffffffffff191614610a88576040517fe1310aed00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b610ab987604001518281518110610aa257610aa16126c1565b5b602002602001015185610fd890919063ffffffff16565b93507f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff1663c8294c568b8b84818110610b0a57610b096126c1565b5b9050013560f81c60f81b60f81c8a8a60c001518581518110610b2f57610b2e6126c1565b5b60200260200101516040518463ffffffff1660e01b8152600401610b55939291906129e9565b602060405180830381865afa158015610b70573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610b949190612ac8565b83602001518281518110610bab57610baa6126c1565b5b60200260200101906bffffffffffffffffffffffff1690816bffffffffffffffffffffffff168152505082602001518181518110610bec57610beb6126c1565b5b6020026020010151835f01518281518110610c0a57610c096126c1565b5b60200260200101906bffffffffffffffffffffffff1690816bffffffffffffffffffffffff16815250505f5f90505f5f90505b886020015151811015610e0b57610c90845f01518281518110610c6357610c626126c1565b5b60200260200101518d8d86818110610c7e57610c7d6126c1565b5b9050013560f81c60f81b60f81c6117a8565b15610dfe577f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff1663f2be94ae8d8d86818110610ce457610ce36126c1565b5b9050013560f81c60f81b60f81c8c87602001518581518110610d0957610d086126c1565b5b60200260200101518d60e001518881518110610d2857610d276126c1565b5b60200260200101518781518110610d4257610d416126c1565b5b60200260200101516040518563ffffffff1660e01b8152600401610d699493929190612af3565b602060405180830381865afa158015610d84573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610da89190612ac8565b855f01518481518110610dbe57610dbd6126c1565b5b60200260200101818151610dd29190612b36565b9150906bffffffffffffffffffffffff1690816bffffffffffffffffffffffff16815250508160010191505b8080600101915050610c3d565b50508080600101915050610930565b505f5f610e318c868a606001518b6080015161013e565b9150915081610e6c576040517f67988d3300000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b80610ea3576040517fab1b236b00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b50505f878260200151604051602001610ebd929190612c5a565b604051602081830303815290604052805190602001209050828195509550505050509550959350505050565b7f000000000000000000000000000000000000000000000000000000000000000081565b610f15611b2d565b610f1d611b45565b835f0151815f60038110610f3457610f336126c1565b5b602002018181525050836020015181600160038110610f5657610f556126c1565b5b6020020181815250508281600260038110610f7457610f736126c1565b5b6020020181815250505f60408360608460076107d05a03fa9050805f8103610f9857fe5b5080610fd0576040517f4633be3200000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b505092915050565b610fe0611b2d565b610fe8611b67565b835f0151815f60048110610fff57610ffe6126c1565b5b602002018181525050836020015181600160048110611021576110206126c1565b5b602002018181525050825f015181600260048110611042576110416126c1565b5b602002018181525050826020015181600360048110611064576110636126c1565b5b6020020181815250505f60408360808460066107d05a03fa9050805f810361108857fe5b50806110c0576040517fd4b68fd700000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b505092915050565b6110d0611b89565b604051806040016040528060405180604001604052807f198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c281526020017f1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed815250815260200160405180604001604052807f275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec81526020017f1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d815250815250905090565b61119a611b2d565b6040518060400160405280600181526020016002815250905090565b6111be611b2d565b5f5f90505f5f90505f7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47855f1c6111f591906127fd565b90505b6001156112a157611208816117be565b80935081945050507f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd478061123f5761123e6127d0565b5b82830983036112675760405180604001604052808281526020018381525093505050506112bc565b7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4780611296576112956127d0565b5b6001820890506111f8565b60405180604001604052805f81526020015f81525093505050505b919050565b5f5f5f60405180604001604052808981526020018781525090505f60405180604001604052808981526020018781525090506112fb611baf565b5f5f90505b6002811015611519575f6006826113179190612c81565b905084826002811061132c5761132b6126c1565b5b60200201515f0151835f836113419190612cc2565b600c8110611352576113516126c1565b5b60200201818152505084826002811061136e5761136d6126c1565b5b602002015160200151836001836113859190612cc2565b600c8110611396576113956126c1565b5b6020020181815250508382600281106113b2576113b16126c1565b5b60200201515f01515f600281106113cc576113cb6126c1565b5b6020020151836002836113df9190612cc2565b600c81106113f0576113ef6126c1565b5b60200201818152505083826002811061140c5761140b6126c1565b5b60200201515f0151600160028110611427576114266126c1565b5b60200201518360038361143a9190612cc2565b600c811061144b5761144a6126c1565b5b602002018181525050838260028110611467576114666126c1565b5b6020020151602001515f60028110611482576114816126c1565b5b6020020151836004836114959190612cc2565b600c81106114a6576114a56126c1565b5b6020020181815250508382600281106114c2576114c16126c1565b5b6020020151602001516001600281106114de576114dd6126c1565b5b6020020151836005836114f19190612cc2565b600c8110611502576115016126c1565b5b602002018181525050508080600101915050611300565b50611522611bd2565b5f6020826020600c028560088cfa9050805f835f60018110611547576115466126c1565b5b602002015114159650965050505050509550959350505050565b5f5f61156c846118b3565b9050808360ff166001901b116115ae576040517fca95733300000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b8091505092915050565b5f81515f52816020015160205260405f209050919050565b5f5f5f90505b5f831115611602576001836115eb91906128bb565b8316925080806115fa90612d02565b9150506115d6565b80915050919050565b611613611b2d565b6102008261ffff1610611652576040517fff89d4fa00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b60018261ffff1603611666578290506116ea565b5f60405180604001604052805f81526020015f81525090505f8490505f600190505f5f90505b8161ffff168661ffff16106116e2576001808260ff168861ffff16901c1661ffff16036116c0576116bd8484610fd8565b93505b6116ca8384610fd8565b925060018261ffff16901b915080600101905061168c565b839450505050505b92915050565b6116f8611b2d565b5f825f015114801561170d57505f8260200151145b1561172e5760405180604001604052805f81526020015f81525090506117a3565b6040518060400160405280835f015181526020017f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47846020015161177291906127fd565b7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4761179d91906128bb565b81525090505b919050565b5f60018260ff1684901c16600114905092915050565b5f5f5f7f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47806117f0576117ef6127d0565b5b60037f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4780611821576118206127d0565b5b867f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4780611851576118506127d0565b5b888909090890505f6118a4827f0c19139cb84c680a6e14116da060561765e05aa45a1c72a34f082305b61f3f527f30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd476119bb565b90508181935093505050915091565b5f610100825111156118f1576040517ffb4a9c8e00000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b5f825103611901575f90506119b6565b5f5f835f81518110611916576119156126c1565b5b602001015160f81c60f81b60f81c60ff166001901b91505f600190505b84518110156119af5784818151811061194f5761194e6126c1565b5b602001015160f81c60f81b60f81c60ff166001901b915082821161199f576040517f80c8834800000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b8183179250806001019050611933565b5081925050505b919050565b5f5f6119c5611bd2565b6119cd611bf4565b6020815f600681106119e2576119e16126c1565b5b602002018181525050602081600160068110611a0157611a006126c1565b5b602002018181525050602081600260068110611a2057611a1f6126c1565b5b6020020181815250508681600360068110611a3e57611a3d6126c1565b5b6020020181815250508581600460068110611a5c57611a5b6126c1565b5b6020020181815250508481600560068110611a7a57611a796126c1565b5b60200201818152505060208260c08360056107d05a03fa9250825f8103611a9d57fe5b5082611ad5576040517fd51edae300000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b815f60018110611ae857611ae76126c1565b5b602002015193505050509392505050565b604051806040016040528060608152602001606081525090565b604051806040016040528060608152602001606081525090565b60405180604001604052805f81526020015f81525090565b6040518060600160405280600390602082028036833780820191505090505090565b6040518060800160405280600490602082028036833780820191505090505090565b6040518060400160405280611b9c611c16565b8152602001611ba9611c16565b81525090565b604051806101800160405280600c90602082028036833780820191505090505090565b6040518060200160405280600190602082028036833780820191505090505090565b6040518060c00160405280600690602082028036833780820191505090505090565b6040518060400160405280600290602082028036833780820191505090505090565b5f604051905090565b5f5ffd5b5f5ffd5b5f819050919050565b611c5b81611c49565b8114611c65575f5ffd5b50565b5f81359050611c7681611c52565b92915050565b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b611cc682611c80565b810181811067ffffffffffffffff82111715611ce557611ce4611c90565b5b80604052505050565b5f611cf7611c38565b9050611d038282611cbd565b919050565b5f5ffd5b5f819050919050565b611d1e81611d0c565b8114611d28575f5ffd5b50565b5f81359050611d3981611d15565b92915050565b5f60408284031215611d5457611d53611c7c565b5b611d5e6040611cee565b90505f611d6d84828501611d2b565b5f830152506020611d8084828501611d2b565b60208301525092915050565b5f5ffd5b5f67ffffffffffffffff821115611daa57611da9611c90565b5b602082029050919050565b5f5ffd5b5f611dcb611dc684611d90565b611cee565b90508060208402830185811115611de557611de4611db5565b5b835b81811015611e0e5780611dfa8882611d2b565b845260208401935050602081019050611de7565b5050509392505050565b5f82601f830112611e2c57611e2b611d8c565b5b6002611e39848285611db9565b91505092915050565b5f60808284031215611e5757611e56611c7c565b5b611e616040611cee565b90505f611e7084828501611e18565b5f830152506040611e8384828501611e18565b60208301525092915050565b5f5f5f5f6101208587031215611ea857611ea7611c41565b5b5f611eb587828801611c68565b9450506020611ec687828801611d3f565b9350506060611ed787828801611e42565b92505060e0611ee887828801611d3f565b91505092959194509250565b5f8115159050919050565b611f0881611ef4565b82525050565b5f604082019050611f215f830185611eff565b611f2e6020830184611eff565b9392505050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f819050919050565b5f611f77611f72611f6d84611f35565b611f54565b611f35565b9050919050565b5f611f8882611f5d565b9050919050565b5f611f9982611f7e565b9050919050565b611fa981611f8f565b82525050565b5f602082019050611fc25f830184611fa0565b92915050565b5f611fd282611f7e565b9050919050565b611fe281611fc8565b82525050565b5f602082019050611ffb5f830184611fd9565b92915050565b5f61200b82611f7e565b9050919050565b61201b81612001565b82525050565b5f6020820190506120345f830184612012565b92915050565b5f5ffd5b5f5f83601f84011261205357612052611d8c565b5b8235905067ffffffffffffffff8111156120705761206f61203a565b5b60208301915083600182028301111561208c5761208b611db5565b5b9250929050565b5f63ffffffff82169050919050565b6120ab81612093565b81146120b5575f5ffd5b50565b5f813590506120c6816120a2565b92915050565b5f67ffffffffffffffff8211156120e6576120e5611c90565b5b602082029050602081019050919050565b5f612109612104846120cc565b611cee565b9050808382526020820190506020840283018581111561212c5761212b611db5565b5b835b81811015612155578061214188826120b8565b84526020840193505060208101905061212e565b5050509392505050565b5f82601f83011261217357612172611d8c565b5b81356121838482602086016120f7565b91505092915050565b5f67ffffffffffffffff8211156121a6576121a5611c90565b5b602082029050602081019050919050565b5f6121c96121c48461218c565b611cee565b905080838252602082019050604084028301858111156121ec576121eb611db5565b5b835b8181101561221557806122018882611d3f565b8452602084019350506040810190506121ee565b5050509392505050565b5f82601f83011261223357612232611d8c565b5b81356122438482602086016121b7565b91505092915050565b5f67ffffffffffffffff82111561226657612265611c90565b5b602082029050602081019050919050565b5f6122896122848461224c565b611cee565b905080838252602082019050602084028301858111156122ac576122ab611db5565b5b835b818110156122f357803567ffffffffffffffff8111156122d1576122d0611d8c565b5b8086016122de898261215f565b855260208501945050506020810190506122ae565b5050509392505050565b5f82601f83011261231157612310611d8c565b5b8135612321848260208601612277565b91505092915050565b5f61018082840312156123405761233f611c7c565b5b61234b610100611cee565b90505f82013567ffffffffffffffff81111561236a57612369611d08565b5b6123768482850161215f565b5f83015250602082013567ffffffffffffffff81111561239957612398611d08565b5b6123a58482850161221f565b602083015250604082013567ffffffffffffffff8111156123c9576123c8611d08565b5b6123d58482850161221f565b60408301525060606123e984828501611e42565b60608301525060e06123fd84828501611d3f565b60808301525061012082013567ffffffffffffffff81111561242257612421611d08565b5b61242e8482850161215f565b60a08301525061014082013567ffffffffffffffff81111561245357612452611d08565b5b61245f8482850161215f565b60c08301525061016082013567ffffffffffffffff81111561248457612483611d08565b5b612490848285016122fd565b60e08301525092915050565b5f5f5f5f5f608086880312156124b5576124b4611c41565b5b5f6124c288828901611c68565b955050602086013567ffffffffffffffff8111156124e3576124e2611c45565b5b6124ef8882890161203e565b94509450506040612502888289016120b8565b925050606086013567ffffffffffffffff81111561252357612522611c45565b5b61252f8882890161232a565b9150509295509295909350565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b5f6bffffffffffffffffffffffff82169050919050565b61258581612565565b82525050565b5f612596838361257c565b60208301905092915050565b5f602082019050919050565b5f6125b88261253c565b6125c28185612546565b93506125cd83612556565b805f5b838110156125fd5781516125e4888261258b565b97506125ef836125a2565b9250506001810190506125d0565b5085935050505092915050565b5f604083015f8301518482035f86015261262482826125ae565b9150506020830151848203602086015261263e82826125ae565b9150508091505092915050565b61265481611c49565b82525050565b5f6040820190508181035f830152612672818561260a565b9050612681602083018461264b565b9392505050565b5f61269282611f7e565b9050919050565b6126a281612688565b82525050565b5f6020820190506126bb5f830184612699565b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffd5b5f819050919050565b61270861270382611c49565b6126ee565b82525050565b5f819050919050565b61272861272382611d0c565b61270e565b82525050565b5f612739828c6126f7565b602082019150612749828b612717565b602082019150612759828a612717565b6020820191506127698289612717565b6020820191506127798288612717565b6020820191506127898287612717565b6020820191506127998286612717565b6020820191506127a98285612717565b6020820191506127b98284612717565b6020820191508190509a9950505050505050505050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b5f61280782611d0c565b915061281283611d0c565b925082612822576128216127d0565b5b828206905092915050565b5f60ff82169050919050565b6128428161282d565b811461284c575f5ffd5b50565b5f8151905061285d81612839565b92915050565b5f6020828403121561287857612877611c41565b5b5f6128858482850161284f565b91505092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6128c582611d0c565b91506128d083611d0c565b92508282039050818111156128e8576128e761288e565b5b92915050565b6128f781612093565b82525050565b5f61291761291261290d84612093565b611f54565b611d0c565b9050919050565b612927816128fd565b82525050565b5f6060820190506129405f83018661264b565b61294d60208301856128ee565b61295a604083018461291e565b949350505050565b5f77ffffffffffffffffffffffffffffffffffffffffffffffff82169050919050565b61298e81612962565b8114612998575f5ffd5b50565b5f815190506129a981612985565b92915050565b5f602082840312156129c4576129c3611c41565b5b5f6129d18482850161299b565b91505092915050565b6129e38161282d565b82525050565b5f6060820190506129fc5f8301866129da565b612a0960208301856128ee565b612a16604083018461291e565b949350505050565b5f7fffffffffffffffffffffffffffffffffffffffffffffffff000000000000000082169050919050565b612a5281612a1e565b8114612a5c575f5ffd5b50565b5f81519050612a6d81612a49565b92915050565b5f60208284031215612a8857612a87611c41565b5b5f612a9584828501612a5f565b91505092915050565b612aa781612565565b8114612ab1575f5ffd5b50565b5f81519050612ac281612a9e565b92915050565b5f60208284031215612add57612adc611c41565b5b5f612aea84828501612ab4565b91505092915050565b5f608082019050612b065f8301876129da565b612b1360208301866128ee565b612b20604083018561264b565b612b2d606083018461291e565b95945050505050565b5f612b4082612565565b9150612b4b83612565565b925082820390506bffffffffffffffffffffffff811115612b6f57612b6e61288e565b5b92915050565b5f8160e01b9050919050565b5f612b8b82612b75565b9050919050565b612ba3612b9e82612093565b612b81565b82525050565b5f81519050919050565b5f81905092915050565b5f819050602082019050919050565b612bd581611c49565b82525050565b5f612be68383612bcc565b60208301905092915050565b5f602082019050919050565b5f612c0882612ba9565b612c128185612bb3565b9350612c1d83612bbd565b805f5b83811015612c4d578151612c348882612bdb565b9750612c3f83612bf2565b925050600181019050612c20565b5085935050505092915050565b5f612c658285612b92565b600482019150612c758284612bfe565b91508190509392505050565b5f612c8b82611d0c565b9150612c9683611d0c565b9250828202612ca481611d0c565b91508282048414831517612cbb57612cba61288e565b5b5092915050565b5f612ccc82611d0c565b9150612cd783611d0c565b9250828201905080821115612cef57612cee61288e565b5b92915050565b5f61ffff82169050919050565b5f612d0c82612cf5565b915061ffff8203612d2057612d1f61288e565b5b60018201905091905056fea264697066735822122009fafd6a9cb72c083d065be8b674687b4e0fc68c06baf5234eac5e2e4975cdc864736f6c634300081d0033
    /// ```
    #[rustfmt::skip]
    #[allow(clippy::all)]
    pub static DEPLOYED_BYTECODE: alloy_sol_types::private::Bytes = alloy_sol_types::private::Bytes::from_static(
        b"`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\0`W_5`\xE0\x1C\x80c\x17\x1F\x1D[\x14a\0dW\x80c]\xF4YF\x14a\0\x95W\x80ch0H5\x14a\0\xB3W\x80cm\x14\xA9\x87\x14a\0\xD1W\x80cn\xFBF6\x14a\0\xEFW\x80c\xDF\\\xF7#\x14a\x01 W[__\xFD[a\0~`\x04\x806\x03\x81\x01\x90a\0y\x91\x90a\x1E\x8FV[a\x01>V[`@Qa\0\x8C\x92\x91\x90a\x1F\x0EV[`@Q\x80\x91\x03\x90\xF3[a\0\x9Da\x02\xA1V[`@Qa\0\xAA\x91\x90a\x1F\xAFV[`@Q\x80\x91\x03\x90\xF3[a\0\xBBa\x02\xC5V[`@Qa\0\xC8\x91\x90a\x1F\xE8V[`@Q\x80\x91\x03\x90\xF3[a\0\xD9a\x02\xE9V[`@Qa\0\xE6\x91\x90a !V[`@Q\x80\x91\x03\x90\xF3[a\x01\t`\x04\x806\x03\x81\x01\x90a\x01\x04\x91\x90a$\x9CV[a\x03\rV[`@Qa\x01\x17\x92\x91\x90a&ZV[`@Q\x80\x91\x03\x90\xF3[a\x01(a\x0E\xE9V[`@Qa\x015\x91\x90a&\xA8V[`@Q\x80\x91\x03\x90\xF3[___\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X](3\xE8Hy\xB9p\x91C\xE1\xF5\x93\xF0\0\0\x01\x87\x87_\x01Q\x88` \x01Q\x88_\x01Q_`\x02\x81\x10a\x01\x82Wa\x01\x81a&\xC1V[[` \x02\x01Q\x89_\x01Q`\x01`\x02\x81\x10a\x01\x9EWa\x01\x9Da&\xC1V[[` \x02\x01Q\x8A` \x01Q_`\x02\x81\x10a\x01\xBAWa\x01\xB9a&\xC1V[[` \x02\x01Q\x8B` \x01Q`\x01`\x02\x81\x10a\x01\xD7Wa\x01\xD6a&\xC1V[[` \x02\x01Q\x8B_\x01Q\x8C` \x01Q`@Q` \x01a\x01\xFD\x99\x98\x97\x96\x95\x94\x93\x92\x91\x90a'.V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 _\x1Ca\x02\x1F\x91\x90a'\xFDV[\x90Pa\x02\x8Fa\x02Ia\x02:\x83\x89a\x0F\r\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x86a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[a\x02Qa\x10\xC8V[a\x02\x85a\x02n\x85a\x02`a\x11\x92V[a\x0F\r\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[a\x02w\x8Ca\x11\xB6V[a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x88b\x01\xD4\xC0a\x12\xC1V[\x80\x93P\x81\x94PPPP\x94P\x94\x92PPPV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03\x15a\x1A\xF9V[__\x86\x86\x90P\x03a\x03RW`@Q\x7F\x1F\x04\x05\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82`@\x01QQ\x86\x86\x90P\x14\x80\x15a\x03pWP\x82`\xA0\x01QQ\x86\x86\x90P\x14[\x80\x15a\x03\x83WP\x82`\xC0\x01QQ\x86\x86\x90P\x14[\x80\x15a\x03\x96WP\x82`\xE0\x01QQ\x86\x86\x90P\x14[a\x03\xCCW`@Q\x7FCqJ\xFD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82_\x01QQ\x83` \x01QQ\x14a\x04\x0EW`@Q\x7F_\x83/A\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[Cc\xFF\xFF\xFF\xFF\x16\x84c\xFF\xFF\xFF\xFF\x16\x10a\x04SW`@Q\x7FK\x87OE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90Pa\x04sa\x1A\xF9V[\x87\x87\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x04\x90Wa\x04\x8Fa\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x04\xBEW\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81` \x01\x81\x90RP\x87\x87\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x04\xE4Wa\x04\xE3a\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05\x12W\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81_\x01\x81\x90RPa\x05\"a\x1B\x13V[\x85` \x01QQg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x05AWa\x05@a\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05oW\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81_\x01\x81\x90RP\x85` \x01QQg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x05\x96Wa\x05\x95a\x1C\x90V[[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05\xC4W\x81` \x01` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90P[P\x81` \x01\x81\x90RP_a\x06\xA7\x8A\x8A\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847_\x81\x84\x01R`\x1F\x19`\x1F\x82\x01\x16\x90P\x80\x83\x01\x92PPPPPPP\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\x9A\xA1e=`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x06~W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x06\xA2\x91\x90a(cV[a\x15aV[\x90P__\x90P[\x87` \x01QQ\x81\x10\x15a\t\x1EWa\x06\xE2\x88` \x01Q\x82\x81Q\x81\x10a\x06\xD5Wa\x06\xD4a&\xC1V[[` \x02` \x01\x01Qa\x15\xB8V[\x83` \x01Q\x82\x81Q\x81\x10a\x06\xF9Wa\x06\xF8a&\xC1V[[` \x02` \x01\x01\x81\x81RPP_\x81\x14a\x07\x92W\x82` \x01Q`\x01\x82a\x07\x1E\x91\x90a(\xBBV[\x81Q\x81\x10a\x07/Wa\x07.a&\xC1V[[` \x02` \x01\x01Q_\x1C\x83` \x01Q\x82\x81Q\x81\x10a\x07PWa\x07Oa&\xC1V[[` \x02` \x01\x01Q_\x1C\x11a\x07\x91W`@Q\x7F\xFFq\x94\x14\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\x04\xECcQ\x84` \x01Q\x83\x81Q\x81\x10a\x07\xE5Wa\x07\xE4a&\xC1V[[` \x02` \x01\x01Q\x8B\x8B_\x01Q\x85\x81Q\x81\x10a\x08\x04Wa\x08\x03a&\xC1V[[` \x02` \x01\x01Q`@Q\x84c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x08*\x93\x92\x91\x90a)-V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x08EW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x08i\x91\x90a)\xAFV[w\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x83_\x01Q\x82\x81Q\x81\x10a\x08\x99Wa\x08\x98a&\xC1V[[` \x02` \x01\x01\x81\x81RPPa\t\x0Fa\t\0a\x08\xD3\x84\x86_\x01Q\x85\x81Q\x81\x10a\x08\xC5Wa\x08\xC4a&\xC1V[[` \x02` \x01\x01Q\x16a\x15\xD0V[\x8A` \x01Q\x84\x81Q\x81\x10a\x08\xEAWa\x08\xE9a&\xC1V[[` \x02` \x01\x01Qa\x16\x0B\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x86a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x94P\x80\x80`\x01\x01\x91PPa\x06\xAEV[PPa\t)\x83a\x16\xF0V[\x92P__\x90P[\x89\x89\x90P\x81\x10\x15a\x0E\x1AW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16ch\xBC\xCA\xAC\x8B\x8B\x84\x81\x81\x10a\t\x8AWa\t\x89a&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1C\x8A\x8A`\xA0\x01Q\x85\x81Q\x81\x10a\t\xAFWa\t\xAEa&\xC1V[[` \x02` \x01\x01Q`@Q\x84c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\t\xD5\x93\x92\x91\x90a)\xE9V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\t\xF0W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\n\x14\x91\x90a*sV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16a\nF\x88`@\x01Q\x83\x81Q\x81\x10a\n9Wa\n8a&\xC1V[[` \x02` \x01\x01Qa\x15\xB8V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16\x14a\n\x88W`@Q\x7F\xE11\n\xED\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\n\xB9\x87`@\x01Q\x82\x81Q\x81\x10a\n\xA2Wa\n\xA1a&\xC1V[[` \x02` \x01\x01Q\x85a\x0F\xD8\x90\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x93P\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\xC8)LV\x8B\x8B\x84\x81\x81\x10a\x0B\nWa\x0B\ta&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1C\x8A\x8A`\xC0\x01Q\x85\x81Q\x81\x10a\x0B/Wa\x0B.a&\xC1V[[` \x02` \x01\x01Q`@Q\x84c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x0BU\x93\x92\x91\x90a)\xE9V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0BpW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0B\x94\x91\x90a*\xC8V[\x83` \x01Q\x82\x81Q\x81\x10a\x0B\xABWa\x0B\xAAa&\xC1V[[` \x02` \x01\x01\x90k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x81k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP\x82` \x01Q\x81\x81Q\x81\x10a\x0B\xECWa\x0B\xEBa&\xC1V[[` \x02` \x01\x01Q\x83_\x01Q\x82\x81Q\x81\x10a\x0C\nWa\x0C\ta&\xC1V[[` \x02` \x01\x01\x90k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x81k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP__\x90P__\x90P[\x88` \x01QQ\x81\x10\x15a\x0E\x0BWa\x0C\x90\x84_\x01Q\x82\x81Q\x81\x10a\x0CcWa\x0Cba&\xC1V[[` \x02` \x01\x01Q\x8D\x8D\x86\x81\x81\x10a\x0C~Wa\x0C}a&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1Ca\x17\xA8V[\x15a\r\xFEW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\xF2\xBE\x94\xAE\x8D\x8D\x86\x81\x81\x10a\x0C\xE4Wa\x0C\xE3a&\xC1V[[\x90P\x015`\xF8\x1C`\xF8\x1B`\xF8\x1C\x8C\x87` \x01Q\x85\x81Q\x81\x10a\r\tWa\r\x08a&\xC1V[[` \x02` \x01\x01Q\x8D`\xE0\x01Q\x88\x81Q\x81\x10a\r(Wa\r'a&\xC1V[[` \x02` \x01\x01Q\x87\x81Q\x81\x10a\rBWa\rAa&\xC1V[[` \x02` \x01\x01Q`@Q\x85c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\ri\x94\x93\x92\x91\x90a*\xF3V[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\r\x84W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\r\xA8\x91\x90a*\xC8V[\x85_\x01Q\x84\x81Q\x81\x10a\r\xBEWa\r\xBDa&\xC1V[[` \x02` \x01\x01\x81\x81Qa\r\xD2\x91\x90a+6V[\x91P\x90k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x81k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81RPP\x81`\x01\x01\x91P[\x80\x80`\x01\x01\x91PPa\x0C=V[PP\x80\x80`\x01\x01\x91PPa\t0V[P__a\x0E1\x8C\x86\x8A``\x01Q\x8B`\x80\x01Qa\x01>V[\x91P\x91P\x81a\x0ElW`@Q\x7Fg\x98\x8D3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80a\x0E\xA3W`@Q\x7F\xAB\x1B#k\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PP_\x87\x82` \x01Q`@Q` \x01a\x0E\xBD\x92\x91\x90a,ZV[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P\x82\x81\x95P\x95PPPPP\x95P\x95\x93PPPPV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x0F\x15a\x1B-V[a\x0F\x1Da\x1BEV[\x83_\x01Q\x81_`\x03\x81\x10a\x0F4Wa\x0F3a&\xC1V[[` \x02\x01\x81\x81RPP\x83` \x01Q\x81`\x01`\x03\x81\x10a\x0FVWa\x0FUa&\xC1V[[` \x02\x01\x81\x81RPP\x82\x81`\x02`\x03\x81\x10a\x0FtWa\x0Fsa&\xC1V[[` \x02\x01\x81\x81RPP_`@\x83``\x84`\x07a\x07\xD0Z\x03\xFA\x90P\x80_\x81\x03a\x0F\x98W\xFE[P\x80a\x0F\xD0W`@Q\x7FF3\xBE2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PP\x92\x91PPV[a\x0F\xE0a\x1B-V[a\x0F\xE8a\x1BgV[\x83_\x01Q\x81_`\x04\x81\x10a\x0F\xFFWa\x0F\xFEa&\xC1V[[` \x02\x01\x81\x81RPP\x83` \x01Q\x81`\x01`\x04\x81\x10a\x10!Wa\x10 a&\xC1V[[` \x02\x01\x81\x81RPP\x82_\x01Q\x81`\x02`\x04\x81\x10a\x10BWa\x10Aa&\xC1V[[` \x02\x01\x81\x81RPP\x82` \x01Q\x81`\x03`\x04\x81\x10a\x10dWa\x10ca&\xC1V[[` \x02\x01\x81\x81RPP_`@\x83`\x80\x84`\x06a\x07\xD0Z\x03\xFA\x90P\x80_\x81\x03a\x10\x88W\xFE[P\x80a\x10\xC0W`@Q\x7F\xD4\xB6\x8F\xD7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PP\x92\x91PPV[a\x10\xD0a\x1B\x89V[`@Q\x80`@\x01`@R\x80`@Q\x80`@\x01`@R\x80\x7F\x19\x8E\x93\x93\x92\rH:r`\xBF\xB71\xFB]%\xF1\xAAI35\xA9\xE7\x12\x97\xE4\x85\xB7\xAE\xF3\x12\xC2\x81R` \x01\x7F\x18\0\xDE\xEF\x12\x1F\x1EvBj\0f^\\DygC\"\xD4\xF7^\xDA\xDDF\xDE\xBD\\\xD9\x92\xF6\xED\x81RP\x81R` \x01`@Q\x80`@\x01`@R\x80\x7F']\xC4\xA2\x88\xD1\xAF\xB3\xCB\xB1\xAC\t\x18u$\xC7\xDB69]\xF7\xBE;\x99\xE6s\xB1:\x07Ze\xEC\x81R` \x01\x7F\x1D\x9B\xEF\xCD\x05\xA52>m\xA4\xD45\xF3\xB6\x17\xCD\xB3\xAF\x83(\\-\xF7\x11\xEF9\xC0\x15q\x82\x7F\x9D\x81RP\x81RP\x90P\x90V[a\x11\x9Aa\x1B-V[`@Q\x80`@\x01`@R\x80`\x01\x81R` \x01`\x02\x81RP\x90P\x90V[a\x11\xBEa\x1B-V[__\x90P__\x90P_\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x85_\x1Ca\x11\xF5\x91\x90a'\xFDV[\x90P[`\x01\x15a\x12\xA1Wa\x12\x08\x81a\x17\xBEV[\x80\x93P\x81\x94PPP\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x12?Wa\x12>a'\xD0V[[\x82\x83\t\x83\x03a\x12gW`@Q\x80`@\x01`@R\x80\x82\x81R` \x01\x83\x81RP\x93PPPPa\x12\xBCV[\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x12\x96Wa\x12\x95a'\xD0V[[`\x01\x82\x08\x90Pa\x11\xF8V[`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x93PPPP[\x91\x90PV[___`@Q\x80`@\x01`@R\x80\x89\x81R` \x01\x87\x81RP\x90P_`@Q\x80`@\x01`@R\x80\x89\x81R` \x01\x87\x81RP\x90Pa\x12\xFBa\x1B\xAFV[__\x90P[`\x02\x81\x10\x15a\x15\x19W_`\x06\x82a\x13\x17\x91\x90a,\x81V[\x90P\x84\x82`\x02\x81\x10a\x13,Wa\x13+a&\xC1V[[` \x02\x01Q_\x01Q\x83_\x83a\x13A\x91\x90a,\xC2V[`\x0C\x81\x10a\x13RWa\x13Qa&\xC1V[[` \x02\x01\x81\x81RPP\x84\x82`\x02\x81\x10a\x13nWa\x13ma&\xC1V[[` \x02\x01Q` \x01Q\x83`\x01\x83a\x13\x85\x91\x90a,\xC2V[`\x0C\x81\x10a\x13\x96Wa\x13\x95a&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x13\xB2Wa\x13\xB1a&\xC1V[[` \x02\x01Q_\x01Q_`\x02\x81\x10a\x13\xCCWa\x13\xCBa&\xC1V[[` \x02\x01Q\x83`\x02\x83a\x13\xDF\x91\x90a,\xC2V[`\x0C\x81\x10a\x13\xF0Wa\x13\xEFa&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x14\x0CWa\x14\x0Ba&\xC1V[[` \x02\x01Q_\x01Q`\x01`\x02\x81\x10a\x14'Wa\x14&a&\xC1V[[` \x02\x01Q\x83`\x03\x83a\x14:\x91\x90a,\xC2V[`\x0C\x81\x10a\x14KWa\x14Ja&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x14gWa\x14fa&\xC1V[[` \x02\x01Q` \x01Q_`\x02\x81\x10a\x14\x82Wa\x14\x81a&\xC1V[[` \x02\x01Q\x83`\x04\x83a\x14\x95\x91\x90a,\xC2V[`\x0C\x81\x10a\x14\xA6Wa\x14\xA5a&\xC1V[[` \x02\x01\x81\x81RPP\x83\x82`\x02\x81\x10a\x14\xC2Wa\x14\xC1a&\xC1V[[` \x02\x01Q` \x01Q`\x01`\x02\x81\x10a\x14\xDEWa\x14\xDDa&\xC1V[[` \x02\x01Q\x83`\x05\x83a\x14\xF1\x91\x90a,\xC2V[`\x0C\x81\x10a\x15\x02Wa\x15\x01a&\xC1V[[` \x02\x01\x81\x81RPPP\x80\x80`\x01\x01\x91PPa\x13\0V[Pa\x15\"a\x1B\xD2V[_` \x82` `\x0C\x02\x85`\x08\x8C\xFA\x90P\x80_\x83_`\x01\x81\x10a\x15GWa\x15Fa&\xC1V[[` \x02\x01Q\x14\x15\x96P\x96PPPPPP\x95P\x95\x93PPPPV[__a\x15l\x84a\x18\xB3V[\x90P\x80\x83`\xFF\x16`\x01\x90\x1B\x11a\x15\xAEW`@Q\x7F\xCA\x95s3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80\x91PP\x92\x91PPV[_\x81Q_R\x81` \x01Q` R`@_ \x90P\x91\x90PV[___\x90P[_\x83\x11\x15a\x16\x02W`\x01\x83a\x15\xEB\x91\x90a(\xBBV[\x83\x16\x92P\x80\x80a\x15\xFA\x90a-\x02V[\x91PPa\x15\xD6V[\x80\x91PP\x91\x90PV[a\x16\x13a\x1B-V[a\x02\0\x82a\xFF\xFF\x16\x10a\x16RW`@Q\x7F\xFF\x89\xD4\xFA\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x82a\xFF\xFF\x16\x03a\x16fW\x82\x90Pa\x16\xEAV[_`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90P_\x84\x90P_`\x01\x90P__\x90P[\x81a\xFF\xFF\x16\x86a\xFF\xFF\x16\x10a\x16\xE2W`\x01\x80\x82`\xFF\x16\x88a\xFF\xFF\x16\x90\x1C\x16a\xFF\xFF\x16\x03a\x16\xC0Wa\x16\xBD\x84\x84a\x0F\xD8V[\x93P[a\x16\xCA\x83\x84a\x0F\xD8V[\x92P`\x01\x82a\xFF\xFF\x16\x90\x1B\x91P\x80`\x01\x01\x90Pa\x16\x8CV[\x83\x94PPPPP[\x92\x91PPV[a\x16\xF8a\x1B-V[_\x82_\x01Q\x14\x80\x15a\x17\rWP_\x82` \x01Q\x14[\x15a\x17.W`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90Pa\x17\xA3V[`@Q\x80`@\x01`@R\x80\x83_\x01Q\x81R` \x01\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x84` \x01Qa\x17r\x91\x90a'\xFDV[\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDGa\x17\x9D\x91\x90a(\xBBV[\x81RP\x90P[\x91\x90PV[_`\x01\x82`\xFF\x16\x84\x90\x1C\x16`\x01\x14\x90P\x92\x91PPV[___\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x17\xF0Wa\x17\xEFa'\xD0V[[`\x03\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x18!Wa\x18 a'\xD0V[[\x86\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDG\x80a\x18QWa\x18Pa'\xD0V[[\x88\x89\t\t\x08\x90P_a\x18\xA4\x82\x7F\x0C\x19\x13\x9C\xB8Lh\nn\x14\x11m\xA0`V\x17e\xE0Z\xA4Z\x1Cr\xA3O\x08#\x05\xB6\x1F?R\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X]\x97\x81j\x91hq\xCA\x8D< \x8C\x16\xD8|\xFDGa\x19\xBBV[\x90P\x81\x81\x93P\x93PPP\x91P\x91V[_a\x01\0\x82Q\x11\x15a\x18\xF1W`@Q\x7F\xFBJ\x9C\x8E\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_\x82Q\x03a\x19\x01W_\x90Pa\x19\xB6V[__\x83_\x81Q\x81\x10a\x19\x16Wa\x19\x15a&\xC1V[[` \x01\x01Q`\xF8\x1C`\xF8\x1B`\xF8\x1C`\xFF\x16`\x01\x90\x1B\x91P_`\x01\x90P[\x84Q\x81\x10\x15a\x19\xAFW\x84\x81\x81Q\x81\x10a\x19OWa\x19Na&\xC1V[[` \x01\x01Q`\xF8\x1C`\xF8\x1B`\xF8\x1C`\xFF\x16`\x01\x90\x1B\x91P\x82\x82\x11a\x19\x9FW`@Q\x7F\x80\xC8\x83H\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x81\x83\x17\x92P\x80`\x01\x01\x90Pa\x193V[P\x81\x92PPP[\x91\x90PV[__a\x19\xC5a\x1B\xD2V[a\x19\xCDa\x1B\xF4V[` \x81_`\x06\x81\x10a\x19\xE2Wa\x19\xE1a&\xC1V[[` \x02\x01\x81\x81RPP` \x81`\x01`\x06\x81\x10a\x1A\x01Wa\x1A\0a&\xC1V[[` \x02\x01\x81\x81RPP` \x81`\x02`\x06\x81\x10a\x1A Wa\x1A\x1Fa&\xC1V[[` \x02\x01\x81\x81RPP\x86\x81`\x03`\x06\x81\x10a\x1A>Wa\x1A=a&\xC1V[[` \x02\x01\x81\x81RPP\x85\x81`\x04`\x06\x81\x10a\x1A\\Wa\x1A[a&\xC1V[[` \x02\x01\x81\x81RPP\x84\x81`\x05`\x06\x81\x10a\x1AzWa\x1Aya&\xC1V[[` \x02\x01\x81\x81RPP` \x82`\xC0\x83`\x05a\x07\xD0Z\x03\xFA\x92P\x82_\x81\x03a\x1A\x9DW\xFE[P\x82a\x1A\xD5W`@Q\x7F\xD5\x1E\xDA\xE3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x81_`\x01\x81\x10a\x1A\xE8Wa\x1A\xE7a&\xC1V[[` \x02\x01Q\x93PPPP\x93\x92PPPV[`@Q\x80`@\x01`@R\x80``\x81R` \x01``\x81RP\x90V[`@Q\x80`@\x01`@R\x80``\x81R` \x01``\x81RP\x90V[`@Q\x80`@\x01`@R\x80_\x81R` \x01_\x81RP\x90V[`@Q\x80``\x01`@R\x80`\x03\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`\x80\x01`@R\x80`\x04\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`@\x01`@R\x80a\x1B\x9Ca\x1C\x16V[\x81R` \x01a\x1B\xA9a\x1C\x16V[\x81RP\x90V[`@Q\x80a\x01\x80\x01`@R\x80`\x0C\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80` \x01`@R\x80`\x01\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`\xC0\x01`@R\x80`\x06\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[`@Q\x80`@\x01`@R\x80`\x02\x90` \x82\x02\x806\x837\x80\x82\x01\x91PP\x90PP\x90V[_`@Q\x90P\x90V[__\xFD[__\xFD[_\x81\x90P\x91\x90PV[a\x1C[\x81a\x1CIV[\x81\x14a\x1CeW__\xFD[PV[_\x815\x90Pa\x1Cv\x81a\x1CRV[\x92\x91PPV[__\xFD[_`\x1F\x19`\x1F\x83\x01\x16\x90P\x91\x90PV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`A`\x04R`$_\xFD[a\x1C\xC6\x82a\x1C\x80V[\x81\x01\x81\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17\x15a\x1C\xE5Wa\x1C\xE4a\x1C\x90V[[\x80`@RPPPV[_a\x1C\xF7a\x1C8V[\x90Pa\x1D\x03\x82\x82a\x1C\xBDV[\x91\x90PV[__\xFD[_\x81\x90P\x91\x90PV[a\x1D\x1E\x81a\x1D\x0CV[\x81\x14a\x1D(W__\xFD[PV[_\x815\x90Pa\x1D9\x81a\x1D\x15V[\x92\x91PPV[_`@\x82\x84\x03\x12\x15a\x1DTWa\x1DSa\x1C|V[[a\x1D^`@a\x1C\xEEV[\x90P_a\x1Dm\x84\x82\x85\x01a\x1D+V[_\x83\x01RP` a\x1D\x80\x84\x82\x85\x01a\x1D+V[` \x83\x01RP\x92\x91PPV[__\xFD[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a\x1D\xAAWa\x1D\xA9a\x1C\x90V[[` \x82\x02\x90P\x91\x90PV[__\xFD[_a\x1D\xCBa\x1D\xC6\x84a\x1D\x90V[a\x1C\xEEV[\x90P\x80` \x84\x02\x83\x01\x85\x81\x11\x15a\x1D\xE5Wa\x1D\xE4a\x1D\xB5V[[\x83[\x81\x81\x10\x15a\x1E\x0EW\x80a\x1D\xFA\x88\x82a\x1D+V[\x84R` \x84\x01\x93PP` \x81\x01\x90Pa\x1D\xE7V[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a\x1E,Wa\x1E+a\x1D\x8CV[[`\x02a\x1E9\x84\x82\x85a\x1D\xB9V[\x91PP\x92\x91PPV[_`\x80\x82\x84\x03\x12\x15a\x1EWWa\x1EVa\x1C|V[[a\x1Ea`@a\x1C\xEEV[\x90P_a\x1Ep\x84\x82\x85\x01a\x1E\x18V[_\x83\x01RP`@a\x1E\x83\x84\x82\x85\x01a\x1E\x18V[` \x83\x01RP\x92\x91PPV[____a\x01 \x85\x87\x03\x12\x15a\x1E\xA8Wa\x1E\xA7a\x1CAV[[_a\x1E\xB5\x87\x82\x88\x01a\x1ChV[\x94PP` a\x1E\xC6\x87\x82\x88\x01a\x1D?V[\x93PP``a\x1E\xD7\x87\x82\x88\x01a\x1EBV[\x92PP`\xE0a\x1E\xE8\x87\x82\x88\x01a\x1D?V[\x91PP\x92\x95\x91\x94P\x92PV[_\x81\x15\x15\x90P\x91\x90PV[a\x1F\x08\x81a\x1E\xF4V[\x82RPPV[_`@\x82\x01\x90Pa\x1F!_\x83\x01\x85a\x1E\xFFV[a\x1F.` \x83\x01\x84a\x1E\xFFV[\x93\x92PPPV[_s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[_\x81\x90P\x91\x90PV[_a\x1Fwa\x1Fra\x1Fm\x84a\x1F5V[a\x1FTV[a\x1F5V[\x90P\x91\x90PV[_a\x1F\x88\x82a\x1F]V[\x90P\x91\x90PV[_a\x1F\x99\x82a\x1F~V[\x90P\x91\x90PV[a\x1F\xA9\x81a\x1F\x8FV[\x82RPPV[_` \x82\x01\x90Pa\x1F\xC2_\x83\x01\x84a\x1F\xA0V[\x92\x91PPV[_a\x1F\xD2\x82a\x1F~V[\x90P\x91\x90PV[a\x1F\xE2\x81a\x1F\xC8V[\x82RPPV[_` \x82\x01\x90Pa\x1F\xFB_\x83\x01\x84a\x1F\xD9V[\x92\x91PPV[_a \x0B\x82a\x1F~V[\x90P\x91\x90PV[a \x1B\x81a \x01V[\x82RPPV[_` \x82\x01\x90Pa 4_\x83\x01\x84a \x12V[\x92\x91PPV[__\xFD[__\x83`\x1F\x84\x01\x12a SWa Ra\x1D\x8CV[[\x825\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a pWa oa :V[[` \x83\x01\x91P\x83`\x01\x82\x02\x83\x01\x11\x15a \x8CWa \x8Ba\x1D\xB5V[[\x92P\x92\x90PV[_c\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[a \xAB\x81a \x93V[\x81\x14a \xB5W__\xFD[PV[_\x815\x90Pa \xC6\x81a \xA2V[\x92\x91PPV[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a \xE6Wa \xE5a\x1C\x90V[[` \x82\x02\x90P` \x81\x01\x90P\x91\x90PV[_a!\ta!\x04\x84a \xCCV[a\x1C\xEEV[\x90P\x80\x83\x82R` \x82\x01\x90P` \x84\x02\x83\x01\x85\x81\x11\x15a!,Wa!+a\x1D\xB5V[[\x83[\x81\x81\x10\x15a!UW\x80a!A\x88\x82a \xB8V[\x84R` \x84\x01\x93PP` \x81\x01\x90Pa!.V[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a!sWa!ra\x1D\x8CV[[\x815a!\x83\x84\x82` \x86\x01a \xF7V[\x91PP\x92\x91PPV[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a!\xA6Wa!\xA5a\x1C\x90V[[` \x82\x02\x90P` \x81\x01\x90P\x91\x90PV[_a!\xC9a!\xC4\x84a!\x8CV[a\x1C\xEEV[\x90P\x80\x83\x82R` \x82\x01\x90P`@\x84\x02\x83\x01\x85\x81\x11\x15a!\xECWa!\xEBa\x1D\xB5V[[\x83[\x81\x81\x10\x15a\"\x15W\x80a\"\x01\x88\x82a\x1D?V[\x84R` \x84\x01\x93PP`@\x81\x01\x90Pa!\xEEV[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a\"3Wa\"2a\x1D\x8CV[[\x815a\"C\x84\x82` \x86\x01a!\xB7V[\x91PP\x92\x91PPV[_g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a\"fWa\"ea\x1C\x90V[[` \x82\x02\x90P` \x81\x01\x90P\x91\x90PV[_a\"\x89a\"\x84\x84a\"LV[a\x1C\xEEV[\x90P\x80\x83\x82R` \x82\x01\x90P` \x84\x02\x83\x01\x85\x81\x11\x15a\"\xACWa\"\xABa\x1D\xB5V[[\x83[\x81\x81\x10\x15a\"\xF3W\x805g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\"\xD1Wa\"\xD0a\x1D\x8CV[[\x80\x86\x01a\"\xDE\x89\x82a!_V[\x85R` \x85\x01\x94PPP` \x81\x01\x90Pa\"\xAEV[PPP\x93\x92PPPV[_\x82`\x1F\x83\x01\x12a#\x11Wa#\x10a\x1D\x8CV[[\x815a#!\x84\x82` \x86\x01a\"wV[\x91PP\x92\x91PPV[_a\x01\x80\x82\x84\x03\x12\x15a#@Wa#?a\x1C|V[[a#Ka\x01\0a\x1C\xEEV[\x90P_\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a#jWa#ia\x1D\x08V[[a#v\x84\x82\x85\x01a!_V[_\x83\x01RP` \x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a#\x99Wa#\x98a\x1D\x08V[[a#\xA5\x84\x82\x85\x01a\"\x1FV[` \x83\x01RP`@\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a#\xC9Wa#\xC8a\x1D\x08V[[a#\xD5\x84\x82\x85\x01a\"\x1FV[`@\x83\x01RP``a#\xE9\x84\x82\x85\x01a\x1EBV[``\x83\x01RP`\xE0a#\xFD\x84\x82\x85\x01a\x1D?V[`\x80\x83\x01RPa\x01 \x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$\"Wa$!a\x1D\x08V[[a$.\x84\x82\x85\x01a!_V[`\xA0\x83\x01RPa\x01@\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$SWa$Ra\x1D\x08V[[a$_\x84\x82\x85\x01a!_V[`\xC0\x83\x01RPa\x01`\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$\x84Wa$\x83a\x1D\x08V[[a$\x90\x84\x82\x85\x01a\"\xFDV[`\xE0\x83\x01RP\x92\x91PPV[_____`\x80\x86\x88\x03\x12\x15a$\xB5Wa$\xB4a\x1CAV[[_a$\xC2\x88\x82\x89\x01a\x1ChV[\x95PP` \x86\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a$\xE3Wa$\xE2a\x1CEV[[a$\xEF\x88\x82\x89\x01a >V[\x94P\x94PP`@a%\x02\x88\x82\x89\x01a \xB8V[\x92PP``\x86\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a%#Wa%\"a\x1CEV[[a%/\x88\x82\x89\x01a#*V[\x91PP\x92\x95P\x92\x95\x90\x93PV[_\x81Q\x90P\x91\x90PV[_\x82\x82R` \x82\x01\x90P\x92\x91PPV[_\x81\x90P` \x82\x01\x90P\x91\x90PV[_k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[a%\x85\x81a%eV[\x82RPPV[_a%\x96\x83\x83a%|V[` \x83\x01\x90P\x92\x91PPV[_` \x82\x01\x90P\x91\x90PV[_a%\xB8\x82a%<V[a%\xC2\x81\x85a%FV[\x93Pa%\xCD\x83a%VV[\x80_[\x83\x81\x10\x15a%\xFDW\x81Qa%\xE4\x88\x82a%\x8BV[\x97Pa%\xEF\x83a%\xA2V[\x92PP`\x01\x81\x01\x90Pa%\xD0V[P\x85\x93PPPP\x92\x91PPV[_`@\x83\x01_\x83\x01Q\x84\x82\x03_\x86\x01Ra&$\x82\x82a%\xAEV[\x91PP` \x83\x01Q\x84\x82\x03` \x86\x01Ra&>\x82\x82a%\xAEV[\x91PP\x80\x91PP\x92\x91PPV[a&T\x81a\x1CIV[\x82RPPV[_`@\x82\x01\x90P\x81\x81\x03_\x83\x01Ra&r\x81\x85a&\nV[\x90Pa&\x81` \x83\x01\x84a&KV[\x93\x92PPPV[_a&\x92\x82a\x1F~V[\x90P\x91\x90PV[a&\xA2\x81a&\x88V[\x82RPPV[_` \x82\x01\x90Pa&\xBB_\x83\x01\x84a&\x99V[\x92\x91PPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`2`\x04R`$_\xFD[_\x81\x90P\x91\x90PV[a'\x08a'\x03\x82a\x1CIV[a&\xEEV[\x82RPPV[_\x81\x90P\x91\x90PV[a'(a'#\x82a\x1D\x0CV[a'\x0EV[\x82RPPV[_a'9\x82\x8Ca&\xF7V[` \x82\x01\x91Pa'I\x82\x8Ba'\x17V[` \x82\x01\x91Pa'Y\x82\x8Aa'\x17V[` \x82\x01\x91Pa'i\x82\x89a'\x17V[` \x82\x01\x91Pa'y\x82\x88a'\x17V[` \x82\x01\x91Pa'\x89\x82\x87a'\x17V[` \x82\x01\x91Pa'\x99\x82\x86a'\x17V[` \x82\x01\x91Pa'\xA9\x82\x85a'\x17V[` \x82\x01\x91Pa'\xB9\x82\x84a'\x17V[` \x82\x01\x91P\x81\x90P\x9A\x99PPPPPPPPPPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x12`\x04R`$_\xFD[_a(\x07\x82a\x1D\x0CV[\x91Pa(\x12\x83a\x1D\x0CV[\x92P\x82a(\"Wa(!a'\xD0V[[\x82\x82\x06\x90P\x92\x91PPV[_`\xFF\x82\x16\x90P\x91\x90PV[a(B\x81a(-V[\x81\x14a(LW__\xFD[PV[_\x81Q\x90Pa(]\x81a(9V[\x92\x91PPV[_` \x82\x84\x03\x12\x15a(xWa(wa\x1CAV[[_a(\x85\x84\x82\x85\x01a(OV[\x91PP\x92\x91PPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x11`\x04R`$_\xFD[_a(\xC5\x82a\x1D\x0CV[\x91Pa(\xD0\x83a\x1D\x0CV[\x92P\x82\x82\x03\x90P\x81\x81\x11\x15a(\xE8Wa(\xE7a(\x8EV[[\x92\x91PPV[a(\xF7\x81a \x93V[\x82RPPV[_a)\x17a)\x12a)\r\x84a \x93V[a\x1FTV[a\x1D\x0CV[\x90P\x91\x90PV[a)'\x81a(\xFDV[\x82RPPV[_``\x82\x01\x90Pa)@_\x83\x01\x86a&KV[a)M` \x83\x01\x85a(\xEEV[a)Z`@\x83\x01\x84a)\x1EV[\x94\x93PPPPV[_w\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90P\x91\x90PV[a)\x8E\x81a)bV[\x81\x14a)\x98W__\xFD[PV[_\x81Q\x90Pa)\xA9\x81a)\x85V[\x92\x91PPV[_` \x82\x84\x03\x12\x15a)\xC4Wa)\xC3a\x1CAV[[_a)\xD1\x84\x82\x85\x01a)\x9BV[\x91PP\x92\x91PPV[a)\xE3\x81a(-V[\x82RPPV[_``\x82\x01\x90Pa)\xFC_\x83\x01\x86a)\xDAV[a*\t` \x83\x01\x85a(\xEEV[a*\x16`@\x83\x01\x84a)\x1EV[\x94\x93PPPPV[_\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x82\x16\x90P\x91\x90PV[a*R\x81a*\x1EV[\x81\x14a*\\W__\xFD[PV[_\x81Q\x90Pa*m\x81a*IV[\x92\x91PPV[_` \x82\x84\x03\x12\x15a*\x88Wa*\x87a\x1CAV[[_a*\x95\x84\x82\x85\x01a*_V[\x91PP\x92\x91PPV[a*\xA7\x81a%eV[\x81\x14a*\xB1W__\xFD[PV[_\x81Q\x90Pa*\xC2\x81a*\x9EV[\x92\x91PPV[_` \x82\x84\x03\x12\x15a*\xDDWa*\xDCa\x1CAV[[_a*\xEA\x84\x82\x85\x01a*\xB4V[\x91PP\x92\x91PPV[_`\x80\x82\x01\x90Pa+\x06_\x83\x01\x87a)\xDAV[a+\x13` \x83\x01\x86a(\xEEV[a+ `@\x83\x01\x85a&KV[a+-``\x83\x01\x84a)\x1EV[\x95\x94PPPPPV[_a+@\x82a%eV[\x91Pa+K\x83a%eV[\x92P\x82\x82\x03\x90Pk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a+oWa+na(\x8EV[[\x92\x91PPV[_\x81`\xE0\x1B\x90P\x91\x90PV[_a+\x8B\x82a+uV[\x90P\x91\x90PV[a+\xA3a+\x9E\x82a \x93V[a+\x81V[\x82RPPV[_\x81Q\x90P\x91\x90PV[_\x81\x90P\x92\x91PPV[_\x81\x90P` \x82\x01\x90P\x91\x90PV[a+\xD5\x81a\x1CIV[\x82RPPV[_a+\xE6\x83\x83a+\xCCV[` \x83\x01\x90P\x92\x91PPV[_` \x82\x01\x90P\x91\x90PV[_a,\x08\x82a+\xA9V[a,\x12\x81\x85a+\xB3V[\x93Pa,\x1D\x83a+\xBDV[\x80_[\x83\x81\x10\x15a,MW\x81Qa,4\x88\x82a+\xDBV[\x97Pa,?\x83a+\xF2V[\x92PP`\x01\x81\x01\x90Pa, V[P\x85\x93PPPP\x92\x91PPV[_a,e\x82\x85a+\x92V[`\x04\x82\x01\x91Pa,u\x82\x84a+\xFEV[\x91P\x81\x90P\x93\x92PPPV[_a,\x8B\x82a\x1D\x0CV[\x91Pa,\x96\x83a\x1D\x0CV[\x92P\x82\x82\x02a,\xA4\x81a\x1D\x0CV[\x91P\x82\x82\x04\x84\x14\x83\x15\x17a,\xBBWa,\xBAa(\x8EV[[P\x92\x91PPV[_a,\xCC\x82a\x1D\x0CV[\x91Pa,\xD7\x83a\x1D\x0CV[\x92P\x82\x82\x01\x90P\x80\x82\x11\x15a,\xEFWa,\xEEa(\x8EV[[\x92\x91PPV[_a\xFF\xFF\x82\x16\x90P\x91\x90PV[_a-\x0C\x82a,\xF5V[\x91Pa\xFF\xFF\x82\x03a- Wa-\x1Fa(\x8EV[[`\x01\x82\x01\x90P\x91\x90PV\xFE\xA2dipfsX\"\x12 \t\xFA\xFDj\x9C\xB7,\x08=\x06[\xE8\xB6th{N\x0F\xC6\x8C\x06\xBA\xF5#N\xAC^.Iu\xCD\xC8dsolcC\0\x08\x1D\x003",
    );
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `BitmapValueTooLarge()` and selector `0xca957333`.
    ```solidity
    error BitmapValueTooLarge();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct BitmapValueTooLarge {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<BitmapValueTooLarge> for UnderlyingRustTuple<'_> {
            fn from(value: BitmapValueTooLarge) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for BitmapValueTooLarge {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for BitmapValueTooLarge {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "BitmapValueTooLarge()";
            const SELECTOR: [u8; 4] = [202u8, 149u8, 115u8, 51u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `BytesArrayLengthTooLong()` and selector `0xfb4a9c8e`.
    ```solidity
    error BytesArrayLengthTooLong();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct BytesArrayLengthTooLong {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<BytesArrayLengthTooLong> for UnderlyingRustTuple<'_> {
            fn from(value: BytesArrayLengthTooLong) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for BytesArrayLengthTooLong {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for BytesArrayLengthTooLong {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "BytesArrayLengthTooLong()";
            const SELECTOR: [u8; 4] = [251u8, 74u8, 156u8, 142u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `BytesArrayNotOrdered()` and selector `0x80c88348`.
    ```solidity
    error BytesArrayNotOrdered();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct BytesArrayNotOrdered {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<BytesArrayNotOrdered> for UnderlyingRustTuple<'_> {
            fn from(value: BytesArrayNotOrdered) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for BytesArrayNotOrdered {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for BytesArrayNotOrdered {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "BytesArrayNotOrdered()";
            const SELECTOR: [u8; 4] = [128u8, 200u8, 131u8, 72u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `ECAddFailed()` and selector `0xd4b68fd7`.
    ```solidity
    error ECAddFailed();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ECAddFailed {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<ECAddFailed> for UnderlyingRustTuple<'_> {
            fn from(value: ECAddFailed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ECAddFailed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ECAddFailed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ECAddFailed()";
            const SELECTOR: [u8; 4] = [212u8, 182u8, 143u8, 215u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `ECMulFailed()` and selector `0x4633be32`.
    ```solidity
    error ECMulFailed();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ECMulFailed {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<ECMulFailed> for UnderlyingRustTuple<'_> {
            fn from(value: ECMulFailed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ECMulFailed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ECMulFailed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ECMulFailed()";
            const SELECTOR: [u8; 4] = [70u8, 51u8, 190u8, 50u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `ExpModFailed()` and selector `0xd51edae3`.
    ```solidity
    error ExpModFailed();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ExpModFailed {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<ExpModFailed> for UnderlyingRustTuple<'_> {
            fn from(value: ExpModFailed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ExpModFailed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ExpModFailed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ExpModFailed()";
            const SELECTOR: [u8; 4] = [213u8, 30u8, 218u8, 227u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `InputArrayLengthMismatch()` and selector `0x43714afd`.
    ```solidity
    error InputArrayLengthMismatch();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InputArrayLengthMismatch {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<InputArrayLengthMismatch> for UnderlyingRustTuple<'_> {
            fn from(value: InputArrayLengthMismatch) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InputArrayLengthMismatch {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InputArrayLengthMismatch {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InputArrayLengthMismatch()";
            const SELECTOR: [u8; 4] = [67u8, 113u8, 74u8, 253u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `InputEmptyQuorumNumbers()` and selector `0x1f0405a0`.
    ```solidity
    error InputEmptyQuorumNumbers();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InputEmptyQuorumNumbers {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<InputEmptyQuorumNumbers> for UnderlyingRustTuple<'_> {
            fn from(value: InputEmptyQuorumNumbers) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InputEmptyQuorumNumbers {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InputEmptyQuorumNumbers {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InputEmptyQuorumNumbers()";
            const SELECTOR: [u8; 4] = [31u8, 4u8, 5u8, 160u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `InputNonSignerLengthMismatch()` and selector `0x5f832f41`.
    ```solidity
    error InputNonSignerLengthMismatch();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InputNonSignerLengthMismatch {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<InputNonSignerLengthMismatch> for UnderlyingRustTuple<'_> {
            fn from(value: InputNonSignerLengthMismatch) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InputNonSignerLengthMismatch {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InputNonSignerLengthMismatch {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InputNonSignerLengthMismatch()";
            const SELECTOR: [u8; 4] = [95u8, 131u8, 47u8, 65u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `InvalidBLSPairingKey()` and selector `0x67988d33`.
    ```solidity
    error InvalidBLSPairingKey();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidBLSPairingKey {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<InvalidBLSPairingKey> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidBLSPairingKey) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidBLSPairingKey {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidBLSPairingKey {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidBLSPairingKey()";
            const SELECTOR: [u8; 4] = [103u8, 152u8, 141u8, 51u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `InvalidBLSSignature()` and selector `0xab1b236b`.
    ```solidity
    error InvalidBLSSignature();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidBLSSignature {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<InvalidBLSSignature> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidBLSSignature) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidBLSSignature {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidBLSSignature {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidBLSSignature()";
            const SELECTOR: [u8; 4] = [171u8, 27u8, 35u8, 107u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `InvalidQuorumApkHash()` and selector `0xe1310aed`.
    ```solidity
    error InvalidQuorumApkHash();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidQuorumApkHash {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<InvalidQuorumApkHash> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidQuorumApkHash) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidQuorumApkHash {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidQuorumApkHash {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidQuorumApkHash()";
            const SELECTOR: [u8; 4] = [225u8, 49u8, 10u8, 237u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `InvalidReferenceBlocknumber()` and selector `0x4b874f45`.
    ```solidity
    error InvalidReferenceBlocknumber();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidReferenceBlocknumber {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<InvalidReferenceBlocknumber> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidReferenceBlocknumber) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidReferenceBlocknumber {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidReferenceBlocknumber {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidReferenceBlocknumber()";
            const SELECTOR: [u8; 4] = [75u8, 135u8, 79u8, 69u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `NonSignerPubkeysNotSorted()` and selector `0xff719414`.
    ```solidity
    error NonSignerPubkeysNotSorted();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct NonSignerPubkeysNotSorted {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<NonSignerPubkeysNotSorted> for UnderlyingRustTuple<'_> {
            fn from(value: NonSignerPubkeysNotSorted) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for NonSignerPubkeysNotSorted {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for NonSignerPubkeysNotSorted {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "NonSignerPubkeysNotSorted()";
            const SELECTOR: [u8; 4] = [255u8, 113u8, 148u8, 20u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `OnlyRegistryCoordinatorOwner()` and selector `0xe0e1e762`.
    ```solidity
    error OnlyRegistryCoordinatorOwner();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct OnlyRegistryCoordinatorOwner {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<OnlyRegistryCoordinatorOwner> for UnderlyingRustTuple<'_> {
            fn from(value: OnlyRegistryCoordinatorOwner) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for OnlyRegistryCoordinatorOwner {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for OnlyRegistryCoordinatorOwner {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "OnlyRegistryCoordinatorOwner()";
            const SELECTOR: [u8; 4] = [224u8, 225u8, 231u8, 98u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `ScalarTooLarge()` and selector `0xff89d4fa`.
    ```solidity
    error ScalarTooLarge();
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ScalarTooLarge {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<ScalarTooLarge> for UnderlyingRustTuple<'_> {
            fn from(value: ScalarTooLarge) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ScalarTooLarge {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {}
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ScalarTooLarge {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ScalarTooLarge()";
            const SELECTOR: [u8; 4] = [255u8, 137u8, 212u8, 250u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    /**Constructor`.
    ```solidity
    constructor(address _registryCoordinator);
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct constructorCall {
        #[allow(missing_docs)]
        pub _registryCoordinator: alloy::sol_types::private::Address,
    }
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<constructorCall> for UnderlyingRustTuple<'_> {
                fn from(value: constructorCall) -> Self {
                    (value._registryCoordinator,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for constructorCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        _registryCoordinator: tuple.0,
                    }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolConstructor for constructorCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self._registryCoordinator,
                    ),
                )
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `blsApkRegistry()` and selector `0x5df45946`.
    ```solidity
    function blsApkRegistry() external view returns (address);
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct blsApkRegistryCall {}
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`blsApkRegistry()`](blsApkRegistryCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct blsApkRegistryReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<blsApkRegistryCall> for UnderlyingRustTuple<'_> {
                fn from(value: blsApkRegistryCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for blsApkRegistryCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<blsApkRegistryReturn> for UnderlyingRustTuple<'_> {
                fn from(value: blsApkRegistryReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for blsApkRegistryReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for blsApkRegistryCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            type Return = blsApkRegistryReturn;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Address,);
            type ReturnToken<'a> = <Self::ReturnTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "blsApkRegistry()";
            const SELECTOR: [u8; 4] = [93u8, 244u8, 89u8, 70u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_returns(
                data: &[u8],
                validate: bool,
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<'_> as alloy_sol_types::SolType>::abi_decode_sequence(
                    data, validate,
                )
                .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `checkSignatures(bytes32,bytes,uint32,(uint32[],(uint256,uint256)[],(uint256,uint256)[],(uint256[2],uint256[2]),(uint256,uint256),uint32[],uint32[],uint32[][]))` and selector `0x6efb4636`.
    ```solidity
    function checkSignatures(bytes32 msgHash, bytes memory quorumNumbers, uint32 referenceBlockNumber, IBLSSignatureCheckerTypes.NonSignerStakesAndSignature memory params) external view returns (IBLSSignatureCheckerTypes.QuorumStakeTotals memory, bytes32);
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct checkSignaturesCall {
        #[allow(missing_docs)]
        pub msgHash: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub quorumNumbers: alloy::sol_types::private::Bytes,
        #[allow(missing_docs)]
        pub referenceBlockNumber: u32,
        #[allow(missing_docs)]
        pub params: <IBLSSignatureCheckerTypes::NonSignerStakesAndSignature as alloy::sol_types::SolType>::RustType,
    }
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`checkSignatures(bytes32,bytes,uint32,(uint32[],(uint256,uint256)[],(uint256,uint256)[],(uint256[2],uint256[2]),(uint256,uint256),uint32[],uint32[],uint32[][]))`](checkSignaturesCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct checkSignaturesReturn {
        #[allow(missing_docs)]
        pub _0:
            <IBLSSignatureCheckerTypes::QuorumStakeTotals as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub _1: alloy::sol_types::private::FixedBytes<32>,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Bytes,
                alloy::sol_types::sol_data::Uint<32>,
                IBLSSignatureCheckerTypes::NonSignerStakesAndSignature,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::FixedBytes<32>,
                alloy::sol_types::private::Bytes,
                u32,
                <IBLSSignatureCheckerTypes::NonSignerStakesAndSignature as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<checkSignaturesCall> for UnderlyingRustTuple<'_> {
                fn from(value: checkSignaturesCall) -> Self {
                    (
                        value.msgHash,
                        value.quorumNumbers,
                        value.referenceBlockNumber,
                        value.params,
                    )
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for checkSignaturesCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        msgHash: tuple.0,
                        quorumNumbers: tuple.1,
                        referenceBlockNumber: tuple.2,
                        params: tuple.3,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                IBLSSignatureCheckerTypes::QuorumStakeTotals,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <IBLSSignatureCheckerTypes::QuorumStakeTotals as alloy::sol_types::SolType>::RustType,
                alloy::sol_types::private::FixedBytes<32>,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<checkSignaturesReturn> for UnderlyingRustTuple<'_> {
                fn from(value: checkSignaturesReturn) -> Self {
                    (value._0, value._1)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for checkSignaturesReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        _0: tuple.0,
                        _1: tuple.1,
                    }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for checkSignaturesCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Bytes,
                alloy::sol_types::sol_data::Uint<32>,
                IBLSSignatureCheckerTypes::NonSignerStakesAndSignature,
            );
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            type Return = checkSignaturesReturn;
            type ReturnTuple<'a> = (
                IBLSSignatureCheckerTypes::QuorumStakeTotals,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            type ReturnToken<'a> = <Self::ReturnTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "checkSignatures(bytes32,bytes,uint32,(uint32[],(uint256,uint256)[],(uint256,uint256)[],(uint256[2],uint256[2]),(uint256,uint256),uint32[],uint32[],uint32[][]))";
            const SELECTOR: [u8; 4] = [110u8, 251u8, 70u8, 54u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.msgHash),
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self.quorumNumbers,
                    ),
                    <alloy::sol_types::sol_data::Uint<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.referenceBlockNumber),
                    <IBLSSignatureCheckerTypes::NonSignerStakesAndSignature as alloy_sol_types::SolType>::tokenize(
                        &self.params,
                    ),
                )
            }
            #[inline]
            fn abi_decode_returns(
                data: &[u8],
                validate: bool,
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<'_> as alloy_sol_types::SolType>::abi_decode_sequence(
                    data, validate,
                )
                .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `delegation()` and selector `0xdf5cf723`.
    ```solidity
    function delegation() external view returns (address);
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct delegationCall {}
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`delegation()`](delegationCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct delegationReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<delegationCall> for UnderlyingRustTuple<'_> {
                fn from(value: delegationCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for delegationCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<delegationReturn> for UnderlyingRustTuple<'_> {
                fn from(value: delegationReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for delegationReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for delegationCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            type Return = delegationReturn;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Address,);
            type ReturnToken<'a> = <Self::ReturnTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "delegation()";
            const SELECTOR: [u8; 4] = [223u8, 92u8, 247u8, 35u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_returns(
                data: &[u8],
                validate: bool,
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<'_> as alloy_sol_types::SolType>::abi_decode_sequence(
                    data, validate,
                )
                .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `registryCoordinator()` and selector `0x6d14a987`.
    ```solidity
    function registryCoordinator() external view returns (address);
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct registryCoordinatorCall {}
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`registryCoordinator()`](registryCoordinatorCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct registryCoordinatorReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<registryCoordinatorCall> for UnderlyingRustTuple<'_> {
                fn from(value: registryCoordinatorCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for registryCoordinatorCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<registryCoordinatorReturn> for UnderlyingRustTuple<'_> {
                fn from(value: registryCoordinatorReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for registryCoordinatorReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for registryCoordinatorCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            type Return = registryCoordinatorReturn;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Address,);
            type ReturnToken<'a> = <Self::ReturnTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "registryCoordinator()";
            const SELECTOR: [u8; 4] = [109u8, 20u8, 169u8, 135u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_returns(
                data: &[u8],
                validate: bool,
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<'_> as alloy_sol_types::SolType>::abi_decode_sequence(
                    data, validate,
                )
                .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `stakeRegistry()` and selector `0x68304835`.
    ```solidity
    function stakeRegistry() external view returns (address);
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct stakeRegistryCall {}
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`stakeRegistry()`](stakeRegistryCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct stakeRegistryReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<stakeRegistryCall> for UnderlyingRustTuple<'_> {
                fn from(value: stakeRegistryCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for stakeRegistryCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<stakeRegistryReturn> for UnderlyingRustTuple<'_> {
                fn from(value: stakeRegistryReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for stakeRegistryReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for stakeRegistryCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            type Return = stakeRegistryReturn;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Address,);
            type ReturnToken<'a> = <Self::ReturnTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "stakeRegistry()";
            const SELECTOR: [u8; 4] = [104u8, 48u8, 72u8, 53u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_returns(
                data: &[u8],
                validate: bool,
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<'_> as alloy_sol_types::SolType>::abi_decode_sequence(
                    data, validate,
                )
                .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `trySignatureAndApkVerification(bytes32,(uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256))` and selector `0x171f1d5b`.
    ```solidity
    function trySignatureAndApkVerification(bytes32 msgHash, BN254.G1Point memory apk, BN254.G2Point memory apkG2, BN254.G1Point memory sigma) external view returns (bool pairingSuccessful, bool siganatureIsValid);
    ```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct trySignatureAndApkVerificationCall {
        #[allow(missing_docs)]
        pub msgHash: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub apk: <BN254::G1Point as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub apkG2: <BN254::G2Point as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub sigma: <BN254::G1Point as alloy::sol_types::SolType>::RustType,
    }
    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`trySignatureAndApkVerification(bytes32,(uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256))`](trySignatureAndApkVerificationCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct trySignatureAndApkVerificationReturn {
        #[allow(missing_docs)]
        pub pairingSuccessful: bool,
        #[allow(missing_docs)]
        pub siganatureIsValid: bool,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                BN254::G1Point,
                BN254::G2Point,
                BN254::G1Point,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::FixedBytes<32>,
                <BN254::G1Point as alloy::sol_types::SolType>::RustType,
                <BN254::G2Point as alloy::sol_types::SolType>::RustType,
                <BN254::G1Point as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<trySignatureAndApkVerificationCall> for UnderlyingRustTuple<'_> {
                fn from(value: trySignatureAndApkVerificationCall) -> Self {
                    (value.msgHash, value.apk, value.apkG2, value.sigma)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for trySignatureAndApkVerificationCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        msgHash: tuple.0,
                        apk: tuple.1,
                        apkG2: tuple.2,
                        sigma: tuple.3,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Bool,
                alloy::sol_types::sol_data::Bool,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (bool, bool);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(_t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<trySignatureAndApkVerificationReturn> for UnderlyingRustTuple<'_> {
                fn from(value: trySignatureAndApkVerificationReturn) -> Self {
                    (value.pairingSuccessful, value.siganatureIsValid)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for trySignatureAndApkVerificationReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        pairingSuccessful: tuple.0,
                        siganatureIsValid: tuple.1,
                    }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for trySignatureAndApkVerificationCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                BN254::G1Point,
                BN254::G2Point,
                BN254::G1Point,
            );
            type Token<'a> = <Self::Parameters<'a> as alloy_sol_types::SolType>::Token<'a>;
            type Return = trySignatureAndApkVerificationReturn;
            type ReturnTuple<'a> = (
                alloy::sol_types::sol_data::Bool,
                alloy::sol_types::sol_data::Bool,
            );
            type ReturnToken<'a> = <Self::ReturnTuple<'a> as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "trySignatureAndApkVerification(bytes32,(uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256))";
            const SELECTOR: [u8; 4] = [23u8, 31u8, 29u8, 91u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.msgHash),
                    <BN254::G1Point as alloy_sol_types::SolType>::tokenize(&self.apk),
                    <BN254::G2Point as alloy_sol_types::SolType>::tokenize(&self.apkG2),
                    <BN254::G1Point as alloy_sol_types::SolType>::tokenize(&self.sigma),
                )
            }
            #[inline]
            fn abi_decode_returns(
                data: &[u8],
                validate: bool,
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<'_> as alloy_sol_types::SolType>::abi_decode_sequence(
                    data, validate,
                )
                .map(Into::into)
            }
        }
    };
    ///Container for all the [`BLSSignatureChecker`](self) function calls.
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum BLSSignatureCheckerCalls {
        #[allow(missing_docs)]
        blsApkRegistry(blsApkRegistryCall),
        #[allow(missing_docs)]
        checkSignatures(checkSignaturesCall),
        #[allow(missing_docs)]
        delegation(delegationCall),
        #[allow(missing_docs)]
        registryCoordinator(registryCoordinatorCall),
        #[allow(missing_docs)]
        stakeRegistry(stakeRegistryCall),
        #[allow(missing_docs)]
        trySignatureAndApkVerification(trySignatureAndApkVerificationCall),
    }

    impl BLSSignatureCheckerCalls {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 4usize]] = &[
            [23u8, 31u8, 29u8, 91u8],
            [93u8, 244u8, 89u8, 70u8],
            [104u8, 48u8, 72u8, 53u8],
            [109u8, 20u8, 169u8, 135u8],
            [110u8, 251u8, 70u8, 54u8],
            [223u8, 92u8, 247u8, 35u8],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolInterface for BLSSignatureCheckerCalls {
        const NAME: &'static str = "BLSSignatureCheckerCalls";
        const MIN_DATA_LENGTH: usize = 0usize;
        const COUNT: usize = 6usize;
        #[inline]
        fn selector(&self) -> [u8; 4] {
            match self {
                Self::blsApkRegistry(_) => {
                    <blsApkRegistryCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::checkSignatures(_) => {
                    <checkSignaturesCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::delegation(_) => <delegationCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::registryCoordinator(_) => {
                    <registryCoordinatorCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::stakeRegistry(_) => <stakeRegistryCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::trySignatureAndApkVerification(_) => {
                    <trySignatureAndApkVerificationCall as alloy_sol_types::SolCall>::SELECTOR
                }
            }
        }
        #[inline]
        fn selector_at(i: usize) -> ::core::option::Option<[u8; 4]> {
            Self::SELECTORS.get(i).copied()
        }
        #[inline]
        fn valid_selector(selector: [u8; 4]) -> bool {
            Self::SELECTORS.binary_search(&selector).is_ok()
        }
        #[inline]
        #[allow(non_snake_case)]
        fn abi_decode_raw(
            selector: [u8; 4],
            data: &[u8],
            validate: bool,
        ) -> alloy_sol_types::Result<Self> {
            static DECODE_SHIMS: &[fn(
                &[u8],
                bool,
            )
                -> alloy_sol_types::Result<BLSSignatureCheckerCalls>] = &[
                {
                    fn trySignatureAndApkVerification(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerCalls> {
                        <trySignatureAndApkVerificationCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                                validate,
                            )
                            .map(
                                BLSSignatureCheckerCalls::trySignatureAndApkVerification,
                            )
                    }
                    trySignatureAndApkVerification
                },
                {
                    fn blsApkRegistry(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerCalls> {
                        <blsApkRegistryCall as alloy_sol_types::SolCall>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerCalls::blsApkRegistry)
                    }
                    blsApkRegistry
                },
                {
                    fn stakeRegistry(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerCalls> {
                        <stakeRegistryCall as alloy_sol_types::SolCall>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerCalls::stakeRegistry)
                    }
                    stakeRegistry
                },
                {
                    fn registryCoordinator(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerCalls> {
                        <registryCoordinatorCall as alloy_sol_types::SolCall>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerCalls::registryCoordinator)
                    }
                    registryCoordinator
                },
                {
                    fn checkSignatures(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerCalls> {
                        <checkSignaturesCall as alloy_sol_types::SolCall>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerCalls::checkSignatures)
                    }
                    checkSignatures
                },
                {
                    fn delegation(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerCalls> {
                        <delegationCall as alloy_sol_types::SolCall>::abi_decode_raw(data, validate)
                            .map(BLSSignatureCheckerCalls::delegation)
                    }
                    delegation
                },
            ];
            let Ok(idx) = Self::SELECTORS.binary_search(&selector) else {
                return Err(alloy_sol_types::Error::unknown_selector(
                    <Self as alloy_sol_types::SolInterface>::NAME,
                    selector,
                ));
            };
            DECODE_SHIMS[idx](data, validate)
        }
        #[inline]
        fn abi_encoded_size(&self) -> usize {
            match self {
                Self::blsApkRegistry(inner) => {
                    <blsApkRegistryCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::checkSignatures(inner) => {
                    <checkSignaturesCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::delegation(inner) => {
                    <delegationCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::registryCoordinator(inner) => {
                    <registryCoordinatorCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::stakeRegistry(inner) => {
                    <stakeRegistryCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::trySignatureAndApkVerification(inner) => {
                    <trySignatureAndApkVerificationCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
            }
        }
        #[inline]
        fn abi_encode_raw(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
            match self {
                Self::blsApkRegistry(inner) => {
                    <blsApkRegistryCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::checkSignatures(inner) => {
                    <checkSignaturesCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::delegation(inner) => {
                    <delegationCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::registryCoordinator(inner) => {
                    <registryCoordinatorCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::stakeRegistry(inner) => {
                    <stakeRegistryCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::trySignatureAndApkVerification(inner) => {
                    <trySignatureAndApkVerificationCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner, out,
                    )
                }
            }
        }
    }
    ///Container for all the [`BLSSignatureChecker`](self) custom errors.
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Hash)]
    pub enum BLSSignatureCheckerErrors {
        #[allow(missing_docs)]
        BitmapValueTooLarge(BitmapValueTooLarge),
        #[allow(missing_docs)]
        BytesArrayLengthTooLong(BytesArrayLengthTooLong),
        #[allow(missing_docs)]
        BytesArrayNotOrdered(BytesArrayNotOrdered),
        #[allow(missing_docs)]
        ECAddFailed(ECAddFailed),
        #[allow(missing_docs)]
        ECMulFailed(ECMulFailed),
        #[allow(missing_docs)]
        ExpModFailed(ExpModFailed),
        #[allow(missing_docs)]
        InputArrayLengthMismatch(InputArrayLengthMismatch),
        #[allow(missing_docs)]
        InputEmptyQuorumNumbers(InputEmptyQuorumNumbers),
        #[allow(missing_docs)]
        InputNonSignerLengthMismatch(InputNonSignerLengthMismatch),
        #[allow(missing_docs)]
        InvalidBLSPairingKey(InvalidBLSPairingKey),
        #[allow(missing_docs)]
        InvalidBLSSignature(InvalidBLSSignature),
        #[allow(missing_docs)]
        InvalidQuorumApkHash(InvalidQuorumApkHash),
        #[allow(missing_docs)]
        InvalidReferenceBlocknumber(InvalidReferenceBlocknumber),
        #[allow(missing_docs)]
        NonSignerPubkeysNotSorted(NonSignerPubkeysNotSorted),
        #[allow(missing_docs)]
        OnlyRegistryCoordinatorOwner(OnlyRegistryCoordinatorOwner),
        #[allow(missing_docs)]
        ScalarTooLarge(ScalarTooLarge),
    }

    impl BLSSignatureCheckerErrors {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 4usize]] = &[
            [31u8, 4u8, 5u8, 160u8],
            [67u8, 113u8, 74u8, 253u8],
            [70u8, 51u8, 190u8, 50u8],
            [75u8, 135u8, 79u8, 69u8],
            [95u8, 131u8, 47u8, 65u8],
            [103u8, 152u8, 141u8, 51u8],
            [128u8, 200u8, 131u8, 72u8],
            [171u8, 27u8, 35u8, 107u8],
            [202u8, 149u8, 115u8, 51u8],
            [212u8, 182u8, 143u8, 215u8],
            [213u8, 30u8, 218u8, 227u8],
            [224u8, 225u8, 231u8, 98u8],
            [225u8, 49u8, 10u8, 237u8],
            [251u8, 74u8, 156u8, 142u8],
            [255u8, 113u8, 148u8, 20u8],
            [255u8, 137u8, 212u8, 250u8],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolInterface for BLSSignatureCheckerErrors {
        const NAME: &'static str = "BLSSignatureCheckerErrors";
        const MIN_DATA_LENGTH: usize = 0usize;
        const COUNT: usize = 16usize;
        #[inline]
        fn selector(&self) -> [u8; 4] {
            match self {
                Self::BitmapValueTooLarge(_) => {
                    <BitmapValueTooLarge as alloy_sol_types::SolError>::SELECTOR
                }
                Self::BytesArrayLengthTooLong(_) => {
                    <BytesArrayLengthTooLong as alloy_sol_types::SolError>::SELECTOR
                }
                Self::BytesArrayNotOrdered(_) => {
                    <BytesArrayNotOrdered as alloy_sol_types::SolError>::SELECTOR
                }
                Self::ECAddFailed(_) => <ECAddFailed as alloy_sol_types::SolError>::SELECTOR,
                Self::ECMulFailed(_) => <ECMulFailed as alloy_sol_types::SolError>::SELECTOR,
                Self::ExpModFailed(_) => <ExpModFailed as alloy_sol_types::SolError>::SELECTOR,
                Self::InputArrayLengthMismatch(_) => {
                    <InputArrayLengthMismatch as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InputEmptyQuorumNumbers(_) => {
                    <InputEmptyQuorumNumbers as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InputNonSignerLengthMismatch(_) => {
                    <InputNonSignerLengthMismatch as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidBLSPairingKey(_) => {
                    <InvalidBLSPairingKey as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidBLSSignature(_) => {
                    <InvalidBLSSignature as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidQuorumApkHash(_) => {
                    <InvalidQuorumApkHash as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidReferenceBlocknumber(_) => {
                    <InvalidReferenceBlocknumber as alloy_sol_types::SolError>::SELECTOR
                }
                Self::NonSignerPubkeysNotSorted(_) => {
                    <NonSignerPubkeysNotSorted as alloy_sol_types::SolError>::SELECTOR
                }
                Self::OnlyRegistryCoordinatorOwner(_) => {
                    <OnlyRegistryCoordinatorOwner as alloy_sol_types::SolError>::SELECTOR
                }
                Self::ScalarTooLarge(_) => <ScalarTooLarge as alloy_sol_types::SolError>::SELECTOR,
            }
        }
        #[inline]
        fn selector_at(i: usize) -> ::core::option::Option<[u8; 4]> {
            Self::SELECTORS.get(i).copied()
        }
        #[inline]
        fn valid_selector(selector: [u8; 4]) -> bool {
            Self::SELECTORS.binary_search(&selector).is_ok()
        }
        #[inline]
        #[allow(non_snake_case)]
        fn abi_decode_raw(
            selector: [u8; 4],
            data: &[u8],
            validate: bool,
        ) -> alloy_sol_types::Result<Self> {
            static DECODE_SHIMS: &[fn(
                &[u8],
                bool,
            )
                -> alloy_sol_types::Result<BLSSignatureCheckerErrors>] = &[
                {
                    fn InputEmptyQuorumNumbers(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <InputEmptyQuorumNumbers as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::InputEmptyQuorumNumbers)
                    }
                    InputEmptyQuorumNumbers
                },
                {
                    fn InputArrayLengthMismatch(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <InputArrayLengthMismatch as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::InputArrayLengthMismatch)
                    }
                    InputArrayLengthMismatch
                },
                {
                    fn ECMulFailed(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <ECMulFailed as alloy_sol_types::SolError>::abi_decode_raw(data, validate)
                            .map(BLSSignatureCheckerErrors::ECMulFailed)
                    }
                    ECMulFailed
                },
                {
                    fn InvalidReferenceBlocknumber(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <InvalidReferenceBlocknumber as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::InvalidReferenceBlocknumber)
                    }
                    InvalidReferenceBlocknumber
                },
                {
                    fn InputNonSignerLengthMismatch(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <InputNonSignerLengthMismatch as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::InputNonSignerLengthMismatch)
                    }
                    InputNonSignerLengthMismatch
                },
                {
                    fn InvalidBLSPairingKey(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <InvalidBLSPairingKey as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::InvalidBLSPairingKey)
                    }
                    InvalidBLSPairingKey
                },
                {
                    fn BytesArrayNotOrdered(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <BytesArrayNotOrdered as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::BytesArrayNotOrdered)
                    }
                    BytesArrayNotOrdered
                },
                {
                    fn InvalidBLSSignature(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <InvalidBLSSignature as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::InvalidBLSSignature)
                    }
                    InvalidBLSSignature
                },
                {
                    fn BitmapValueTooLarge(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <BitmapValueTooLarge as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::BitmapValueTooLarge)
                    }
                    BitmapValueTooLarge
                },
                {
                    fn ECAddFailed(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <ECAddFailed as alloy_sol_types::SolError>::abi_decode_raw(data, validate)
                            .map(BLSSignatureCheckerErrors::ECAddFailed)
                    }
                    ECAddFailed
                },
                {
                    fn ExpModFailed(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <ExpModFailed as alloy_sol_types::SolError>::abi_decode_raw(data, validate)
                            .map(BLSSignatureCheckerErrors::ExpModFailed)
                    }
                    ExpModFailed
                },
                {
                    fn OnlyRegistryCoordinatorOwner(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <OnlyRegistryCoordinatorOwner as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::OnlyRegistryCoordinatorOwner)
                    }
                    OnlyRegistryCoordinatorOwner
                },
                {
                    fn InvalidQuorumApkHash(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <InvalidQuorumApkHash as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::InvalidQuorumApkHash)
                    }
                    InvalidQuorumApkHash
                },
                {
                    fn BytesArrayLengthTooLong(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <BytesArrayLengthTooLong as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::BytesArrayLengthTooLong)
                    }
                    BytesArrayLengthTooLong
                },
                {
                    fn NonSignerPubkeysNotSorted(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <NonSignerPubkeysNotSorted as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::NonSignerPubkeysNotSorted)
                    }
                    NonSignerPubkeysNotSorted
                },
                {
                    fn ScalarTooLarge(
                        data: &[u8],
                        validate: bool,
                    ) -> alloy_sol_types::Result<BLSSignatureCheckerErrors> {
                        <ScalarTooLarge as alloy_sol_types::SolError>::abi_decode_raw(
                            data, validate,
                        )
                        .map(BLSSignatureCheckerErrors::ScalarTooLarge)
                    }
                    ScalarTooLarge
                },
            ];
            let Ok(idx) = Self::SELECTORS.binary_search(&selector) else {
                return Err(alloy_sol_types::Error::unknown_selector(
                    <Self as alloy_sol_types::SolInterface>::NAME,
                    selector,
                ));
            };
            DECODE_SHIMS[idx](data, validate)
        }
        #[inline]
        fn abi_encoded_size(&self) -> usize {
            match self {
                Self::BitmapValueTooLarge(inner) => {
                    <BitmapValueTooLarge as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::BytesArrayLengthTooLong(inner) => {
                    <BytesArrayLengthTooLong as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::BytesArrayNotOrdered(inner) => {
                    <BytesArrayNotOrdered as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::ECAddFailed(inner) => {
                    <ECAddFailed as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::ECMulFailed(inner) => {
                    <ECMulFailed as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::ExpModFailed(inner) => {
                    <ExpModFailed as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::InputArrayLengthMismatch(inner) => {
                    <InputArrayLengthMismatch as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::InputEmptyQuorumNumbers(inner) => {
                    <InputEmptyQuorumNumbers as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::InputNonSignerLengthMismatch(inner) => {
                    <InputNonSignerLengthMismatch as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidBLSPairingKey(inner) => {
                    <InvalidBLSPairingKey as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::InvalidBLSSignature(inner) => {
                    <InvalidBLSSignature as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::InvalidQuorumApkHash(inner) => {
                    <InvalidQuorumApkHash as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::InvalidReferenceBlocknumber(inner) => {
                    <InvalidReferenceBlocknumber as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::NonSignerPubkeysNotSorted(inner) => {
                    <NonSignerPubkeysNotSorted as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::OnlyRegistryCoordinatorOwner(inner) => {
                    <OnlyRegistryCoordinatorOwner as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::ScalarTooLarge(inner) => {
                    <ScalarTooLarge as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
            }
        }
        #[inline]
        fn abi_encode_raw(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
            match self {
                Self::BitmapValueTooLarge(inner) => {
                    <BitmapValueTooLarge as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::BytesArrayLengthTooLong(inner) => {
                    <BytesArrayLengthTooLong as alloy_sol_types::SolError>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::BytesArrayNotOrdered(inner) => {
                    <BytesArrayNotOrdered as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::ECAddFailed(inner) => {
                    <ECAddFailed as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::ECMulFailed(inner) => {
                    <ECMulFailed as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::ExpModFailed(inner) => {
                    <ExpModFailed as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::InputArrayLengthMismatch(inner) => {
                    <InputArrayLengthMismatch as alloy_sol_types::SolError>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::InputEmptyQuorumNumbers(inner) => {
                    <InputEmptyQuorumNumbers as alloy_sol_types::SolError>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::InputNonSignerLengthMismatch(inner) => {
                    <InputNonSignerLengthMismatch as alloy_sol_types::SolError>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::InvalidBLSPairingKey(inner) => {
                    <InvalidBLSPairingKey as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::InvalidBLSSignature(inner) => {
                    <InvalidBLSSignature as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::InvalidQuorumApkHash(inner) => {
                    <InvalidQuorumApkHash as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::InvalidReferenceBlocknumber(inner) => {
                    <InvalidReferenceBlocknumber as alloy_sol_types::SolError>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::NonSignerPubkeysNotSorted(inner) => {
                    <NonSignerPubkeysNotSorted as alloy_sol_types::SolError>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::OnlyRegistryCoordinatorOwner(inner) => {
                    <OnlyRegistryCoordinatorOwner as alloy_sol_types::SolError>::abi_encode_raw(
                        inner, out,
                    )
                }
                Self::ScalarTooLarge(inner) => {
                    <ScalarTooLarge as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
            }
        }
    }
    use alloy::contract as alloy_contract;
    /**Creates a new wrapper around an on-chain [`BLSSignatureChecker`](self) contract instance.

    See the [wrapper's documentation](`BLSSignatureCheckerInstance`) for more details.*/
    #[inline]
    pub const fn new<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    >(
        address: alloy_sol_types::private::Address,
        provider: P,
    ) -> BLSSignatureCheckerInstance<T, P, N> {
        BLSSignatureCheckerInstance::<T, P, N>::new(address, provider)
    }
    /**Deploys this contract using the given `provider` and constructor arguments, if any.

    Returns a new instance of the contract, if the deployment was successful.

    For more fine-grained control over the deployment process, use [`deploy_builder`] instead.*/
    #[inline]
    pub fn deploy<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    >(
        provider: P,
        _registryCoordinator: alloy::sol_types::private::Address,
    ) -> impl ::core::future::Future<Output = alloy_contract::Result<BLSSignatureCheckerInstance<T, P, N>>>
    {
        BLSSignatureCheckerInstance::<T, P, N>::deploy(provider, _registryCoordinator)
    }
    /**Creates a `RawCallBuilder` for deploying this contract using the given `provider`
    and constructor arguments, if any.

    This is a simple wrapper around creating a `RawCallBuilder` with the data set to
    the bytecode concatenated with the constructor's ABI-encoded arguments.*/
    #[inline]
    pub fn deploy_builder<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    >(
        provider: P,
        _registryCoordinator: alloy::sol_types::private::Address,
    ) -> alloy_contract::RawCallBuilder<T, P, N> {
        BLSSignatureCheckerInstance::<T, P, N>::deploy_builder(provider, _registryCoordinator)
    }
    /**A [`BLSSignatureChecker`](self) instance.

    Contains type-safe methods for interacting with an on-chain instance of the
    [`BLSSignatureChecker`](self) contract located at a given `address`, using a given
    provider `P`.

    If the contract bytecode is available (see the [`sol!`](alloy_sol_types::sol!)
    documentation on how to provide it), the `deploy` and `deploy_builder` methods can
    be used to deploy a new instance of the contract.

    See the [module-level documentation](self) for all the available methods.*/
    #[derive(Clone)]
    pub struct BLSSignatureCheckerInstance<T, P, N = alloy_contract::private::Ethereum> {
        address: alloy_sol_types::private::Address,
        provider: P,
        _network_transport: ::core::marker::PhantomData<(N, T)>,
    }
    #[automatically_derived]
    impl<T, P, N> ::core::fmt::Debug for BLSSignatureCheckerInstance<T, P, N> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple("BLSSignatureCheckerInstance")
                .field(&self.address)
                .finish()
        }
    }
    /// Instantiation and getters/setters.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > BLSSignatureCheckerInstance<T, P, N>
    {
        /**Creates a new wrapper around an on-chain [`BLSSignatureChecker`](self) contract instance.

        See the [wrapper's documentation](`BLSSignatureCheckerInstance`) for more details.*/
        #[inline]
        pub const fn new(address: alloy_sol_types::private::Address, provider: P) -> Self {
            Self {
                address,
                provider,
                _network_transport: ::core::marker::PhantomData,
            }
        }
        /**Deploys this contract using the given `provider` and constructor arguments, if any.

        Returns a new instance of the contract, if the deployment was successful.

        For more fine-grained control over the deployment process, use [`deploy_builder`] instead.*/
        #[inline]
        pub async fn deploy(
            provider: P,
            _registryCoordinator: alloy::sol_types::private::Address,
        ) -> alloy_contract::Result<BLSSignatureCheckerInstance<T, P, N>> {
            let call_builder = Self::deploy_builder(provider, _registryCoordinator);
            let contract_address = call_builder.deploy().await?;
            Ok(Self::new(contract_address, call_builder.provider))
        }
        /**Creates a `RawCallBuilder` for deploying this contract using the given `provider`
        and constructor arguments, if any.

        This is a simple wrapper around creating a `RawCallBuilder` with the data set to
        the bytecode concatenated with the constructor's ABI-encoded arguments.*/
        #[inline]
        pub fn deploy_builder(
            provider: P,
            _registryCoordinator: alloy::sol_types::private::Address,
        ) -> alloy_contract::RawCallBuilder<T, P, N> {
            alloy_contract::RawCallBuilder::new_raw_deploy(
                provider,
                [
                    &BYTECODE[..],
                    &alloy_sol_types::SolConstructor::abi_encode(&constructorCall {
                        _registryCoordinator,
                    })[..],
                ]
                .concat()
                .into(),
            )
        }
        /// Returns a reference to the address.
        #[inline]
        pub const fn address(&self) -> &alloy_sol_types::private::Address {
            &self.address
        }
        /// Sets the address.
        #[inline]
        pub fn set_address(&mut self, address: alloy_sol_types::private::Address) {
            self.address = address;
        }
        /// Sets the address and returns `self`.
        pub fn at(mut self, address: alloy_sol_types::private::Address) -> Self {
            self.set_address(address);
            self
        }
        /// Returns a reference to the provider.
        #[inline]
        pub const fn provider(&self) -> &P {
            &self.provider
        }
    }
    impl<T, P: ::core::clone::Clone, N> BLSSignatureCheckerInstance<T, &P, N> {
        /// Clones the provider and returns a new instance with the cloned provider.
        #[inline]
        pub fn with_cloned_provider(self) -> BLSSignatureCheckerInstance<T, P, N> {
            BLSSignatureCheckerInstance {
                address: self.address,
                provider: ::core::clone::Clone::clone(&self.provider),
                _network_transport: ::core::marker::PhantomData,
            }
        }
    }
    /// Function calls.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > BLSSignatureCheckerInstance<T, P, N>
    {
        /// Creates a new call builder using this contract instance's provider and address.
        ///
        /// Note that the call can be any function call, not just those defined in this
        /// contract. Prefer using the other methods for building type-safe contract calls.
        pub fn call_builder<C: alloy_sol_types::SolCall>(
            &self,
            call: &C,
        ) -> alloy_contract::SolCallBuilder<T, &P, C, N> {
            alloy_contract::SolCallBuilder::new_sol(&self.provider, &self.address, call)
        }
        ///Creates a new call builder for the [`blsApkRegistry`] function.
        pub fn blsApkRegistry(
            &self,
        ) -> alloy_contract::SolCallBuilder<T, &P, blsApkRegistryCall, N> {
            self.call_builder(&blsApkRegistryCall {})
        }
        ///Creates a new call builder for the [`checkSignatures`] function.
        pub fn checkSignatures(
            &self,
            msgHash: alloy::sol_types::private::FixedBytes<32>,
            quorumNumbers: alloy::sol_types::private::Bytes,
            referenceBlockNumber: u32,
            params: <IBLSSignatureCheckerTypes::NonSignerStakesAndSignature as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<T, &P, checkSignaturesCall, N> {
            self.call_builder(&checkSignaturesCall {
                msgHash,
                quorumNumbers,
                referenceBlockNumber,
                params,
            })
        }
        ///Creates a new call builder for the [`delegation`] function.
        pub fn delegation(&self) -> alloy_contract::SolCallBuilder<T, &P, delegationCall, N> {
            self.call_builder(&delegationCall {})
        }
        ///Creates a new call builder for the [`registryCoordinator`] function.
        pub fn registryCoordinator(
            &self,
        ) -> alloy_contract::SolCallBuilder<T, &P, registryCoordinatorCall, N> {
            self.call_builder(&registryCoordinatorCall {})
        }
        ///Creates a new call builder for the [`stakeRegistry`] function.
        pub fn stakeRegistry(&self) -> alloy_contract::SolCallBuilder<T, &P, stakeRegistryCall, N> {
            self.call_builder(&stakeRegistryCall {})
        }
        ///Creates a new call builder for the [`trySignatureAndApkVerification`] function.
        pub fn trySignatureAndApkVerification(
            &self,
            msgHash: alloy::sol_types::private::FixedBytes<32>,
            apk: <BN254::G1Point as alloy::sol_types::SolType>::RustType,
            apkG2: <BN254::G2Point as alloy::sol_types::SolType>::RustType,
            sigma: <BN254::G1Point as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<T, &P, trySignatureAndApkVerificationCall, N> {
            self.call_builder(&trySignatureAndApkVerificationCall {
                msgHash,
                apk,
                apkG2,
                sigma,
            })
        }
    }
    /// Event filters.
    impl<
        T: alloy_contract::private::Transport + ::core::clone::Clone,
        P: alloy_contract::private::Provider<T, N>,
        N: alloy_contract::private::Network,
    > BLSSignatureCheckerInstance<T, P, N>
    {
        /// Creates a new event filter using this contract instance's provider and address.
        ///
        /// Note that the type can be any event, not just those defined in this contract.
        /// Prefer using the other methods for building type-safe event filters.
        pub fn event_filter<E: alloy_sol_types::SolEvent>(
            &self,
        ) -> alloy_contract::Event<T, &P, E, N> {
            alloy_contract::Event::new_sol(&self.provider, &self.address)
        }
    }
}
