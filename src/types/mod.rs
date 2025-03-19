pub mod executor;

use alloy_primitives::Log;
use alloy_primitives::{Address, B256, U256};
use alloy_rpc_types_trace::parity::Action;
use alloy_rpc_types_trace::parity::TransactionTrace;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct TxTrace {
    pub block_number: u64,
    pub trace: Vec<TransactionTraceWithLogs>,
    pub tx_hash: B256,
    pub gas_used: u128,
    pub effective_price: u128,
    pub tx_index: u64,
    // False if the transaction reverted
    pub is_success: bool,
}

impl TxTrace {
    pub fn new(
        block_number: u64,
        trace: Vec<TransactionTraceWithLogs>,
        tx_hash: B256,
        tx_index: u64,
        gas_used: u128,
        effective_price: u128,
        is_success: bool,
    ) -> Self {
        Self {
            block_number,
            trace,
            tx_hash,
            tx_index,
            effective_price,
            gas_used,
            is_success,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionTraceWithLogs {
    pub trace: TransactionTrace,
    pub logs: Vec<Log>,
    /// the msg.sender of the trace. This allows us to properly deal with
    /// delegate calls and the headache they cause when it comes to proxies
    pub msg_sender: Address,
    pub trace_idx: u64,
    pub decoded_data: Option<DecodedCallData>,
}

impl TransactionTraceWithLogs {
    pub fn get_msg_value(&self) -> U256 {
        match &self.trace.action {
            Action::Call(c) => c.value,
            Action::Create(c) => c.value,
            Action::Reward(r) => r.value,
            Action::Selfdestruct(_) => U256::ZERO,
        }
    }

    pub fn get_trace_address(&self) -> Vec<usize> {
        self.trace.trace_address.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

pub struct DecodedCallData {
    pub function_name: String,
    pub call_data: Vec<DecodedParams>,
    pub return_data: Vec<DecodedParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecodedParams {
    pub field_name: String,
    pub field_type: String,
    pub value: String,
}
