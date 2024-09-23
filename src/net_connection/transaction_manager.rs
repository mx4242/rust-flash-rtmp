use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use flash_lso::types::Value;

use crate::net_connection::packets::AMFCommandMessage;

pub struct Transaction {
    result_callback: Box<dyn Fn(Value, &[Value])>,
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction")
            .field("result_callback", &"Fn(Value, &[Value])")
            .finish()
    }
}

pub enum TransactionResult {
    Result,
    Error,
}

#[derive(Debug)]
pub struct TransactionManager {
    current_transaction_id: Arc<Mutex<u32>>,
    transactions: HashMap<u32, Transaction>,
}

impl TransactionManager {
    pub fn new() -> Self {
        TransactionManager {
            current_transaction_id: Arc::new(Mutex::new(1)),
            transactions: HashMap::new(),
        }
    }

    pub fn initialize_transaction(
        &mut self,
        result_callback: Box<dyn Fn(Value, &[Value])>,
    ) -> u32 {
        let mut current_transaction_id = self.current_transaction_id.lock().unwrap();
        *current_transaction_id += 1;

        let transaction_id = *current_transaction_id;
        self.transactions.insert(
            transaction_id,
            Transaction {
                result_callback,
            },
        );

        transaction_id
    }

    pub fn finalize_transaction(&mut self, transaction_id: u32, result: TransactionResult, response: AMFCommandMessage) -> std::io::Result<()> {
        let transaction = self.get_transaction(transaction_id).unwrap();
        let response = response;

        let callback = match result {
            TransactionResult::Result => &transaction.result_callback,
            // TransactionResult::Error => &transaction.error_callback,
            TransactionResult::Error => todo!("Error callback"),
        };

        callback(response.command_object.unwrap(), &response.optional_arguments);

        self.clean_up_transaction(transaction_id);

        Ok(())
    }

    pub fn get_transaction(&self, transaction_id: u32) -> Option<&Transaction> {
        self.transactions.get(&transaction_id)
    }

    pub fn clean_up_transaction(&mut self, transaction_id: u32) {
        self.transactions.remove(&transaction_id);
    }

}