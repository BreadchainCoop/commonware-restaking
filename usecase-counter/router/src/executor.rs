use alloy::{
    network::Ethereum,
    primitives::{Bytes, FixedBytes},
};
use anyhow::Result;
use async_trait::async_trait;
use commonware_avs_bindings::WalletProvider;
use commonware_avs_bindings::blssigcheckoperatorstateretriever::BLSSigCheckOperatorStateRetriever::getNonSignerStakesAndSignatureReturn;
use commonware_avs_bindings::counter::{self, Counter};
use commonware_avs_router::executor::ExecutionResult;
use commonware_avs_router::executor::bls::{
    BlsSignatureVerificationHandler, convert_non_signer_data,
};
use commonware_usecase_counter::types::CounterTaskData;

pub struct CounterHandler {
    pub counter: Counter::CounterInstance<WalletProvider, Ethereum>,
}

impl CounterHandler {
    pub fn new(counter: Counter::CounterInstance<WalletProvider, Ethereum>) -> Self {
        Self { counter }
    }
}

#[async_trait]
impl BlsSignatureVerificationHandler for CounterHandler {
    type TaskData = CounterTaskData;
    async fn handle_verification(
        &mut self,
        msg_hash: FixedBytes<32>,
        quorum_numbers: Bytes,
        current_block_number: u32,
        non_signer_data: getNonSignerStakesAndSignatureReturn,
        _task_data: Option<&Self::TaskData>,
    ) -> Result<ExecutionResult> {
        let converted_data = convert_non_signer_data(non_signer_data);
        let non_signer_struct_data =
            counter::IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
                nonSignerQuorumBitmapIndices: converted_data.nonSignerQuorumBitmapIndices,
                nonSignerPubkeys: converted_data
                    .nonSignerPubkeys
                    .into_iter()
                    .map(|p| counter::BN254::G1Point { X: p.X, Y: p.Y })
                    .collect(),
                quorumApks: converted_data
                    .quorumApks
                    .into_iter()
                    .map(|p| counter::BN254::G1Point { X: p.X, Y: p.Y })
                    .collect(),
                apkG2: counter::BN254::G2Point {
                    X: converted_data.apkG2.X,
                    Y: converted_data.apkG2.Y,
                },
                sigma: counter::BN254::G1Point {
                    X: converted_data.sigma.X,
                    Y: converted_data.sigma.Y,
                },
                quorumApkIndices: converted_data.quorumApkIndices,
                totalStakeIndices: converted_data.totalStakeIndices,
                nonSignerStakeIndices: converted_data.nonSignerStakeIndices,
            };

        let call_return = self
            .counter
            .increment(
                msg_hash,
                quorum_numbers,
                current_block_number,
                non_signer_struct_data,
            )
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send increment transaction: {}", e))?;

        let receipt = call_return
            .get_receipt()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get transaction receipt: {}", e))?;

        Ok(ExecutionResult {
            transaction_hash: format!("{:?}", receipt.transaction_hash),
            block_number: receipt.block_number,
            gas_used: Some(receipt.gas_used),
            status: Some(receipt.status()),
            contract_address: receipt.contract_address.map(|addr| format!("{addr:?}")),
        })
    }
}
