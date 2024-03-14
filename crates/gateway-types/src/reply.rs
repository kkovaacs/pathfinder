//! Structures used for deserializing replies from Starkware's sequencer REST API.
use pathfinder_common::{
    BlockCommitmentSignatureElem, BlockHash, BlockNumber, BlockTimestamp, ContractAddress,
    EthereumAddress, EventCommitment, GasPrice, SequencerAddress, StarknetVersion, StateCommitment,
    StateDiffCommitment, TransactionCommitment,
};
use pathfinder_serde::{EthereumAddressAsHexStr, GasPriceAsHexStr};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub use transaction::DataAvailabilityMode;

// TODO Make all the gas price fields private and expose getters

/// Used to deserialize replies to Starknet block requests.
#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub block_hash: BlockHash,
    pub block_number: BlockNumber,

    /// Excluded in blocks prior to Starknet 0.9.
    ///
    /// This field is an implementation detail. Use the `eth_l1_gas_price`
    /// method instead of using this field directly.
    // TODO: remove alias after Starknet 0.13.0 is deployed on all networks
    #[serde_as(as = "Option<GasPriceAsHexStr>")]
    #[serde(default, alias = "gas_price", rename = "eth_l1_gas_price")]
    #[doc(hidden)]
    pub eth_l1_gas_price_implementation_detail: Option<GasPrice>,
    /// This field is an implementation detail. Use the `strk_l1_gas_price`
    /// method instead of using this field directly.
    #[serde_as(as = "Option<GasPriceAsHexStr>")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "strk_l1_gas_price"
    )]
    pub strk_l1_gas_price_implementation_detail: Option<GasPrice>,

    /// Excluded in blocks prior to Starknet 0.13.1.
    pub l1_data_gas_price: Option<GasPrices>,
    /// Excluded in blocks prior to Starknet 0.13.1.
    ///
    /// This field is an implementation detail. Use the `eth_l1_gas_price` and
    /// `strk_l1_gas_price` methods instead.
    #[serde(rename = "l1_gas_price")]
    pub l1_gas_price_implementation_detail: Option<GasPrices>,

    pub parent_block_hash: BlockHash,
    /// Excluded in blocks prior to Starknet 0.8
    #[serde(default)]
    pub sequencer_address: Option<SequencerAddress>,
    // Historical blocks (pre v0.11) still use `state_root`.
    #[serde(alias = "state_root")]
    pub state_commitment: StateCommitment,
    pub status: Status,
    pub timestamp: BlockTimestamp,
    #[serde_as(as = "Vec<transaction::Receipt>")]
    pub transaction_receipts: Vec<(
        pathfinder_common::receipt::Receipt,
        Vec<pathfinder_common::event::Event>,
    )>,
    #[serde_as(as = "Vec<transaction::Transaction>")]
    pub transactions: Vec<pathfinder_common::transaction::Transaction>,
    /// Version metadata introduced in 0.9.1, older blocks will not have it.
    #[serde(default)]
    pub starknet_version: StarknetVersion,

    // Introduced in v0.13.1
    #[serde(default)]
    pub transaction_commitment: Option<TransactionCommitment>,
    #[serde(default)]
    pub event_commitment: Option<EventCommitment>,
    #[serde(default)]
    pub l1_da_mode: Option<L1DataAvailabilityMode>,
}

impl Block {
    pub fn eth_l1_gas_price(&self) -> Option<GasPrice> {
        self.l1_gas_price_implementation_detail
            .map(|p| p.price_in_wei)
            .or(self.eth_l1_gas_price_implementation_detail)
    }

    pub fn strk_l1_gas_price(&self) -> Option<GasPrice> {
        self.l1_gas_price_implementation_detail
            .map(|p| p.price_in_fri)
            .or(self.strk_l1_gas_price_implementation_detail)
    }
}

#[serde_as]
#[derive(Clone, Default, Debug, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct PendingBlock {
    /// Excluded in blocks prior to Starknet 0.9.
    ///
    /// This field is an implementation detail. Use the `eth_l1_gas_price`
    /// method instead of using this field directly.
    // TODO: remove alias after Starknet 0.13.0 is deployed on all networks
    #[serde_as(as = "Option<GasPriceAsHexStr>")]
    #[serde(default, alias = "gas_price", rename = "eth_l1_gas_price")]
    pub eth_l1_gas_price_implementation_detail: Option<GasPrice>,
    /// This field is an implementation detail. Use the `strk_l1_gas_price`
    /// method instead of using this field directly.
    #[serde_as(as = "Option<GasPriceAsHexStr>")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "strk_l1_gas_price"
    )]
    pub strk_l1_gas_price_implementation_detail: Option<GasPrice>,

    /// Excluded in blocks prior to Starknet 0.13.1.
    pub l1_data_gas_price: Option<GasPrices>,
    /// Excluded in blocks prior to Starknet 0.13.1.
    ///
    /// This field is an implementation detail. Use the `strk_l1_gas_price`
    /// method instead of using this field directly.
    #[serde(rename = "l1_gas_price")]
    pub l1_gas_price_implementation_detail: Option<GasPrices>,

    #[serde(rename = "parent_block_hash")]
    pub parent_hash: BlockHash,
    pub sequencer_address: SequencerAddress,
    pub status: Status,
    pub timestamp: BlockTimestamp,
    #[serde_as(as = "Vec<transaction::Receipt>")]
    pub transaction_receipts: Vec<(
        pathfinder_common::receipt::Receipt,
        Vec<pathfinder_common::event::Event>,
    )>,
    #[serde_as(as = "Vec<transaction::Transaction>")]
    pub transactions: Vec<pathfinder_common::transaction::Transaction>,
    /// Version metadata introduced in 0.9.1, older blocks will not have it.
    #[serde(default)]
    pub starknet_version: StarknetVersion,
    #[serde(default)]
    pub l1_da_mode: Option<L1DataAvailabilityMode>,
}

impl PendingBlock {
    pub fn eth_l1_gas_price(&self) -> GasPrice {
        self.l1_gas_price_implementation_detail
            .map(|p| p.price_in_wei)
            .or(self.eth_l1_gas_price_implementation_detail)
            .expect("missing L1 gas price")
    }

    pub fn strk_l1_gas_price(&self) -> Option<GasPrice> {
        self.l1_gas_price_implementation_detail
            .map(|p| p.price_in_fri)
            .or(self.strk_l1_gas_price_implementation_detail)
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum L1DataAvailabilityMode {
    #[default]
    Calldata,
    Blob,
}

impl From<L1DataAvailabilityMode> for pathfinder_common::L1DataAvailabilityMode {
    fn from(value: L1DataAvailabilityMode) -> Self {
        match value {
            L1DataAvailabilityMode::Calldata => Self::Calldata,
            L1DataAvailabilityMode::Blob => Self::Blob,
        }
    }
}

impl From<pathfinder_common::L1DataAvailabilityMode> for L1DataAvailabilityMode {
    fn from(value: pathfinder_common::L1DataAvailabilityMode) -> Self {
        match value {
            pathfinder_common::L1DataAvailabilityMode::Calldata => Self::Calldata,
            pathfinder_common::L1DataAvailabilityMode::Blob => Self::Blob,
        }
    }
}

#[serde_as]
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GasPrices {
    #[serde_as(as = "GasPriceAsHexStr")]
    pub price_in_wei: GasPrice,
    #[serde_as(as = "GasPriceAsHexStr")]
    pub price_in_fri: GasPrice,
}

/// Block and transaction status values.
#[derive(Copy, Clone, Default, Debug, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum Status {
    #[serde(rename = "NOT_RECEIVED")]
    NotReceived,
    #[serde(rename = "RECEIVED")]
    Received,
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "ACCEPTED_ON_L1")]
    AcceptedOnL1,
    #[serde(rename = "ACCEPTED_ON_L2")]
    #[default]
    AcceptedOnL2,
    #[serde(rename = "REVERTED")]
    Reverted,
    #[serde(rename = "ABORTED")]
    Aborted,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::NotReceived => write!(f, "NOT_RECEIVED"),
            Status::Received => write!(f, "RECEIVED"),
            Status::Pending => write!(f, "PENDING"),
            Status::Rejected => write!(f, "REJECTED"),
            Status::AcceptedOnL1 => write!(f, "ACCEPTED_ON_L1"),
            Status::AcceptedOnL2 => write!(f, "ACCEPTED_ON_L2"),
            Status::Reverted => write!(f, "REVERTED"),
            Status::Aborted => write!(f, "ABORTED"),
        }
    }
}

/// Types used when deserializing L2 call related data.
pub mod call {
    use serde::Deserialize;
    use serde_with::serde_as;
    use std::collections::HashMap;

    /// Describes problems encountered during some of call failures .
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Problems {
        #[serde_as(as = "HashMap<_, _>")]
        pub calldata: HashMap<u64, Vec<String>>,
    }
}

/// Used to deserialize replies to Starknet transaction requests.
///
/// We only care about the statuses so we ignore other fields.
/// Please note that this does not have to be backwards compatible:
/// since we only ever use it to deserialize replies from the Starknet
/// feeder gateway.
#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TransactionStatus {
    pub status: Status,
    pub finality_status: transaction_status::FinalityStatus,
    #[serde(default)]
    pub execution_status: transaction_status::ExecutionStatus,
}

/// Types used when deserializing get_transaction replies.
pub mod transaction_status {
    use serde::Deserialize;

    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    pub enum FinalityStatus {
        #[serde(rename = "NOT_RECEIVED")]
        NotReceived,
        #[serde(rename = "RECEIVED")]
        Received,
        #[serde(rename = "ACCEPTED_ON_L1")]
        AcceptedOnL1,
        #[serde(rename = "ACCEPTED_ON_L2")]
        AcceptedOnL2,
    }

    #[derive(Clone, Default, Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum ExecutionStatus {
        #[default]
        Succeeded,
        Reverted,
        Rejected,
    }
}

/// Types used when deserializing L2 transaction related data.
pub(crate) mod transaction {
    use fake::{Dummy, Fake, Faker};
    use pathfinder_common::{
        AccountDeploymentDataElem, CallParam, CasmHash, ClassHash, ConstructorParam,
        ContractAddress, ContractAddressSalt, EntryPoint, EthereumAddress, Fee, L1ToL2MessageNonce,
        L1ToL2MessagePayloadElem, L2ToL1MessagePayloadElem, PaymasterDataElem, ResourceAmount,
        ResourcePricePerUnit, Tip, TransactionHash, TransactionIndex, TransactionNonce,
        TransactionSignatureElem, TransactionVersion,
    };
    use pathfinder_crypto::Felt;
    use pathfinder_serde::{
        CallParamAsDecimalStr, ConstructorParamAsDecimalStr, EthereumAddressAsHexStr,
        L1ToL2MessagePayloadElemAsDecimalStr, L2ToL1MessagePayloadElemAsDecimalStr,
        ResourceAmountAsHexStr, ResourcePricePerUnitAsHexStr, TipAsHexStr,
        TransactionSignatureElemAsDecimalStr,
    };
    use primitive_types::H256;
    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;

    /// Represents deserialized L2 transaction entry point values.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    #[serde(deny_unknown_fields)]
    pub enum EntryPointType {
        #[serde(rename = "EXTERNAL")]
        External,
        #[serde(rename = "L1_HANDLER")]
        L1Handler,
    }

    impl From<pathfinder_common::transaction::EntryPointType> for EntryPointType {
        fn from(value: pathfinder_common::transaction::EntryPointType) -> Self {
            use pathfinder_common::transaction::EntryPointType::{External, L1Handler};
            match value {
                External => Self::External,
                L1Handler => Self::L1Handler,
            }
        }
    }

    impl From<EntryPointType> for pathfinder_common::transaction::EntryPointType {
        fn from(value: EntryPointType) -> Self {
            match value {
                EntryPointType::External => Self::External,
                EntryPointType::L1Handler => Self::L1Handler,
            }
        }
    }

    /// Represents execution resources for L2 transaction.
    #[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct ExecutionResources {
        pub builtin_instance_counter: BuiltinCounters,
        pub n_steps: u64,
        pub n_memory_holes: u64,
        pub data_availability: Option<ExecutionDataAvailability>,
    }

    impl From<ExecutionResources> for pathfinder_common::receipt::ExecutionResources {
        fn from(value: ExecutionResources) -> Self {
            Self {
                builtins: value.builtin_instance_counter.into(),
                n_steps: value.n_steps,
                n_memory_holes: value.n_memory_holes,
                data_availability: value.data_availability.unwrap_or_default().into(),
            }
        }
    }

    impl From<pathfinder_common::receipt::ExecutionResources> for ExecutionResources {
        fn from(value: pathfinder_common::receipt::ExecutionResources) -> Self {
            Self {
                builtin_instance_counter: value.builtins.into(),
                n_steps: value.n_steps,
                n_memory_holes: value.n_memory_holes,
                data_availability: Some(value.data_availability.into()),
            }
        }
    }

    #[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct ExecutionDataAvailability {
        pub l1_gas: u128,
        pub l1_data_gas: u128,
    }

    impl From<ExecutionDataAvailability> for pathfinder_common::receipt::ExecutionDataAvailability {
        fn from(value: ExecutionDataAvailability) -> Self {
            Self {
                l1_gas: value.l1_gas,
                l1_data_gas: value.l1_data_gas,
            }
        }
    }

    impl From<pathfinder_common::receipt::ExecutionDataAvailability> for ExecutionDataAvailability {
        fn from(value: pathfinder_common::receipt::ExecutionDataAvailability) -> Self {
            Self {
                l1_gas: value.l1_gas,
                l1_data_gas: value.l1_data_gas,
            }
        }
    }

    impl<T> Dummy<T> for ExecutionResources {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            Self {
                builtin_instance_counter: Faker.fake_with_rng(rng),
                n_steps: rng.next_u32() as u64,
                n_memory_holes: rng.next_u32() as u64,
                data_availability: Some(ExecutionDataAvailability {
                    l1_gas: rng.next_u32() as u128,
                    l1_data_gas: rng.next_u32() as u128,
                }),
            }
        }
    }

    // This struct purposefully allows for unknown fields as it is not critical to
    // store these counters perfectly. Failure would be far more costly than simply
    // ignoring them.
    #[derive(Copy, Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(default)]
    pub struct BuiltinCounters {
        pub output_builtin: u64,
        pub pedersen_builtin: u64,
        pub range_check_builtin: u64,
        pub ecdsa_builtin: u64,
        pub bitwise_builtin: u64,
        pub ec_op_builtin: u64,
        pub keccak_builtin: u64,
        pub poseidon_builtin: u64,
        pub segment_arena_builtin: u64, // TODO REMOVE (?)
    }

    impl From<BuiltinCounters> for pathfinder_common::receipt::BuiltinCounters {
        fn from(value: BuiltinCounters) -> Self {
            // Use deconstruction to ensure these structs remain in-sync.
            let BuiltinCounters {
                output_builtin,
                pedersen_builtin,
                range_check_builtin,
                ecdsa_builtin,
                bitwise_builtin,
                ec_op_builtin,
                keccak_builtin,
                poseidon_builtin,
                segment_arena_builtin,
            } = value;
            Self {
                output: output_builtin,
                pedersen: pedersen_builtin,
                range_check: range_check_builtin,
                ecdsa: ecdsa_builtin,
                bitwise: bitwise_builtin,
                ec_op: ec_op_builtin,
                keccak: keccak_builtin,
                poseidon: poseidon_builtin,
                segment_arena: segment_arena_builtin,
            }
        }
    }

    impl From<pathfinder_common::receipt::BuiltinCounters> for BuiltinCounters {
        fn from(value: pathfinder_common::receipt::BuiltinCounters) -> Self {
            // Use deconstruction to ensure these structs remain in-sync.
            let pathfinder_common::receipt::BuiltinCounters {
                output: output_builtin,
                pedersen: pedersen_builtin,
                range_check: range_check_builtin,
                ecdsa: ecdsa_builtin,
                bitwise: bitwise_builtin,
                ec_op: ec_op_builtin,
                keccak: keccak_builtin,
                poseidon: poseidon_builtin,
                segment_arena: segment_arena_builtin,
            } = value;
            Self {
                output_builtin,
                pedersen_builtin,
                range_check_builtin,
                ecdsa_builtin,
                bitwise_builtin,
                ec_op_builtin,
                keccak_builtin,
                poseidon_builtin,
                segment_arena_builtin,
            }
        }
    }

    impl<T> Dummy<T> for BuiltinCounters {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            Self {
                output_builtin: rng.next_u32() as u64,
                pedersen_builtin: rng.next_u32() as u64,
                range_check_builtin: rng.next_u32() as u64,
                ecdsa_builtin: rng.next_u32() as u64,
                bitwise_builtin: rng.next_u32() as u64,
                ec_op_builtin: rng.next_u32() as u64,
                keccak_builtin: rng.next_u32() as u64,
                poseidon_builtin: rng.next_u32() as u64,
                segment_arena_builtin: 0, // Not used in p2p
            }
        }
    }

    /// Represents deserialized L1 to L2 message.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct L1ToL2Message {
        #[serde_as(as = "EthereumAddressAsHexStr")]
        pub from_address: EthereumAddress,
        #[serde_as(as = "Vec<L1ToL2MessagePayloadElemAsDecimalStr>")]
        pub payload: Vec<L1ToL2MessagePayloadElem>,
        pub selector: EntryPoint,
        pub to_address: ContractAddress,
        #[serde(default)]
        pub nonce: Option<L1ToL2MessageNonce>,
    }

    impl<T> Dummy<T> for L1ToL2Message {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            // Nonces were missing in very old messages, we don't care about it
            Self {
                from_address: Faker.fake_with_rng(rng),
                payload: Faker.fake_with_rng(rng),
                selector: Faker.fake_with_rng(rng),
                to_address: Faker.fake_with_rng(rng),
                nonce: Some(Faker.fake_with_rng(rng)),
            }
        }
    }

    /// Represents deserialized L2 to L1 message.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    #[serde(deny_unknown_fields)]
    pub struct L2ToL1Message {
        pub from_address: ContractAddress,
        #[serde_as(as = "Vec<L2ToL1MessagePayloadElemAsDecimalStr>")]
        pub payload: Vec<L2ToL1MessagePayloadElem>,
        #[serde_as(as = "EthereumAddressAsHexStr")]
        pub to_address: EthereumAddress,
    }

    impl From<L2ToL1Message> for pathfinder_common::receipt::L2ToL1Message {
        fn from(value: L2ToL1Message) -> Self {
            let L2ToL1Message {
                from_address,
                payload,
                to_address,
            } = value;
            pathfinder_common::receipt::L2ToL1Message {
                from_address,
                payload,
                to_address,
            }
        }
    }

    impl From<pathfinder_common::receipt::L2ToL1Message> for L2ToL1Message {
        fn from(value: pathfinder_common::receipt::L2ToL1Message) -> Self {
            let pathfinder_common::receipt::L2ToL1Message {
                from_address,
                payload,
                to_address,
            } = value;
            Self {
                from_address,
                payload,
                to_address,
            }
        }
    }

    #[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum ExecutionStatus {
        // This must be the default as pre v0.12.1 receipts did not contain this value and
        // were always success as reverted did not exist.
        #[default]
        Succeeded,
        Reverted,
    }

    /// Represents deserialized L2 transaction receipt data.
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Receipt {
        #[serde(default)]
        pub actual_fee: Option<Fee>,
        pub events: Vec<pathfinder_common::event::Event>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub execution_resources: Option<ExecutionResources>,
        pub l1_to_l2_consumed_message: Option<L1ToL2Message>,
        pub l2_to_l1_messages: Vec<L2ToL1Message>,
        pub transaction_hash: TransactionHash,
        pub transaction_index: TransactionIndex,
        // Introduced in v0.12.1
        #[serde(default)]
        pub execution_status: ExecutionStatus,
        // Introduced in v0.12.1
        /// Only present if status is [ExecutionStatus::Reverted].
        #[serde(default)]
        pub revert_error: Option<String>,
    }

    impl
        From<(
            pathfinder_common::receipt::Receipt,
            Vec<pathfinder_common::event::Event>,
        )> for Receipt
    {
        fn from(
            (receipt, events): (
                pathfinder_common::receipt::Receipt,
                Vec<pathfinder_common::event::Event>,
            ),
        ) -> Self {
            let pathfinder_common::receipt::Receipt {
                actual_fee,
                execution_resources,
                l2_to_l1_messages,
                execution_status,
                transaction_hash,
                transaction_index,
            } = receipt;

            let (execution_status, revert_error) = match execution_status {
                pathfinder_common::receipt::ExecutionStatus::Succeeded => {
                    (ExecutionStatus::Succeeded, None)
                }
                pathfinder_common::receipt::ExecutionStatus::Reverted { reason } => {
                    (ExecutionStatus::Reverted, Some(reason))
                }
            };

            Self {
                actual_fee,
                events,
                execution_resources: Some(execution_resources.into()),
                l1_to_l2_consumed_message: None,
                l2_to_l1_messages: l2_to_l1_messages.into_iter().map(Into::into).collect(),
                transaction_hash,
                transaction_index,
                execution_status,
                revert_error,
            }
        }
    }

    impl<'de>
        serde_with::DeserializeAs<
            'de,
            (
                pathfinder_common::receipt::Receipt,
                Vec<pathfinder_common::event::Event>,
            ),
        > for Receipt
    {
        fn deserialize_as<D>(
            deserializer: D,
        ) -> Result<
            (
                pathfinder_common::receipt::Receipt,
                Vec<pathfinder_common::event::Event>,
            ),
            D::Error,
        >
        where
            D: serde::Deserializer<'de>,
        {
            Self::deserialize(deserializer).map(Into::into)
        }
    }

    impl
        serde_with::SerializeAs<(
            pathfinder_common::receipt::Receipt,
            Vec<pathfinder_common::event::Event>,
        )> for Receipt
    {
        fn serialize_as<S>(
            (receipt, events): &(
                pathfinder_common::receipt::Receipt,
                Vec<pathfinder_common::event::Event>,
            ),
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Self::from((receipt.clone(), events.clone())).serialize(serializer)
        }
    }

    impl From<Receipt>
        for (
            pathfinder_common::receipt::Receipt,
            Vec<pathfinder_common::event::Event>,
        )
    {
        fn from(value: Receipt) -> Self {
            use pathfinder_common::receipt as common;

            let Receipt {
                actual_fee,
                events,
                execution_resources,
                // This information is redundant as it is already in the transaction itself.
                l1_to_l2_consumed_message: _,
                l2_to_l1_messages,
                transaction_hash,
                transaction_index,
                execution_status,
                revert_error,
            } = value;

            (
                common::Receipt {
                    actual_fee,
                    execution_resources: execution_resources.unwrap_or_default().into(),
                    l2_to_l1_messages: l2_to_l1_messages.into_iter().map(Into::into).collect(),
                    transaction_hash,
                    transaction_index,
                    execution_status: match execution_status {
                        ExecutionStatus::Succeeded => common::ExecutionStatus::Succeeded,
                        ExecutionStatus::Reverted => common::ExecutionStatus::Reverted {
                            reason: revert_error.unwrap_or_default(),
                        },
                    },
                },
                events,
            )
        }
    }

    impl<T> Dummy<T> for Receipt {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            let execution_status = Faker.fake_with_rng(rng);
            let revert_error =
                (execution_status == ExecutionStatus::Reverted).then(|| Faker.fake_with_rng(rng));

            // Those fields that were missing in very old receipts are always present
            Self {
                actual_fee: Some(Faker.fake_with_rng(rng)),
                execution_resources: Some(Faker.fake_with_rng(rng)),
                events: Faker.fake_with_rng(rng),
                l1_to_l2_consumed_message: Faker.fake_with_rng(rng),
                l2_to_l1_messages: Faker.fake_with_rng(rng),
                transaction_hash: Faker.fake_with_rng(rng),
                transaction_index: Faker.fake_with_rng(rng),
                execution_status,
                revert_error,
            }
        }
    }

    #[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Dummy)]
    pub enum DataAvailabilityMode {
        #[default]
        L1,
        L2,
    }

    impl Serialize for DataAvailabilityMode {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                DataAvailabilityMode::L1 => serializer.serialize_u8(0),
                DataAvailabilityMode::L2 => serializer.serialize_u8(1),
            }
        }
    }

    impl<'de> Deserialize<'de> for DataAvailabilityMode {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            match <u8 as Deserialize>::deserialize(deserializer)? {
                0 => Ok(Self::L1),
                1 => Ok(Self::L2),
                _ => Err(serde::de::Error::custom("invalid data availability mode")),
            }
        }
    }

    impl From<DataAvailabilityMode> for pathfinder_common::transaction::DataAvailabilityMode {
        fn from(value: DataAvailabilityMode) -> Self {
            match value {
                DataAvailabilityMode::L1 => Self::L1,
                DataAvailabilityMode::L2 => Self::L2,
            }
        }
    }

    impl From<pathfinder_common::transaction::DataAvailabilityMode> for DataAvailabilityMode {
        fn from(value: pathfinder_common::transaction::DataAvailabilityMode) -> Self {
            match value {
                pathfinder_common::transaction::DataAvailabilityMode::L1 => Self::L1,
                pathfinder_common::transaction::DataAvailabilityMode::L2 => Self::L2,
            }
        }
    }

    #[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    pub struct ResourceBounds {
        #[serde(rename = "L1_GAS")]
        pub l1_gas: ResourceBound,
        #[serde(rename = "L2_GAS")]
        pub l2_gas: ResourceBound,
    }

    impl From<ResourceBounds> for pathfinder_common::transaction::ResourceBounds {
        fn from(value: ResourceBounds) -> Self {
            Self {
                l1_gas: value.l1_gas.into(),
                l2_gas: value.l2_gas.into(),
            }
        }
    }

    impl From<pathfinder_common::transaction::ResourceBounds> for ResourceBounds {
        fn from(value: pathfinder_common::transaction::ResourceBounds) -> Self {
            Self {
                l1_gas: value.l1_gas.into(),
                l2_gas: value.l2_gas.into(),
            }
        }
    }

    #[serde_as]
    #[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    pub struct ResourceBound {
        #[serde_as(as = "ResourceAmountAsHexStr")]
        pub max_amount: ResourceAmount,
        #[serde_as(as = "ResourcePricePerUnitAsHexStr")]
        pub max_price_per_unit: ResourcePricePerUnit,
    }

    impl From<ResourceBound> for pathfinder_common::transaction::ResourceBound {
        fn from(value: ResourceBound) -> Self {
            Self {
                max_amount: value.max_amount,
                max_price_per_unit: value.max_price_per_unit,
            }
        }
    }

    impl From<pathfinder_common::transaction::ResourceBound> for ResourceBound {
        fn from(value: pathfinder_common::transaction::ResourceBound) -> Self {
            Self {
                max_amount: value.max_amount,
                max_price_per_unit: value.max_price_per_unit,
            }
        }
    }

    /// Represents deserialized L2 transaction data.
    #[derive(Clone, Debug, Serialize, PartialEq, Eq, Dummy)]
    #[serde(tag = "type")]
    #[serde(deny_unknown_fields)]
    pub enum Transaction {
        #[serde(rename = "DECLARE")]
        Declare(DeclareTransaction),
        #[serde(rename = "DEPLOY")]
        // FIXME regenesis: remove Deploy txn type after regenesis
        // We are keeping this type of transaction until regenesis
        // only to support older pre-0.11.0 blocks
        Deploy(DeployTransaction),
        #[serde(rename = "DEPLOY_ACCOUNT")]
        DeployAccount(DeployAccountTransaction),
        #[serde(rename = "INVOKE_FUNCTION")]
        Invoke(InvokeTransaction),
        #[serde(rename = "L1_HANDLER")]
        L1Handler(L1HandlerTransaction),
    }

    // This manual deserializtion is a work-around for L1 handler transactions
    // historically being served as Invoke V0. However, the gateway has retroactively
    // changed these to L1 handlers. This means older databases will have these as Invoke
    // but modern one's as L1 handler. This causes confusion, so we convert these old Invoke
    // to L1 handler manually.
    //
    // The alternative is to do a costly database migration which involves opening every tx.
    //
    // This work-around may be removed once we are certain all databases no longer contain these
    // transactions, which will likely only occur after either a migration, or regenesis.
    impl<'de> Deserialize<'de> for Transaction {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            /// Copy of [Transaction] to deserialize into, before converting to [Transaction]
            /// with the potential Invoke V0 -> L1 handler cast.
            #[derive(Deserialize)]
            #[serde(tag = "type", deny_unknown_fields)]
            pub enum InnerTransaction {
                #[serde(rename = "DECLARE")]
                Declare(DeclareTransaction),
                #[serde(rename = "DEPLOY")]
                Deploy(DeployTransaction),
                #[serde(rename = "DEPLOY_ACCOUNT")]
                DeployAccount(DeployAccountTransaction),
                #[serde(rename = "INVOKE_FUNCTION")]
                Invoke(InvokeTransaction),
                #[serde(rename = "L1_HANDLER")]
                L1Handler(L1HandlerTransaction),
            }

            let tx = InnerTransaction::deserialize(deserializer)?;
            let tx = match tx {
                InnerTransaction::Declare(x) => Transaction::Declare(x),
                InnerTransaction::Deploy(x) => Transaction::Deploy(x),
                InnerTransaction::DeployAccount(x) => Transaction::DeployAccount(x),
                InnerTransaction::Invoke(InvokeTransaction::V0(i))
                    if i.entry_point_type == Some(EntryPointType::L1Handler) =>
                {
                    let l1_handler = L1HandlerTransaction {
                        contract_address: i.sender_address,
                        entry_point_selector: i.entry_point_selector,
                        nonce: TransactionNonce::ZERO,
                        calldata: i.calldata,
                        transaction_hash: i.transaction_hash,
                        version: TransactionVersion::ZERO,
                    };

                    Transaction::L1Handler(l1_handler)
                }
                InnerTransaction::Invoke(x) => Transaction::Invoke(x),
                InnerTransaction::L1Handler(x) => Transaction::L1Handler(x),
            };

            Ok(tx)
        }
    }

    impl<'de> serde_with::DeserializeAs<'de, pathfinder_common::transaction::Transaction>
        for Transaction
    {
        fn deserialize_as<D>(
            deserializer: D,
        ) -> Result<pathfinder_common::transaction::Transaction, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Self::deserialize(deserializer).map(Into::into)
        }
    }

    impl serde_with::SerializeAs<pathfinder_common::transaction::Transaction> for Transaction {
        fn serialize_as<S>(
            source: &pathfinder_common::transaction::Transaction,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Self::from(source.clone()).serialize(serializer)
        }
    }

    impl From<pathfinder_common::transaction::Transaction> for Transaction {
        fn from(value: pathfinder_common::transaction::Transaction) -> Self {
            use pathfinder_common::transaction::TransactionVariant::*;
            use pathfinder_common::transaction::*;

            let transaction_hash = value.hash;
            match value.variant {
                DeclareV0(DeclareTransactionV0V1 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                }) => Self::Declare(DeclareTransaction::V0(self::DeclareTransactionV0V1 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    transaction_hash,
                })),
                DeclareV1(DeclareTransactionV0V1 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                }) => Self::Declare(DeclareTransaction::V1(self::DeclareTransactionV0V1 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    transaction_hash,
                })),
                DeclareV2(DeclareTransactionV2 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    compiled_class_hash,
                }) => Self::Declare(DeclareTransaction::V2(self::DeclareTransactionV2 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    transaction_hash,
                    compiled_class_hash,
                })),
                DeclareV3(DeclareTransactionV3 {
                    class_hash,
                    nonce,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    resource_bounds,
                    tip,
                    paymaster_data,
                    signature,
                    account_deployment_data,
                    sender_address,
                    compiled_class_hash,
                }) => Self::Declare(DeclareTransaction::V3(self::DeclareTransactionV3 {
                    class_hash,
                    nonce,
                    nonce_data_availability_mode: nonce_data_availability_mode.into(),
                    fee_data_availability_mode: fee_data_availability_mode.into(),
                    resource_bounds: resource_bounds.into(),
                    tip,
                    paymaster_data,
                    sender_address,
                    signature,
                    transaction_hash,
                    compiled_class_hash,
                    account_deployment_data,
                })),
                Deploy(DeployTransaction {
                    contract_address,
                    contract_address_salt,
                    class_hash,
                    constructor_calldata,
                    version,
                }) => Self::Deploy(self::DeployTransaction {
                    contract_address,
                    contract_address_salt,
                    class_hash,
                    constructor_calldata,
                    transaction_hash,
                    version,
                }),
                DeployAccountV0V1(DeployAccountTransactionV0V1 {
                    contract_address,
                    max_fee,
                    version,
                    signature,
                    nonce,
                    contract_address_salt,
                    constructor_calldata,
                    class_hash,
                }) => Self::DeployAccount(self::DeployAccountTransaction::V0V1(
                    self::DeployAccountTransactionV0V1 {
                        contract_address,
                        transaction_hash,
                        max_fee,
                        version,
                        signature,
                        nonce,
                        contract_address_salt,
                        constructor_calldata,
                        class_hash,
                    },
                )),
                DeployAccountV3(DeployAccountTransactionV3 {
                    contract_address,
                    signature,
                    nonce,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    resource_bounds,
                    tip,
                    paymaster_data,
                    contract_address_salt,
                    constructor_calldata,
                    class_hash,
                }) => Self::DeployAccount(self::DeployAccountTransaction::V3(
                    self::DeployAccountTransactionV3 {
                        nonce,
                        nonce_data_availability_mode: nonce_data_availability_mode.into(),
                        fee_data_availability_mode: fee_data_availability_mode.into(),
                        resource_bounds: resource_bounds.into(),
                        tip,
                        paymaster_data,
                        sender_address: contract_address,
                        signature,
                        transaction_hash,
                        version: TransactionVersion::THREE,
                        contract_address_salt,
                        constructor_calldata,
                        class_hash,
                    },
                )),
                InvokeV0(InvokeTransactionV0 {
                    calldata,
                    sender_address,
                    entry_point_selector,
                    entry_point_type,
                    max_fee,
                    signature,
                }) => Self::Invoke(InvokeTransaction::V0(self::InvokeTransactionV0 {
                    calldata,
                    sender_address,
                    entry_point_selector,
                    entry_point_type: entry_point_type.map(Into::into),
                    max_fee,
                    signature,
                    transaction_hash,
                })),
                InvokeV1(InvokeTransactionV1 {
                    calldata,
                    sender_address,
                    max_fee,
                    signature,
                    nonce,
                }) => Self::Invoke(InvokeTransaction::V1(self::InvokeTransactionV1 {
                    calldata,
                    sender_address,
                    max_fee,
                    signature,
                    nonce,
                    transaction_hash,
                })),
                InvokeV3(InvokeTransactionV3 {
                    signature,
                    nonce,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    resource_bounds,
                    tip,
                    paymaster_data,
                    account_deployment_data,
                    calldata,
                    sender_address,
                }) => Self::Invoke(InvokeTransaction::V3(self::InvokeTransactionV3 {
                    nonce,
                    nonce_data_availability_mode: nonce_data_availability_mode.into(),
                    fee_data_availability_mode: fee_data_availability_mode.into(),
                    resource_bounds: resource_bounds.into(),
                    tip,
                    paymaster_data,
                    sender_address,
                    signature,
                    transaction_hash,
                    calldata,
                    account_deployment_data,
                })),
                L1Handler(L1HandlerTransaction {
                    contract_address,
                    entry_point_selector,
                    nonce,
                    calldata,
                }) => Self::L1Handler(self::L1HandlerTransaction {
                    contract_address,
                    entry_point_selector,
                    nonce,
                    calldata,
                    transaction_hash,
                    version: TransactionVersion::ZERO,
                }),
            }
        }
    }

    impl From<Transaction> for pathfinder_common::transaction::Transaction {
        fn from(value: Transaction) -> Self {
            use pathfinder_common::transaction::TransactionVariant;

            let hash = value.hash();
            let variant = match value {
                Transaction::Declare(DeclareTransaction::V0(DeclareTransactionV0V1 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    transaction_hash: _,
                })) => TransactionVariant::DeclareV0(
                    pathfinder_common::transaction::DeclareTransactionV0V1 {
                        class_hash,
                        max_fee,
                        nonce,
                        sender_address,
                        signature,
                    },
                ),
                Transaction::Declare(DeclareTransaction::V1(DeclareTransactionV0V1 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    transaction_hash: _,
                })) => TransactionVariant::DeclareV1(
                    pathfinder_common::transaction::DeclareTransactionV0V1 {
                        class_hash,
                        max_fee,
                        nonce,
                        sender_address,
                        signature,
                    },
                ),
                Transaction::Declare(DeclareTransaction::V2(DeclareTransactionV2 {
                    class_hash,
                    max_fee,
                    nonce,
                    sender_address,
                    signature,
                    transaction_hash: _,
                    compiled_class_hash,
                })) => TransactionVariant::DeclareV2(
                    pathfinder_common::transaction::DeclareTransactionV2 {
                        class_hash,
                        max_fee,
                        nonce,
                        sender_address,
                        signature,
                        compiled_class_hash,
                    },
                ),
                Transaction::Declare(DeclareTransaction::V3(DeclareTransactionV3 {
                    class_hash,
                    nonce,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    resource_bounds,
                    tip,
                    paymaster_data,
                    sender_address,
                    signature,
                    transaction_hash: _,
                    compiled_class_hash,
                    account_deployment_data,
                })) => TransactionVariant::DeclareV3(
                    pathfinder_common::transaction::DeclareTransactionV3 {
                        class_hash,
                        nonce,
                        nonce_data_availability_mode: nonce_data_availability_mode.into(),
                        fee_data_availability_mode: fee_data_availability_mode.into(),
                        resource_bounds: resource_bounds.into(),
                        tip,
                        paymaster_data,
                        sender_address,
                        signature,
                        compiled_class_hash,
                        account_deployment_data,
                    },
                ),
                Transaction::Deploy(DeployTransaction {
                    contract_address,
                    contract_address_salt,
                    class_hash,
                    constructor_calldata,
                    transaction_hash: _,
                    version,
                }) => {
                    TransactionVariant::Deploy(pathfinder_common::transaction::DeployTransaction {
                        contract_address,
                        contract_address_salt,
                        class_hash,
                        constructor_calldata,
                        version,
                    })
                }
                Transaction::DeployAccount(DeployAccountTransaction::V0V1(
                    DeployAccountTransactionV0V1 {
                        contract_address,
                        transaction_hash: _,
                        max_fee,
                        version,
                        signature,
                        nonce,
                        contract_address_salt,
                        constructor_calldata,
                        class_hash,
                    },
                )) => TransactionVariant::DeployAccountV0V1(
                    pathfinder_common::transaction::DeployAccountTransactionV0V1 {
                        contract_address,
                        max_fee,
                        version,
                        signature,
                        nonce,
                        contract_address_salt,
                        constructor_calldata,
                        class_hash,
                    },
                ),
                Transaction::DeployAccount(DeployAccountTransaction::V3(
                    DeployAccountTransactionV3 {
                        nonce,
                        nonce_data_availability_mode,
                        fee_data_availability_mode,
                        resource_bounds,
                        tip,
                        paymaster_data,
                        sender_address,
                        signature,
                        transaction_hash: _,
                        version: _,
                        contract_address_salt,
                        constructor_calldata,
                        class_hash,
                    },
                )) => TransactionVariant::DeployAccountV3(
                    pathfinder_common::transaction::DeployAccountTransactionV3 {
                        contract_address: sender_address,
                        signature,
                        nonce,
                        nonce_data_availability_mode: nonce_data_availability_mode.into(),
                        fee_data_availability_mode: fee_data_availability_mode.into(),
                        resource_bounds: resource_bounds.into(),
                        tip,
                        paymaster_data,
                        contract_address_salt,
                        constructor_calldata,
                        class_hash,
                    },
                ),
                Transaction::Invoke(InvokeTransaction::V0(InvokeTransactionV0 {
                    calldata,
                    sender_address,
                    entry_point_selector,
                    entry_point_type,
                    max_fee,
                    signature,
                    transaction_hash: _,
                })) => TransactionVariant::InvokeV0(
                    pathfinder_common::transaction::InvokeTransactionV0 {
                        calldata,
                        sender_address,
                        entry_point_selector,
                        entry_point_type: entry_point_type.map(Into::into),
                        max_fee,
                        signature,
                    },
                ),
                Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
                    calldata,
                    sender_address,
                    max_fee,
                    signature,
                    nonce,
                    transaction_hash: _,
                })) => TransactionVariant::InvokeV1(
                    pathfinder_common::transaction::InvokeTransactionV1 {
                        calldata,
                        sender_address,
                        max_fee,
                        signature,
                        nonce,
                    },
                ),
                Transaction::Invoke(InvokeTransaction::V3(InvokeTransactionV3 {
                    nonce,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    resource_bounds,
                    tip,
                    paymaster_data,
                    sender_address,
                    signature,
                    transaction_hash: _,
                    calldata,
                    account_deployment_data,
                })) => TransactionVariant::InvokeV3(
                    pathfinder_common::transaction::InvokeTransactionV3 {
                        signature,
                        nonce,
                        nonce_data_availability_mode: nonce_data_availability_mode.into(),
                        fee_data_availability_mode: fee_data_availability_mode.into(),
                        resource_bounds: resource_bounds.into(),
                        tip,
                        paymaster_data,
                        account_deployment_data,
                        calldata,
                        sender_address,
                    },
                ),
                Transaction::L1Handler(L1HandlerTransaction {
                    contract_address,
                    entry_point_selector,
                    nonce,
                    calldata,
                    transaction_hash: _,
                    // This should always be zero.
                    version: _,
                }) => TransactionVariant::L1Handler(
                    pathfinder_common::transaction::L1HandlerTransaction {
                        contract_address,
                        entry_point_selector,
                        nonce,
                        calldata,
                    },
                ),
            };

            pathfinder_common::transaction::Transaction { hash, variant }
        }
    }

    impl Transaction {
        /// Returns hash of the transaction
        pub fn hash(&self) -> TransactionHash {
            match self {
                Transaction::Declare(t) => match t {
                    DeclareTransaction::V0(t) => t.transaction_hash,
                    DeclareTransaction::V1(t) => t.transaction_hash,
                    DeclareTransaction::V2(t) => t.transaction_hash,
                    DeclareTransaction::V3(t) => t.transaction_hash,
                },
                Transaction::Deploy(t) => t.transaction_hash,
                Transaction::DeployAccount(t) => match t {
                    DeployAccountTransaction::V0V1(t) => t.transaction_hash,
                    DeployAccountTransaction::V3(t) => t.transaction_hash,
                },
                Transaction::Invoke(t) => match t {
                    InvokeTransaction::V0(t) => t.transaction_hash,
                    InvokeTransaction::V1(t) => t.transaction_hash,
                    InvokeTransaction::V3(t) => t.transaction_hash,
                },
                Transaction::L1Handler(t) => t.transaction_hash,
            }
        }

        pub fn contract_address(&self) -> ContractAddress {
            match self {
                Transaction::Declare(DeclareTransaction::V0(t)) => t.sender_address,
                Transaction::Declare(DeclareTransaction::V1(t)) => t.sender_address,
                Transaction::Declare(DeclareTransaction::V2(t)) => t.sender_address,
                Transaction::Declare(DeclareTransaction::V3(t)) => t.sender_address,
                Transaction::Deploy(t) => t.contract_address,
                Transaction::DeployAccount(t) => match t {
                    DeployAccountTransaction::V0V1(t) => t.contract_address,
                    DeployAccountTransaction::V3(t) => t.sender_address,
                },
                Transaction::Invoke(t) => match t {
                    InvokeTransaction::V0(t) => t.sender_address,
                    InvokeTransaction::V1(t) => t.sender_address,
                    InvokeTransaction::V3(t) => t.sender_address,
                },
                Transaction::L1Handler(t) => t.contract_address,
            }
        }

        pub fn version(&self) -> TransactionVersion {
            match self {
                Transaction::Declare(DeclareTransaction::V0(_)) => TransactionVersion::ZERO,
                Transaction::Declare(DeclareTransaction::V1(_)) => TransactionVersion::ONE,
                Transaction::Declare(DeclareTransaction::V2(_)) => TransactionVersion::TWO,
                Transaction::Declare(DeclareTransaction::V3(_)) => TransactionVersion::THREE,

                Transaction::Deploy(t) => t.version,
                Transaction::DeployAccount(t) => match t {
                    DeployAccountTransaction::V0V1(t) => t.version,
                    DeployAccountTransaction::V3(t) => t.version,
                },
                Transaction::Invoke(InvokeTransaction::V0(_)) => TransactionVersion::ZERO,
                Transaction::Invoke(InvokeTransaction::V1(_)) => TransactionVersion::ONE,
                Transaction::Invoke(InvokeTransaction::V3(_)) => TransactionVersion::THREE,
                Transaction::L1Handler(t) => t.version,
            }
        }
    }

    #[derive(Clone, Debug, Serialize, PartialEq, Eq)]
    #[serde(tag = "version")]
    pub enum DeclareTransaction {
        #[serde(rename = "0x0")]
        V0(DeclareTransactionV0V1),
        #[serde(rename = "0x1")]
        V1(DeclareTransactionV0V1),
        #[serde(rename = "0x2")]
        V2(DeclareTransactionV2),
        #[serde(rename = "0x3")]
        V3(DeclareTransactionV3),
    }

    impl<'de> Deserialize<'de> for DeclareTransaction {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de;

            #[serde_as]
            #[derive(Deserialize)]
            struct Version {
                #[serde(default = "transaction_version_zero")]
                pub version: TransactionVersion,
            }

            let mut v = serde_json::Value::deserialize(deserializer)?;
            let version = Version::deserialize(&v).map_err(de::Error::custom)?;
            // remove "version", since v0 and v1 transactions use deny_unknown_fields
            v.as_object_mut()
                .expect("must be an object because deserializing version succeeded")
                .remove("version");
            match version.version {
                TransactionVersion::ZERO => Ok(Self::V0(
                    DeclareTransactionV0V1::deserialize(&v).map_err(de::Error::custom)?,
                )),
                TransactionVersion::ONE => Ok(Self::V1(
                    DeclareTransactionV0V1::deserialize(&v).map_err(de::Error::custom)?,
                )),
                TransactionVersion::TWO => Ok(Self::V2(
                    DeclareTransactionV2::deserialize(&v).map_err(de::Error::custom)?,
                )),
                TransactionVersion::THREE => Ok(Self::V3(
                    DeclareTransactionV3::deserialize(&v).map_err(de::Error::custom)?,
                )),
                _v => Err(de::Error::custom("version must be 0, 1, 2 or 3")),
            }
        }
    }

    impl DeclareTransaction {
        pub fn signature(&self) -> &[TransactionSignatureElem] {
            match self {
                DeclareTransaction::V0(tx) => tx.signature.as_ref(),
                DeclareTransaction::V1(tx) => tx.signature.as_ref(),
                DeclareTransaction::V2(tx) => tx.signature.as_ref(),
                DeclareTransaction::V3(tx) => tx.signature.as_ref(),
            }
        }
    }

    impl<T> Dummy<T> for DeclareTransaction {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            match rng.gen_range(0..=3) {
                0 => {
                    let mut v0: DeclareTransactionV0V1 = Faker.fake_with_rng(rng);
                    v0.nonce = TransactionNonce::ZERO;
                    Self::V0(v0)
                }
                1 => Self::V1(Faker.fake_with_rng(rng)),
                2 => Self::V2(Faker.fake_with_rng(rng)),
                3 => Self::V3(Faker.fake_with_rng(rng)),
                _ => unreachable!(),
            }
        }
    }

    /// A version 0 or 1 declare transaction.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    #[serde(deny_unknown_fields)]
    pub struct DeclareTransactionV0V1 {
        pub class_hash: ClassHash,
        pub max_fee: Fee,
        pub nonce: TransactionNonce,
        pub sender_address: ContractAddress,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        #[serde(default)]
        pub signature: Vec<TransactionSignatureElem>,
        pub transaction_hash: TransactionHash,
    }

    /// A version 2 declare transaction.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    #[serde(deny_unknown_fields)]
    pub struct DeclareTransactionV2 {
        pub class_hash: ClassHash,
        pub max_fee: Fee,
        pub nonce: TransactionNonce,
        pub sender_address: ContractAddress,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        #[serde(default)]
        pub signature: Vec<TransactionSignatureElem>,
        pub transaction_hash: TransactionHash,
        pub compiled_class_hash: CasmHash,
    }

    /// A version 2 declare transaction.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct DeclareTransactionV3 {
        pub class_hash: ClassHash,

        pub nonce: TransactionNonce,
        pub nonce_data_availability_mode: DataAvailabilityMode,
        pub fee_data_availability_mode: DataAvailabilityMode,
        pub resource_bounds: ResourceBounds,
        #[serde_as(as = "TipAsHexStr")]
        pub tip: Tip,
        pub paymaster_data: Vec<PaymasterDataElem>,

        pub sender_address: ContractAddress,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        #[serde(default)]
        pub signature: Vec<TransactionSignatureElem>,
        pub transaction_hash: TransactionHash,
        pub compiled_class_hash: CasmHash,

        pub account_deployment_data: Vec<AccountDeploymentDataElem>,
    }

    impl<T> Dummy<T> for DeclareTransactionV3 {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            Self {
                class_hash: Faker.fake_with_rng(rng),

                nonce: Faker.fake_with_rng(rng),
                nonce_data_availability_mode: Faker.fake_with_rng(rng),
                fee_data_availability_mode: Faker.fake_with_rng(rng),
                resource_bounds: Faker.fake_with_rng(rng),
                tip: Faker.fake_with_rng(rng),
                paymaster_data: vec![Faker.fake_with_rng(rng)], // TODO p2p allows 1 elem only

                sender_address: Faker.fake_with_rng(rng),
                signature: Faker.fake_with_rng(rng),
                transaction_hash: Faker.fake_with_rng(rng),
                compiled_class_hash: Faker.fake_with_rng(rng),
                account_deployment_data: vec![Faker.fake_with_rng(rng)], // TODO p2p allows 1 elem only
            }
        }
    }

    const fn transaction_version_zero() -> TransactionVersion {
        TransactionVersion::ZERO
    }

    /// Represents deserialized L2 deploy transaction data.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct DeployTransaction {
        pub contract_address: ContractAddress,
        pub contract_address_salt: ContractAddressSalt,
        pub class_hash: ClassHash,
        #[serde_as(as = "Vec<ConstructorParamAsDecimalStr>")]
        pub constructor_calldata: Vec<ConstructorParam>,
        pub transaction_hash: TransactionHash,
        #[serde(default = "transaction_version_zero")]
        pub version: TransactionVersion,
    }

    impl<T> Dummy<T> for DeployTransaction {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            Self {
                version: TransactionVersion(Felt::from_u64(rng.gen_range(0..=1))),
                contract_address: Faker.fake_with_rng(rng),
                contract_address_salt: Faker.fake_with_rng(rng),
                class_hash: Faker.fake_with_rng(rng),
                constructor_calldata: Faker.fake_with_rng(rng),
                transaction_hash: Faker.fake_with_rng(rng),
            }
        }
    }

    /// Represents deserialized L2 deploy account transaction data.
    #[derive(Clone, Debug, Serialize, PartialEq, Eq, Dummy)]
    #[serde(untagged)]
    pub enum DeployAccountTransaction {
        V0V1(DeployAccountTransactionV0V1),
        V3(DeployAccountTransactionV3),
    }

    impl<'de> Deserialize<'de> for DeployAccountTransaction {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de;

            #[serde_as]
            #[derive(Deserialize)]
            struct Version {
                #[serde(default = "transaction_version_zero")]
                pub version: TransactionVersion,
            }

            let v = serde_json::Value::deserialize(deserializer)?;
            let version = Version::deserialize(&v).map_err(de::Error::custom)?;

            match version.version {
                TransactionVersion::ZERO => Ok(Self::V0V1(
                    DeployAccountTransactionV0V1::deserialize(&v).map_err(de::Error::custom)?,
                )),
                TransactionVersion::ONE => Ok(Self::V0V1(
                    DeployAccountTransactionV0V1::deserialize(&v).map_err(de::Error::custom)?,
                )),
                TransactionVersion::THREE => Ok(Self::V3(
                    DeployAccountTransactionV3::deserialize(&v).map_err(de::Error::custom)?,
                )),
                _v => Err(de::Error::custom("version must be 0, 1 or 3")),
            }
        }
    }

    impl DeployAccountTransaction {
        pub fn contract_address(&self) -> ContractAddress {
            match self {
                Self::V0V1(tx) => tx.contract_address,
                Self::V3(tx) => tx.sender_address,
            }
        }

        pub fn signature(&self) -> &[TransactionSignatureElem] {
            match self {
                Self::V0V1(tx) => tx.signature.as_ref(),
                Self::V3(tx) => tx.signature.as_ref(),
            }
        }
    }

    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct DeployAccountTransactionV0V1 {
        pub contract_address: ContractAddress,
        pub transaction_hash: TransactionHash,
        pub max_fee: Fee,
        pub version: TransactionVersion,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        pub signature: Vec<TransactionSignatureElem>,
        pub nonce: TransactionNonce,
        pub contract_address_salt: ContractAddressSalt,
        #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
        pub constructor_calldata: Vec<CallParam>,
        pub class_hash: ClassHash,
    }

    impl<T> Dummy<T> for DeployAccountTransactionV0V1 {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            let contract_address_salt = Faker.fake_with_rng(rng);
            let constructor_calldata: Vec<CallParam> = Faker.fake_with_rng(rng);
            let class_hash = Faker.fake_with_rng(rng);

            Self {
                version: TransactionVersion::ONE,
                contract_address: ContractAddress::deployed_contract_address(
                    constructor_calldata.iter().copied(),
                    &contract_address_salt,
                    &class_hash,
                ),
                transaction_hash: Faker.fake_with_rng(rng),
                max_fee: Faker.fake_with_rng(rng),
                signature: Faker.fake_with_rng(rng),
                nonce: Faker.fake_with_rng(rng),
                contract_address_salt,
                constructor_calldata,
                class_hash,
            }
        }
    }

    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct DeployAccountTransactionV3 {
        pub nonce: TransactionNonce,
        pub nonce_data_availability_mode: DataAvailabilityMode,
        pub fee_data_availability_mode: DataAvailabilityMode,
        pub resource_bounds: ResourceBounds,
        #[serde_as(as = "TipAsHexStr")]
        pub tip: Tip,
        pub paymaster_data: Vec<PaymasterDataElem>,

        pub sender_address: ContractAddress,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        pub signature: Vec<TransactionSignatureElem>,
        pub transaction_hash: TransactionHash,
        pub version: TransactionVersion,
        pub contract_address_salt: ContractAddressSalt,
        #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
        pub constructor_calldata: Vec<CallParam>,
        pub class_hash: ClassHash,
    }

    impl<T> Dummy<T> for DeployAccountTransactionV3 {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            let contract_address_salt = Faker.fake_with_rng(rng);
            let constructor_calldata: Vec<CallParam> = Faker.fake_with_rng(rng);
            let class_hash = Faker.fake_with_rng(rng);

            Self {
                nonce: Faker.fake_with_rng(rng),
                nonce_data_availability_mode: Faker.fake_with_rng(rng),
                fee_data_availability_mode: Faker.fake_with_rng(rng),
                resource_bounds: Faker.fake_with_rng(rng),
                tip: Faker.fake_with_rng(rng),
                paymaster_data: vec![Faker.fake_with_rng(rng)], // TODO p2p allows 1 elem only

                sender_address: ContractAddress::deployed_contract_address(
                    constructor_calldata.iter().copied(),
                    &contract_address_salt,
                    &class_hash,
                ),
                signature: Faker.fake_with_rng(rng),
                transaction_hash: Faker.fake_with_rng(rng),
                version: TransactionVersion::THREE,
                contract_address_salt,
                constructor_calldata,
                class_hash,
            }
        }
    }

    #[derive(Clone, Debug, Serialize, PartialEq, Eq, Dummy)]
    #[serde(tag = "version")]
    pub enum InvokeTransaction {
        #[serde(rename = "0x0")]
        V0(InvokeTransactionV0),
        #[serde(rename = "0x1")]
        V1(InvokeTransactionV1),
        #[serde(rename = "0x3")]
        V3(InvokeTransactionV3),
    }

    impl<'de> Deserialize<'de> for InvokeTransaction {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de;

            #[serde_as]
            #[derive(Deserialize)]
            struct Version {
                #[serde(default = "transaction_version_zero")]
                pub version: TransactionVersion,
            }

            let mut v = serde_json::Value::deserialize(deserializer)?;
            let version = Version::deserialize(&v).map_err(de::Error::custom)?;
            // remove "version", since v0 and v1 transactions use deny_unknown_fields
            v.as_object_mut()
                .expect("must be an object because deserializing version succeeded")
                .remove("version");
            match version.version {
                TransactionVersion::ZERO => Ok(Self::V0(
                    InvokeTransactionV0::deserialize(&v).map_err(de::Error::custom)?,
                )),
                TransactionVersion::ONE => Ok(Self::V1(
                    InvokeTransactionV1::deserialize(&v).map_err(de::Error::custom)?,
                )),
                TransactionVersion::THREE => Ok(Self::V3(
                    InvokeTransactionV3::deserialize(&v).map_err(de::Error::custom)?,
                )),
                _v => Err(de::Error::custom("version must be 0, 1 or 3")),
            }
        }
    }

    impl InvokeTransaction {
        pub fn signature(&self) -> &[TransactionSignatureElem] {
            match self {
                Self::V0(tx) => tx.signature.as_ref(),
                Self::V1(tx) => tx.signature.as_ref(),
                Self::V3(tx) => tx.signature.as_ref(),
            }
        }
    }

    /// Represents deserialized L2 invoke transaction v0 data.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct InvokeTransactionV0 {
        #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
        pub calldata: Vec<CallParam>,
        // contract_address is the historic name for this field. sender_address was
        // introduced with starknet v0.11. Although the gateway no longer uses the historic
        // name at all, this alias must be kept until a database migration fixes all historic
        // transaction naming, or until regenesis removes them all.
        #[serde(alias = "contract_address")]
        pub sender_address: ContractAddress,
        pub entry_point_selector: EntryPoint,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub entry_point_type: Option<EntryPointType>,
        pub max_fee: Fee,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        pub signature: Vec<TransactionSignatureElem>,
        pub transaction_hash: TransactionHash,
    }

    impl<T> Dummy<T> for InvokeTransactionV0 {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            Self {
                calldata: Faker.fake_with_rng(rng),
                sender_address: Faker.fake_with_rng(rng),
                entry_point_selector: Faker.fake_with_rng(rng),
                entry_point_type: None,
                max_fee: Faker.fake_with_rng(rng),
                signature: Faker.fake_with_rng(rng),
                transaction_hash: Faker.fake_with_rng(rng),
            }
        }
    }

    /// Represents deserialized L2 invoke transaction v1 data.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Dummy)]
    #[serde(deny_unknown_fields)]
    pub struct InvokeTransactionV1 {
        #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
        pub calldata: Vec<CallParam>,
        // contract_address is the historic name for this field. sender_address was
        // introduced with starknet v0.11. Although the gateway no longer uses the historic
        // name at all, this alias must be kept until a database migration fixes all historic
        // transaction naming, or until regenesis removes them all.
        #[serde(alias = "contract_address")]
        pub sender_address: ContractAddress,
        pub max_fee: Fee,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        pub signature: Vec<TransactionSignatureElem>,
        pub nonce: TransactionNonce,
        pub transaction_hash: TransactionHash,
    }

    /// Represents deserialized L2 invoke transaction v3 data.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct InvokeTransactionV3 {
        pub nonce: TransactionNonce,
        pub nonce_data_availability_mode: DataAvailabilityMode,
        pub fee_data_availability_mode: DataAvailabilityMode,
        pub resource_bounds: ResourceBounds,
        #[serde_as(as = "TipAsHexStr")]
        pub tip: Tip,
        pub paymaster_data: Vec<PaymasterDataElem>,

        pub sender_address: ContractAddress,
        #[serde_as(as = "Vec<TransactionSignatureElemAsDecimalStr>")]
        pub signature: Vec<TransactionSignatureElem>,
        pub transaction_hash: TransactionHash,
        #[serde_as(as = "Vec<CallParamAsDecimalStr>")]
        pub calldata: Vec<CallParam>,

        pub account_deployment_data: Vec<AccountDeploymentDataElem>,
    }

    impl<T> Dummy<T> for InvokeTransactionV3 {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            Self {
                nonce: Faker.fake_with_rng(rng),
                nonce_data_availability_mode: Faker.fake_with_rng(rng),
                fee_data_availability_mode: Faker.fake_with_rng(rng),
                resource_bounds: Faker.fake_with_rng(rng),
                tip: Faker.fake_with_rng(rng),
                paymaster_data: vec![Faker.fake_with_rng(rng)], // TODO p2p allows 1 elem only

                sender_address: Faker.fake_with_rng(rng),
                signature: Faker.fake_with_rng(rng),
                transaction_hash: Faker.fake_with_rng(rng),
                calldata: Faker.fake_with_rng(rng),
                account_deployment_data: vec![Faker.fake_with_rng(rng)], // TODO p2p allows 1 elem only
            }
        }
    }

    /// Represents deserialized L2 "L1 handler" transaction data.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct L1HandlerTransaction {
        pub contract_address: ContractAddress,
        pub entry_point_selector: EntryPoint,
        // FIXME: remove once starkware fixes their gateway bug which was missing this field.
        #[serde(default)]
        pub nonce: TransactionNonce,
        pub calldata: Vec<CallParam>,
        pub transaction_hash: TransactionHash,
        pub version: TransactionVersion,
    }

    impl L1HandlerTransaction {
        pub fn calculate_message_hash(&self) -> H256 {
            use sha3::{Digest, Keccak256};

            let Some((from_address, payload)) = self.calldata.split_first() else {
                // This would indicate a pretty severe error in the L1 transaction.
                // But since we haven't encoded this during serialization, this could in
                // theory mess us up here.
                //
                // We should incorporate this into the deserialization instead. Returning an
                // error here is unergonomic and far too late.
                return H256::zero();
            };

            let mut hash = Keccak256::new();

            // This is an ethereum address
            hash.update(from_address.0.as_be_bytes());
            hash.update(self.contract_address.0.as_be_bytes());
            hash.update(self.nonce.0.as_be_bytes());
            hash.update(self.entry_point_selector.0.as_be_bytes());

            // Pad the u64 to 32 bytes to match a felt.
            hash.update([0u8; 24]);
            hash.update((payload.len() as u64).to_be_bytes());

            for elem in payload {
                hash.update(elem.0.as_be_bytes());
            }

            let hash = <[u8; 32]>::from(hash.finalize());

            hash.into()
        }
    }

    impl<T> Dummy<T> for L1HandlerTransaction {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &T, rng: &mut R) -> Self {
            Self {
                // TODO verify this is the only realistic value
                version: TransactionVersion::ZERO,

                contract_address: Faker.fake_with_rng(rng),
                entry_point_selector: Faker.fake_with_rng(rng),
                nonce: Faker.fake_with_rng(rng),
                calldata: Faker.fake_with_rng(rng),
                transaction_hash: Faker.fake_with_rng(rng),
            }
        }
    }

    impl From<DeclareTransaction> for Transaction {
        fn from(tx: DeclareTransaction) -> Self {
            Self::Declare(tx)
        }
    }

    impl From<DeployTransaction> for Transaction {
        fn from(tx: DeployTransaction) -> Self {
            Self::Deploy(tx)
        }
    }

    impl From<InvokeTransaction> for Transaction {
        fn from(tx: InvokeTransaction) -> Self {
            Self::Invoke(tx)
        }
    }

    impl From<L1HandlerTransaction> for Transaction {
        fn from(tx: L1HandlerTransaction) -> Self {
            Self::L1Handler(tx)
        }
    }

    impl From<InvokeTransactionV0> for InvokeTransaction {
        fn from(tx: InvokeTransactionV0) -> Self {
            Self::V0(tx)
        }
    }

    impl From<InvokeTransactionV1> for InvokeTransaction {
        fn from(tx: InvokeTransactionV1) -> Self {
            Self::V1(tx)
        }
    }

    /// Describes L2 transaction failure details.
    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct Failure {
        pub code: String,
        pub error_message: String,
    }
}

/// Used to deserialize replies to StarkNet state update requests.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StateUpdate {
    /// Gets default value for pending state updates.
    #[serde(default)]
    pub block_hash: BlockHash,
    /// Gets default value for pending state updates.
    #[serde(default)]
    pub new_root: StateCommitment,
    pub old_root: StateCommitment,
    pub state_diff: state_update::StateDiff,
}

impl From<StateUpdate> for pathfinder_common::StateUpdate {
    fn from(mut gateway: StateUpdate) -> Self {
        let mut state_update = pathfinder_common::StateUpdate::default()
            .with_block_hash(gateway.block_hash)
            .with_parent_state_commitment(gateway.old_root)
            .with_state_commitment(gateway.new_root);

        // Extract the known system contract updates from the normal contract updates.
        // This must occur before we map the contract updates, since we want to first remove
        // the system contract updates.
        //
        // Currently this is only the contract at address 0x1.
        //
        // As of starknet v0.12.0 these are embedded in this way, but in the future will be
        // a separate property in the state diff.
        if let Some((address, storage_updates)) = gateway
            .state_diff
            .storage_diffs
            .remove_entry(&ContractAddress::ONE)
        {
            for state_update::StorageDiff { key, value } in storage_updates {
                state_update = state_update.with_system_storage_update(address, key, value);
            }
        }

        // Aggregate contract deployments, storage, nonce and class replacements into contract updates.
        for (address, storage_updates) in gateway.state_diff.storage_diffs {
            for state_update::StorageDiff { key, value } in storage_updates {
                state_update = state_update.with_storage_update(address, key, value);
            }
        }

        for state_update::DeployedContract {
            address,
            class_hash,
        } in gateway.state_diff.deployed_contracts
        {
            state_update = state_update.with_deployed_contract(address, class_hash);
        }

        for (address, nonce) in gateway.state_diff.nonces {
            state_update = state_update.with_contract_nonce(address, nonce);
        }

        for state_update::ReplacedClass {
            address,
            class_hash,
        } in gateway.state_diff.replaced_classes
        {
            state_update = state_update.with_replaced_class(address, class_hash);
        }

        for state_update::DeclaredSierraClass {
            class_hash,
            compiled_class_hash,
        } in gateway.state_diff.declared_classes
        {
            state_update = state_update.with_declared_sierra_class(class_hash, compiled_class_hash);
        }

        state_update.declared_cairo_classes = gateway.state_diff.old_declared_contracts;

        state_update
    }
}

/// Types used when deserializing state update related data.
pub mod state_update {
    use pathfinder_common::{
        CasmHash, ClassHash, ContractAddress, ContractNonce, SierraHash, StorageAddress,
        StorageValue,
    };
    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;
    use std::collections::{HashMap, HashSet};

    /// L2 state diff.
    #[serde_as]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
    #[serde(deny_unknown_fields)]
    pub struct StateDiff {
        #[serde_as(as = "HashMap<_, Vec<_>>")]
        pub storage_diffs: HashMap<ContractAddress, Vec<StorageDiff>>,
        pub deployed_contracts: Vec<DeployedContract>,
        pub old_declared_contracts: HashSet<ClassHash>,
        pub declared_classes: Vec<DeclaredSierraClass>,
        pub nonces: HashMap<ContractAddress, ContractNonce>,
        pub replaced_classes: Vec<ReplacedClass>,
    }

    /// L2 storage diff.
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct StorageDiff {
        pub key: StorageAddress,
        pub value: StorageValue,
    }

    /// L2 contract data within state diff.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct DeployedContract {
        pub address: ContractAddress,
        /// `class_hash` is the field name from cairo 0.9.0 onwards
        /// `contract_hash` is the name from cairo before 0.9.0
        #[serde(alias = "contract_hash")]
        pub class_hash: ClassHash,
    }

    /// Describes a newly declared class. Maps Sierra class hash to a Casm hash.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct DeclaredSierraClass {
        pub class_hash: SierraHash,
        pub compiled_class_hash: CasmHash,
    }

    /// Describes a newly replaced class. Maps contract address to a new class.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(deny_unknown_fields)]
    pub struct ReplacedClass {
        pub address: ContractAddress,
        pub class_hash: ClassHash,
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn contract_field_backward_compatibility() {
            use super::DeployedContract;

            use pathfinder_common::macro_prelude::*;

            let expected = DeployedContract {
                address: contract_address!("0x1"),
                class_hash: class_hash!("0x2"),
            };

            // cario <0.9.0
            assert_eq!(
                serde_json::from_str::<DeployedContract>(
                    r#"{"address":"0x01","contract_hash":"0x02"}"#
                )
                .unwrap(),
                expected
            );
            // cario >=0.9.0
            assert_eq!(
                serde_json::from_str::<DeployedContract>(
                    r#"{"address":"0x01","class_hash":"0x02"}"#
                )
                .unwrap(),
                expected
            );
        }
    }
}

/// Used to deserialize replies to Starknet Ethereum contract requests.
#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct EthContractAddresses {
    #[serde(rename = "Starknet")]
    #[serde_as(as = "EthereumAddressAsHexStr")]
    pub starknet: EthereumAddress,
}

pub mod add_transaction {
    use pathfinder_common::{ClassHash, ContractAddress, TransactionHash};

    /// API response for an INVOKE_FUNCTION transaction
    #[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct InvokeResponse {
        pub code: String, // TRANSACTION_RECEIVED
        pub transaction_hash: TransactionHash,
    }

    /// API response for a DECLARE transaction
    #[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct DeclareResponse {
        pub code: String, // TRANSACTION_RECEIVED
        pub transaction_hash: TransactionHash,
        pub class_hash: ClassHash,
    }

    /// API response for a DEPLOY transaction
    #[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct DeployResponse {
        pub code: String, // TRANSACTION_RECEIVED
        pub transaction_hash: TransactionHash,
        pub address: ContractAddress,
    }

    /// API response for a DEPLOY ACCOUNT transaction
    #[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
    pub struct DeployAccountResponse {
        pub code: String, // TRANSACTION_RECEIVED
        pub transaction_hash: TransactionHash,
    }

    #[cfg(test)]
    mod serde_test {
        use super::*;

        use pathfinder_common::macro_prelude::*;

        #[test]
        fn test_invoke_response() {
            let result = serde_json::from_str::<InvokeResponse>(r#"{"code": "TRANSACTION_RECEIVED", "transaction_hash": "0x389dd0629f42176cc8b6c43acefc0713d0064ecdfc0470e0fc179f53421a38b"}"#).unwrap();
            let expected = InvokeResponse {
                code: "TRANSACTION_RECEIVED".to_owned(),
                transaction_hash: transaction_hash!(
                    "0389dd0629f42176cc8b6c43acefc0713d0064ecdfc0470e0fc179f53421a38b"
                ),
            };
            assert_eq!(expected, result);
        }

        #[test]
        fn test_deploy_response() {
            let result = serde_json::from_str::<DeployResponse>(r#"{"code": "TRANSACTION_RECEIVED", "transaction_hash": "0x296fb89b8a1c7487a1d4b27e1a1e33f440b05548e64980d06052bc089b1a51f", "address": "0x677bb1cdc050e8d63855e8743ab6e09179138def390676cc03c484daf112ba1"}"#).unwrap();
            let expected = DeployResponse {
                code: "TRANSACTION_RECEIVED".to_owned(),
                transaction_hash: transaction_hash!(
                    "0296fb89b8a1c7487a1d4b27e1a1e33f440b05548e64980d06052bc089b1a51f"
                ),
                address: contract_address!(
                    "0677bb1cdc050e8d63855e8743ab6e09179138def390676cc03c484daf112ba1"
                ),
            };
            assert_eq!(expected, result);
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, serde::Serialize)]
pub struct BlockSignature {
    pub block_number: BlockNumber,
    pub signature: [BlockCommitmentSignatureElem; 2],
    pub signature_input: BlockSignatureInput,
}

#[derive(Clone, Debug, Deserialize, PartialEq, serde::Serialize)]
pub struct BlockSignatureInput {
    pub block_hash: BlockHash,
    pub state_diff_commitment: StateDiffCommitment,
}

impl From<BlockSignature> for pathfinder_common::BlockCommitmentSignature {
    fn from(value: BlockSignature) -> Self {
        Self {
            r: value.signature[0],
            s: value.signature[1],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::str::FromStr;

    use primitive_types::H256;

    use crate::reply::state_update::{
        DeclaredSierraClass, DeployedContract, ReplacedClass, StorageDiff,
    };
    use crate::reply::transaction::L1HandlerTransaction;

    /// The aim of these tests is to make sure pathfinder is still able to correctly
    /// deserialize replies from the mainnet sequencer when it still is using some
    /// previous version of cairo while at the same time the goerli sequencer is
    /// already using a newer version.
    mod backward_compatibility {
        use super::super::StateUpdate;
        use starknet_gateway_test_fixtures::*;

        #[test]
        fn block() {
            use super::super::{Block, PendingBlock};

            // Mainnet block 192 contains an L1_HANDLER transaction without a nonce.
            serde_json::from_str::<Block>(old::block::NUMBER_192).unwrap();
            serde_json::from_str::<Block>(v0_8_2::block::GENESIS).unwrap();
            serde_json::from_str::<Block>(v0_8_2::block::NUMBER_1716).unwrap();
            serde_json::from_str::<PendingBlock>(v0_8_2::block::PENDING).unwrap();
            // This is from integration starknet_version 0.10 and contains the new version 1 invoke transaction.
            serde_json::from_str::<Block>(integration::block::NUMBER_216591).unwrap();
            // This is from integration starknet_version 0.10.0 and contains the new L1 handler transaction.
            serde_json::from_str::<Block>(integration::block::NUMBER_216171).unwrap();
            // This is from integration starknet_version 0.10.1 and contains the new deploy account transaction.
            serde_json::from_str::<Block>(integration::block::NUMBER_228457).unwrap();
            // This is from integration starknet_version 0.13.0 and contains new v3 invoke and deploy account transactions.
            serde_json::from_str::<Block>(integration::block::NUMBER_319693).unwrap();
            // This is from integration starknet_version 0.13.0 and contains a new v3 declare transaction.
            serde_json::from_str::<Block>(integration::block::NUMBER_319709).unwrap();
            serde_json::from_str::<PendingBlock>(v0_13_0::block::PENDING).unwrap();
            // This is from integration starknet_version 0.13.0 and contains data gas prices.
            serde_json::from_str::<Block>(integration::block::NUMBER_329543).unwrap();
            serde_json::from_str::<PendingBlock>(v0_13_1::block::PENDING).unwrap();
        }

        #[test]
        fn state_update() {
            // This is from integration starknet_version 0.11 and contains the new declared_classes field.
            serde_json::from_str::<StateUpdate>(integration::state_update::NUMBER_283364).unwrap();
            // This is from integration starknet_version 0.11 and contains the new replaced_classes field.
            serde_json::from_str::<StateUpdate>(integration::state_update::NUMBER_283428).unwrap();
        }

        #[test]
        fn legacy_l1_handler_is_invoke() {
            // In the times before L1 Handler became an official tx variant,
            // these were instead served as Invoke V0 txs. This test ensures
            // that we correctly map these historic txs to L1 Handler.
            use super::super::transaction::Transaction as TransactionVariant;

            let json = serde_json::json!({
                "type":"INVOKE_FUNCTION",
                "calldata":[
                    "580042449035822898911647251144793933582335302582",
                    "3241583063705060367416058138609427972824194056099997457116843686898315086623",
                    "2000000000000000000",
                    "0",
                    "725188533692944996190142472767755401716439215485"
                ],
                "contract_address":"0x1108cdbe5d82737b9057590adaf97d34e74b5452f0628161d237746b6fe69e",
                "entry_point_selector":"0x2d757788a8d8d6f21d1cd40bce38a8222d70654214e96ff95d8086e684fbee5",
                "entry_point_type":"L1_HANDLER",
                "max_fee":"0x0",
                "signature":[],
                "transaction_hash":"0x70cad5b0d09ff2b252d3bf040708a89e6f175715f5f550e8d8161fabef01261"
            });

            let tx: TransactionVariant = serde_json::from_value(json).unwrap();

            assert_matches::assert_matches!(tx, TransactionVariant::L1Handler(_));
        }
    }

    #[test]
    fn from_state_update() {
        use pathfinder_common::macro_prelude::*;

        let expected = pathfinder_common::StateUpdate::default()
            .with_block_hash(block_hash_bytes!(b"block hash"))
            .with_state_commitment(state_commitment_bytes!(b"state commitment"))
            .with_parent_state_commitment(state_commitment_bytes!(b"parent commitment"))
            .with_storage_update(
                contract_address_bytes!(b"contract 0"),
                storage_address_bytes!(b"storage key 0"),
                storage_value_bytes!(b"storage val 0"),
            )
            .with_deployed_contract(
                contract_address_bytes!(b"deployed contract"),
                class_hash_bytes!(b"deployed class"),
            )
            .with_declared_cairo_class(class_hash_bytes!(b"cairo 0 0"))
            .with_declared_cairo_class(class_hash_bytes!(b"cairo 0 1"))
            .with_declared_sierra_class(
                sierra_hash_bytes!(b"sierra class"),
                casm_hash_bytes!(b"casm hash"),
            )
            .with_contract_nonce(
                contract_address_bytes!(b"contract 0"),
                contract_nonce_bytes!(b"nonce 0"),
            )
            .with_contract_nonce(
                contract_address_bytes!(b"contract 10"),
                contract_nonce_bytes!(b"nonce 10"),
            )
            .with_replaced_class(
                contract_address_bytes!(b"contract 0"),
                class_hash_bytes!(b"replaced class"),
            );

        let gateway = super::StateUpdate {
            block_hash: block_hash_bytes!(b"block hash"),
            new_root: state_commitment_bytes!(b"state commitment"),
            old_root: state_commitment_bytes!(b"parent commitment"),
            state_diff: super::state_update::StateDiff {
                storage_diffs: HashMap::from([(
                    contract_address_bytes!(b"contract 0"),
                    vec![StorageDiff {
                        key: storage_address_bytes!(b"storage key 0"),
                        value: storage_value_bytes!(b"storage val 0"),
                    }],
                )]),
                deployed_contracts: vec![DeployedContract {
                    address: contract_address_bytes!(b"deployed contract"),
                    class_hash: class_hash_bytes!(b"deployed class"),
                }],
                old_declared_contracts: HashSet::from([
                    class_hash_bytes!(b"cairo 0 0"),
                    class_hash_bytes!(b"cairo 0 1"),
                ]),
                declared_classes: vec![DeclaredSierraClass {
                    class_hash: sierra_hash_bytes!(b"sierra class"),
                    compiled_class_hash: casm_hash_bytes!(b"casm hash"),
                }],
                nonces: HashMap::from([
                    (
                        contract_address_bytes!(b"contract 0"),
                        contract_nonce_bytes!(b"nonce 0"),
                    ),
                    (
                        contract_address_bytes!(b"contract 10"),
                        contract_nonce_bytes!(b"nonce 10"),
                    ),
                ]),
                replaced_classes: vec![ReplacedClass {
                    address: contract_address_bytes!(b"contract 0"),
                    class_hash: class_hash_bytes!(b"replaced class"),
                }],
            },
        };

        let common = pathfinder_common::StateUpdate::from(gateway);

        assert_eq!(common, expected);
    }

    mod receipts {
        use crate::reply::transaction::{ExecutionStatus, Receipt};

        #[test]
        fn without_execution_status() {
            // Execution status was introduced in v0.12.1. Receipts from before this time could not revert
            // and should therefore always succeed. Receipt below taken from testnet v0.12.0.
            let json = r#"{
                "transaction_index": 0,
                "transaction_hash": "0xff4820a0ae5859fa2f75606effcb5caab34c01f7aecb413c2bd7dc724d603",
                "l2_to_l1_messages": [],
                "events": [{
                    "from_address": "0x783a9097b26eae0586373b2ce0ed3529ddc44069d1e0fbc4f66d42b69d6850d",
                    "keys": ["0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9"],
                    "data": [
                        "0x0",
                        "0x192688d37fe07a79213990c7bc7d3ca092541db3d9bcba3d7462fb3bfb4265f",
                        "0x3ecb5eb3ee",
                        "0x0"
                    ]
                }]
            }"#;

            let receipt = serde_json::from_str::<Receipt>(json).unwrap();

            assert_eq!(receipt.execution_status, ExecutionStatus::Succeeded);
        }

        #[test]
        fn succeeded() {
            // Taken from integration v0.12.1.
            let json = r#"{
                "execution_status": "SUCCEEDED",
                "transaction_index": 0,
                "transaction_hash": "0x5c01146ca14316ceb337df39653d8cba17593c19aecfa56b7b40005749e159b",
                "l2_to_l1_messages": [],
                "events": [],
                "execution_resources": {
                    "n_steps": 318,
                    "builtin_instance_counter": {
                        "bitwise_builtin": 2,
                        "range_check_builtin": 8,
                        "pedersen_builtin": 2
                    },
                    "n_memory_holes": 25
                },
                "actual_fee": "0x59e58f1d1a0"
            }"#;

            let receipt = serde_json::from_str::<Receipt>(json).unwrap();

            assert_eq!(receipt.execution_status, ExecutionStatus::Succeeded);
        }

        #[test]
        fn reverted() {
            // Taken from integration v0.12.1 (revert_error was changed to shorten it)
            let json = r#"{
                "revert_error": "reason goes here",
                "execution_status": "REVERTED",
                "transaction_index": 1,
                "transaction_hash": "0x19abec18bbacec23c2eee160c70190a48e4b41dd5ff98ad8f247f9393559998",
                "l2_to_l1_messages": [],
                "events": [],
                "actual_fee": "0x247aff6e224"
            }"#;

            let receipt = serde_json::from_str::<Receipt>(json).unwrap();

            assert_eq!(receipt.execution_status, ExecutionStatus::Reverted);
            assert_eq!(receipt.revert_error, Some("reason goes here".to_owned()));
        }
    }

    #[test]
    fn eth_contract_addresses_ignores_extra_fields() {
        // Some gateway mocks include extra addresses, check that we can still parse these.
        let json = serde_json::json!({
            "Starknet": "0x12345abcd",
            "GpsStatementVerifier": "0xaabdde",
            "MemoryPageFactRegistry": "0xdeadbeef"
        });

        serde_json::from_value::<crate::reply::EthContractAddresses>(json).unwrap();
    }

    #[test]
    fn l1_handler_message_hash() {
        // Transaction taken from mainnet.
        let json = serde_json::json!({
            "transaction_hash": "0x63f36452a4255a9d3f06def95a08bbc295f0de0515adefbf04ee795ed4c3f12",
            "version": "0x0",
            "contract_address": "0x73314940630fd6dcda0d772d4c972c4e0a9946bef9dabf4ef84eda8ef542b82",
            "entry_point_selector": "0x2d757788a8d8d6f21d1cd40bce38a8222d70654214e96ff95d8086e684fbee5",
            "nonce": "0x17824b",
            "calldata": [
                "0xae0ee0a63a2ce6baeeffe56e7714fb4efe48d419",
                "0x2c63ec1313901744d1321b93bda51418cc18998a1562d368960711367f7530f",
                "0x11e14e1039c000",
                "0x0"
            ],
        });

        let l1_handler = serde_json::from_value::<L1HandlerTransaction>(json).unwrap();

        let message_hash = l1_handler.calculate_message_hash();

        // Taken from starkscan: https://starkscan.co/tx/0x063f36452a4255a9d3f06def95a08bbc295f0de0515adefbf04ee795ed4c3f12
        let expected =
            H256::from_str("573aeff3cf703775e8a76a27adee9e80f2ce558a6a38ec87e0249a8b175e5c1a")
                .unwrap();

        assert_eq!(message_hash, expected);
    }

    mod block_signature {
        use pathfinder_common::{
            block_commitment_signature_elem, block_hash, state_diff_commitment, BlockNumber,
        };

        use super::super::{BlockSignature, BlockSignatureInput, StateUpdate};

        #[test]
        fn parse() {
            let json = starknet_gateway_test_fixtures::v0_12_2::signature::BLOCK_350000;

            let expected = BlockSignature {
                block_number: BlockNumber::new_or_panic(350000),
                signature: [
                    block_commitment_signature_elem!(
                        "0x95e98f5b91d39ae2b1bf77447a4fc01725352ae8b0b2c0a3fe09d43d1d9e57"
                    ),
                    block_commitment_signature_elem!(
                        "0x541b2db8dae6d5ae24b34e427d251edc2e94dcffddd85f207e1b51f2f4bb1ef"
                    ),
                ],
                signature_input: BlockSignatureInput {
                    block_hash: block_hash!(
                        "0x6f7342a680d7f99bdfdd859f587c75299e7ffabe62c071ded3a6d8a34cb132c"
                    ),
                    state_diff_commitment: state_diff_commitment!(
                        "0x432e8e2ad833548e1c1077fc298991b055ba1e6f7a17dd332db98f4f428c56c"
                    ),
                },
            };

            let signature: BlockSignature = serde_json::from_str(json).unwrap();

            assert_eq!(signature, expected);
        }

        #[test]
        fn state_diff_commitment() {
            let signature_json = starknet_gateway_test_fixtures::v0_12_2::signature::BLOCK_350000;
            let signature: BlockSignature = serde_json::from_str(signature_json).unwrap();

            let state_update_json =
                starknet_gateway_test_fixtures::v0_12_2::state_update::BLOCK_350000;
            let state_update: StateUpdate = serde_json::from_str(state_update_json).unwrap();

            let state_update = pathfinder_common::StateUpdate::from(state_update);

            assert_eq!(
                state_update.compute_state_diff_commitment(),
                signature.signature_input.state_diff_commitment
            )
        }
    }
}
