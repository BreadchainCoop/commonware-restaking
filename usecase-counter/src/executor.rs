use commonware_avs_bindings::WalletProvider;
use commonware_avs_bindings::counter::Counter;

pub struct CounterHandler {
    pub counter: Counter::CounterInstance<(), WalletProvider>,
}

impl CounterHandler {
    pub fn new(counter: Counter::CounterInstance<(), WalletProvider>) -> Self {
        Self { counter }
    }
}
