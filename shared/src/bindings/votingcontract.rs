use alloy::sol;

sol!(
    #[sol(rpc)]
    VotingContract,
    "src/bindings/abi/VotingContract.abi"
);
