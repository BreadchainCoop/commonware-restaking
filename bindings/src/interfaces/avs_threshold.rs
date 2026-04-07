use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IAvsThreshold {
        function QUORUM_THRESHOLD() external view returns (uint256);
        function THRESHOLD_DENOMINATOR() external view returns (uint256);
    }
}
