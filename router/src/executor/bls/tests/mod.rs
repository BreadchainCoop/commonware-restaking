use crate::executor::bls::{BlsSignatureVerificationHandler, convert_non_signer_data};
use crate::executor::core::ExecutionResult;
use alloy_primitives::{Bytes, FixedBytes, U256};
use anyhow::Result;
use async_trait::async_trait;
use commonware_avs_bindings::blssigcheckoperatorstateretriever::{
    BLSSigCheckOperatorStateRetriever::getNonSignerStakesAndSignatureReturn, BN254,
    IBLSSignatureCheckerTypes,
};

/// Mock implementation of BlsSignatureVerificationHandler for testing purposes.
#[derive(Debug)]
pub struct MockVerificationHandler;

impl MockVerificationHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockVerificationHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlsSignatureVerificationHandler for MockVerificationHandler {
    type TaskData = ();
    async fn handle_verification(
        &mut self,
        _msg_hash: FixedBytes<32>,
        _quorum_numbers: Bytes,
        _current_block_number: u32,
        _non_signer_data: getNonSignerStakesAndSignatureReturn,
        _task_data: Option<&Self::TaskData>,
    ) -> Result<ExecutionResult> {
        // Mock implementation returns success with dummy values
        Ok(ExecutionResult {
            transaction_hash: "0x1234567890abcdef".to_string(),
            block_number: Some(12345),
            gas_used: Some(21000),
            status: Some(true),
            contract_address: None,
        })
    }
}

// Test BLS signature verification handler that can be configured for different test scenarios
struct TestBlsSignatureVerificationHandler {
    should_succeed: bool,
    expected_result: ExecutionResult,
    call_count: u32,
}

impl TestBlsSignatureVerificationHandler {
    fn new(should_succeed: bool, expected_result: ExecutionResult) -> Self {
        Self {
            should_succeed,
            expected_result,
            call_count: 0,
        }
    }
}

#[async_trait]
impl BlsSignatureVerificationHandler for TestBlsSignatureVerificationHandler {
    type TaskData = ();
    async fn handle_verification(
        &mut self,
        _msg_hash: FixedBytes<32>,
        _quorum_numbers: Bytes,
        _current_block_number: u32,
        _non_signer_data: getNonSignerStakesAndSignatureReturn,
        _task_data: Option<&Self::TaskData>,
    ) -> Result<ExecutionResult> {
        self.call_count += 1;

        if self.should_succeed {
            Ok(self.expected_result.clone())
        } else {
            Err(anyhow::anyhow!("Verification handler test failure"))
        }
    }
}

#[test]
fn test_mock_verification_handler_creation() {
    let handler = MockVerificationHandler::new();
    assert_eq!(format!("{handler:?}"), "MockVerificationHandler");
}

#[test]
fn test_mock_verification_handler_default() {
    let handler = MockVerificationHandler;
    assert_eq!(format!("{handler:?}"), "MockVerificationHandler");
}

#[tokio::test]
async fn test_mock_verification_handler_success() {
    let mut handler = MockVerificationHandler::new();

    let msg_hash = FixedBytes::<32>::ZERO;
    let quorum_numbers = Bytes::from_static(b"test");
    let current_block_number = 12345;

    // Create a mock non-signer data
    let mock_data = getNonSignerStakesAndSignatureReturn {
        _0: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: vec![],
            nonSignerPubkeys: vec![],
            quorumApks: vec![],
            apkG2: BN254::G2Point {
                X: [Default::default(), Default::default()],
                Y: [Default::default(), Default::default()],
            },
            sigma: BN254::G1Point {
                X: Default::default(),
                Y: Default::default(),
            },
            quorumApkIndices: vec![],
            totalStakeIndices: vec![],
            nonSignerStakeIndices: vec![],
        },
    };

    let result = handler
        .handle_verification(
            msg_hash,
            quorum_numbers,
            current_block_number,
            mock_data,
            None,
        )
        .await;

    assert!(result.is_ok());
    let execution_result = result.unwrap();
    assert_eq!(execution_result.transaction_hash, "0x1234567890abcdef");
    assert_eq!(execution_result.block_number, Some(12345));
    assert_eq!(execution_result.gas_used, Some(21000));
    assert_eq!(execution_result.status, Some(true));
    assert_eq!(execution_result.contract_address, None);
}

#[tokio::test]
async fn test_verification_handler_trait_success() {
    let expected_result = ExecutionResult {
        transaction_hash: "0xtest123".to_string(),
        block_number: Some(99999),
        gas_used: Some(75000),
        status: Some(true),
        contract_address: Some("0xcontract456".to_string()),
    };

    let mut handler = TestBlsSignatureVerificationHandler::new(true, expected_result.clone());

    let msg_hash = FixedBytes::<32>::ZERO;
    let quorum_numbers = Bytes::from_static(b"test");
    let current_block_number = 54321;

    let mock_data = getNonSignerStakesAndSignatureReturn {
        _0: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: vec![],
            nonSignerPubkeys: vec![],
            quorumApks: vec![],
            apkG2: BN254::G2Point {
                X: [Default::default(), Default::default()],
                Y: [Default::default(), Default::default()],
            },
            sigma: BN254::G1Point {
                X: Default::default(),
                Y: Default::default(),
            },
            quorumApkIndices: vec![],
            totalStakeIndices: vec![],
            nonSignerStakeIndices: vec![],
        },
    };

    let result = handler
        .handle_verification(
            msg_hash,
            quorum_numbers,
            current_block_number,
            mock_data,
            None,
        )
        .await;

    assert!(result.is_ok());
    let execution_result = result.unwrap();
    assert_eq!(
        execution_result.transaction_hash,
        expected_result.transaction_hash
    );
    assert_eq!(execution_result.block_number, expected_result.block_number);
    assert_eq!(execution_result.gas_used, expected_result.gas_used);
    assert_eq!(handler.call_count, 1);
}

#[tokio::test]
async fn test_verification_handler_trait_failure() {
    let expected_result = ExecutionResult {
        transaction_hash: "".to_string(),
        block_number: None,
        gas_used: None,
        status: None,
        contract_address: None,
    };

    let mut handler = TestBlsSignatureVerificationHandler::new(false, expected_result);

    let msg_hash = FixedBytes::<32>::ZERO;
    let quorum_numbers = Bytes::from_static(b"test");
    let current_block_number = 54321;

    let mock_data = getNonSignerStakesAndSignatureReturn {
        _0: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: vec![],
            nonSignerPubkeys: vec![],
            quorumApks: vec![],
            apkG2: BN254::G2Point {
                X: [Default::default(), Default::default()],
                Y: [Default::default(), Default::default()],
            },
            sigma: BN254::G1Point {
                X: Default::default(),
                Y: Default::default(),
            },
            quorumApkIndices: vec![],
            totalStakeIndices: vec![],
            nonSignerStakeIndices: vec![],
        },
    };

    let result = handler
        .handle_verification(
            msg_hash,
            quorum_numbers,
            current_block_number,
            mock_data,
            None,
        )
        .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Verification handler test failure")
    );
    assert_eq!(handler.call_count, 1);
}

#[test]
fn test_convert_non_signer_data_empty() {
    // Test with empty data
    let input = getNonSignerStakesAndSignatureReturn {
        _0: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: vec![],
            nonSignerPubkeys: vec![],
            quorumApks: vec![],
            apkG2: BN254::G2Point {
                X: [U256::ZERO, U256::ZERO],
                Y: [U256::ZERO, U256::ZERO],
            },
            sigma: BN254::G1Point {
                X: U256::ZERO,
                Y: U256::ZERO,
            },
            quorumApkIndices: vec![],
            totalStakeIndices: vec![],
            nonSignerStakeIndices: vec![],
        },
    };

    let result = convert_non_signer_data(input);

    // Verify all fields are correctly converted
    assert_eq!(result.nonSignerQuorumBitmapIndices, Vec::<u32>::new());
    assert_eq!(result.nonSignerPubkeys.len(), 0);
    assert_eq!(result.quorumApks.len(), 0);
    assert_eq!(result.apkG2.X, [U256::ZERO, U256::ZERO]);
    assert_eq!(result.apkG2.Y, [U256::ZERO, U256::ZERO]);
    assert_eq!(result.sigma.X, U256::ZERO);
    assert_eq!(result.sigma.Y, U256::ZERO);
    assert_eq!(result.quorumApkIndices, Vec::<u32>::new());
    assert_eq!(result.totalStakeIndices, Vec::<u32>::new());
    assert_eq!(result.nonSignerStakeIndices, Vec::<Vec<u32>>::new());
}

#[test]
fn test_convert_non_signer_data_with_values() {
    // Test with actual data
    let test_x = U256::from(12345);
    let test_y = U256::from(67890);
    let test_x2 = U256::from(11111);
    let test_y2 = U256::from(22222);

    let input = getNonSignerStakesAndSignatureReturn {
        _0: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: vec![1, 2, 3],
            nonSignerPubkeys: vec![
                BN254::G1Point {
                    X: test_x,
                    Y: test_y,
                },
                BN254::G1Point {
                    X: test_x2,
                    Y: test_y2,
                },
            ],
            quorumApks: vec![BN254::G1Point {
                X: test_x,
                Y: test_y,
            }],
            apkG2: BN254::G2Point {
                X: [test_x, test_y],
                Y: [test_x2, test_y2],
            },
            sigma: BN254::G1Point {
                X: test_x,
                Y: test_y,
            },
            quorumApkIndices: vec![4, 5],
            totalStakeIndices: vec![6, 7, 8],
            nonSignerStakeIndices: vec![vec![9, 10], vec![11, 12, 13]],
        },
    };

    let result = convert_non_signer_data(input);

    // Verify all fields are correctly converted
    assert_eq!(result.nonSignerQuorumBitmapIndices, vec![1, 2, 3]);
    assert_eq!(result.nonSignerPubkeys.len(), 2);
    assert_eq!(result.nonSignerPubkeys[0].X, test_x);
    assert_eq!(result.nonSignerPubkeys[0].Y, test_y);
    assert_eq!(result.nonSignerPubkeys[1].X, test_x2);
    assert_eq!(result.nonSignerPubkeys[1].Y, test_y2);

    assert_eq!(result.quorumApks.len(), 1);
    assert_eq!(result.quorumApks[0].X, test_x);
    assert_eq!(result.quorumApks[0].Y, test_y);

    assert_eq!(result.apkG2.X, [test_x, test_y]);
    assert_eq!(result.apkG2.Y, [test_x2, test_y2]);

    assert_eq!(result.sigma.X, test_x);
    assert_eq!(result.sigma.Y, test_y);

    assert_eq!(result.quorumApkIndices, vec![4, 5]);
    assert_eq!(result.totalStakeIndices, vec![6, 7, 8]);
    assert_eq!(
        result.nonSignerStakeIndices,
        vec![vec![9, 10], vec![11, 12, 13]]
    );
}

#[test]
fn test_convert_non_signer_data_preserves_data_integrity() {
    // Test that the conversion preserves data integrity - what goes in comes out
    let original_indices = vec![100, 200, 300];
    let original_quorum_indices = vec![400, 500];
    let original_total_indices = vec![600, 700, 800, 900];
    let original_non_signer_indices = vec![vec![1000, 1100], vec![1200]];

    let input = getNonSignerStakesAndSignatureReturn {
        _0: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: original_indices.clone(),
            nonSignerPubkeys: vec![],
            quorumApks: vec![],
            apkG2: BN254::G2Point {
                X: [U256::from(1), U256::from(2)],
                Y: [U256::from(3), U256::from(4)],
            },
            sigma: BN254::G1Point {
                X: U256::from(5),
                Y: U256::from(6),
            },
            quorumApkIndices: original_quorum_indices.clone(),
            totalStakeIndices: original_total_indices.clone(),
            nonSignerStakeIndices: original_non_signer_indices.clone(),
        },
    };

    let result = convert_non_signer_data(input);

    // Verify exact preservation of values
    assert_eq!(result.nonSignerQuorumBitmapIndices, original_indices);
    assert_eq!(result.quorumApkIndices, original_quorum_indices);
    assert_eq!(result.totalStakeIndices, original_total_indices);
    assert_eq!(result.nonSignerStakeIndices, original_non_signer_indices);

    // Verify G2Point conversion
    assert_eq!(result.apkG2.X[0], U256::from(1));
    assert_eq!(result.apkG2.X[1], U256::from(2));
    assert_eq!(result.apkG2.Y[0], U256::from(3));
    assert_eq!(result.apkG2.Y[1], U256::from(4));

    // Verify G1Point conversion
    assert_eq!(result.sigma.X, U256::from(5));
    assert_eq!(result.sigma.Y, U256::from(6));
}

#[test]
fn test_convert_non_signer_data_g1_points_conversion() {
    // Test specific conversion of G1Points in arrays
    let point1 = BN254::G1Point {
        X: U256::from(111),
        Y: U256::from(222),
    };
    let point2 = BN254::G1Point {
        X: U256::from(333),
        Y: U256::from(444),
    };

    let input = getNonSignerStakesAndSignatureReturn {
        _0: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: vec![],
            nonSignerPubkeys: vec![point1.clone()],
            quorumApks: vec![point1.clone(), point2.clone()],
            apkG2: BN254::G2Point {
                X: [U256::ZERO, U256::ZERO],
                Y: [U256::ZERO, U256::ZERO],
            },
            sigma: point2.clone(),
            quorumApkIndices: vec![],
            totalStakeIndices: vec![],
            nonSignerStakeIndices: vec![],
        },
    };

    let result = convert_non_signer_data(input);

    // Verify nonSignerPubkeys conversion
    assert_eq!(result.nonSignerPubkeys.len(), 1);
    assert_eq!(result.nonSignerPubkeys[0].X, U256::from(111));
    assert_eq!(result.nonSignerPubkeys[0].Y, U256::from(222));

    // Verify quorumApks conversion
    assert_eq!(result.quorumApks.len(), 2);
    assert_eq!(result.quorumApks[0].X, U256::from(111));
    assert_eq!(result.quorumApks[0].Y, U256::from(222));
    assert_eq!(result.quorumApks[1].X, U256::from(333));
    assert_eq!(result.quorumApks[1].Y, U256::from(444));

    // Verify sigma conversion
    assert_eq!(result.sigma.X, U256::from(333));
    assert_eq!(result.sigma.Y, U256::from(444));
}
