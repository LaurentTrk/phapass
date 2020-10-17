use serde::{Serialize, Deserialize};

use crate::contracts;
use crate::types::TxRef;
use crate::TransactionStatus;
use crate::contracts::{AccountIdWrapper};
use crate::std::collections::BTreeMap;
use crate::std::string::String;

/// HelloWorld contract states.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HelloWorld {
    counter: u32,
    notes: BTreeMap<AccountIdWrapper, String>,
}

/// The commands that the contract accepts from the blockchain. Also called transactions.
/// Commands are supposed to update the states of the contract.
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    /// Increments the counter in the contract by some number
    Increment {
        value: u32,
    },
    /// Set the secret note of the current user
    SetNote {
        note: String,
    }
}

/// The errors that the contract could throw for some queries
#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NotAuthorized,
    SomeOtherError,
}

/// Query requests. The end users can only query the contract states by sending requests.
/// Queries are not supposed to write to the contract states.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    /// Ask for the value of the counter
    GetCount,
    // Ask for the secret note
    GetNote,
}

/// Query responses.
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Returns the value of the counter
    GetCount {
        count: u32,
    },
    /// Returns the value of the secret note
    GetNote {
        note: String,
    },
    /// Something wrong happened
    Error(Error)
}


impl HelloWorld {
    /// Initializes the contract
    pub fn new() -> Self {
        Default::default()
    }
}

impl contracts::Contract<Command, Request, Response> for HelloWorld {
    // Returns the contract id
    fn id(&self) -> contracts::ContractId { contracts::HELLO_WORLD }

    // Handles the commands from transactions on the blockchain. This method doesn't respond.
    fn handle_command(&mut self, _origin: &chain::AccountId, _txref: &TxRef, cmd: Command) -> TransactionStatus {
        match cmd {
            // Handle the `Increment` command with one parameter
            Command::Increment { value } => {
                // Simply increment the counter by some value.
                self.counter += value;
                // Returns TransactionStatus::Ok to indicate a successful transaction
                TransactionStatus::Ok
            },
            // Handle the `SetNote` command with one parameter
            Command::SetNote { note } => {
                // Get the current user
                let current_user = AccountIdWrapper(_origin.clone());
                // Set the value of the user.
                self.notes.insert(current_user, note);
                // Returns TransactionStatus::Ok to indicate a successful transaction
                TransactionStatus::Ok
            },
        }
    }

    // Handles a direct query and responds to the query. It shouldn't modify the contract states.
    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Request) -> Response {
        let inner = || -> Result<Response, Error> {
            match req {
                // Handle the `GetCount` request.
                Request::GetCount => {
                    // Respond with the counter in the contract states.
                    Ok(Response::GetCount { count: self.counter })
                },
                // Handle the `GetNote` request.
                Request::GetNote => {
                    // Get the current user
                    if let Some(account) = origin {
                        let current_user = AccountIdWrapper(account.clone());
                        if self.notes.contains_key(&current_user) {
                            // Respond with the note in the notes.
                            let note = self.notes.get(&current_user);
                            return Ok(Response::GetNote { note: note.unwrap().clone() })
                        }
                    }
                    Err(Error::NotAuthorized)
                },
            }
        };
        match inner() {
            Err(error) => Response::Error(error),
            Ok(resp) => resp
        }
    }
}

