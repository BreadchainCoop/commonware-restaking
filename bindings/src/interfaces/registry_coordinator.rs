use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IRegistryCoordinator {
        function serviceManager() external view returns (address);
    }
}
