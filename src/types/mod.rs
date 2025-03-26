// pub mod executor;

use alloy_primitives::Log;
use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_rpc_types_trace::parity::{Action, CallType};
use alloy_rpc_types_trace::parity::{TraceOutput, TransactionTrace};
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

    pub fn is_static_call(&self) -> bool {
        match &self.trace.action {
            Action::Call(call) => call.call_type == CallType::StaticCall,
            _ => false,
        }
    }

    pub fn is_create(&self) -> bool {
        matches!(&self.trace.action, Action::Create(_))
    }

    pub fn is_delegate_call(&self) -> bool {
        match &self.trace.action {
            Action::Call(c) => c.call_type == CallType::DelegateCall,
            _ => false,
        }
    }

    pub fn get_create_output(&self) -> Address {
        match &self.trace.result {
            Some(TraceOutput::Create(o)) => o.address,
            _ => Address::default(),
        }
    }

    pub fn action_type(&self) -> &Action {
        &self.trace.action
    }

    pub fn get_from_addr(&self) -> Address {
        match &self.trace.action {
            Action::Call(call) => call.from,
            Action::Create(call) => call.from,
            Action::Reward(call) => call.author,
            Action::Selfdestruct(call) => call.address,
        }
    }

    pub fn get_msg_sender(&self) -> Address {
        self.msg_sender
    }

    pub fn get_to_address(&self) -> Address {
        match &self.trace.action {
            Action::Call(call) => call.to,
            Action::Create(_) => Address::default(),
            Action::Reward(_) => Address::default(),
            Action::Selfdestruct(call) => call.address,
        }
    }

    pub fn get_calldata(&self) -> Bytes {
        match &self.trace.action {
            Action::Call(call) => call.input.clone(),
            Action::Create(call) => call.init.clone(),
            _ => Bytes::default(),
        }
    }

    pub fn get_return_calldata(&self) -> Bytes {
        let Some(res) = &self.trace.result else {
            return Bytes::default();
        };
        match res {
            alloy_rpc_types_trace::parity::TraceOutput::Call(bytes) => bytes.output.clone(),
            _ => Bytes::default(),
        }
    }

    pub fn get_callframe_info(&self) -> CallFrameInfo<'_> {
        CallFrameInfo {
            trace_idx: self.trace_idx,
            call_data: self.get_calldata(),
            return_data: self.get_return_calldata(),
            target_address: self.get_to_address(),
            from_address: self.get_from_addr(),
            logs: &self.logs,
            delegate_logs: vec![],
            msg_sender: self.msg_sender,
            msg_value: self.get_msg_value(),
        }
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

#[derive(Debug, Clone)]
pub struct CallFrameInfo<'a> {
    pub trace_idx: u64,
    pub call_data: Bytes,
    pub return_data: Bytes,
    pub target_address: Address,
    pub from_address: Address,
    pub logs: &'a [Log],
    pub delegate_logs: Vec<&'a Log>,
    pub msg_sender: Address,
    pub msg_value: U256,
}

impl CallFrameInfo<'_> {
    pub fn get_fixed_fields(&self) -> CallInfo {
        CallInfo {
            trace_idx: self.trace_idx,
            target_address: self.target_address,
            from_address: self.from_address,
            msg_sender: self.msg_sender,
            msg_value: self.msg_value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallInfo {
    pub trace_idx: u64,
    pub target_address: Address,
    pub from_address: Address,
    pub msg_sender: Address,
    pub msg_value: U256,
}
