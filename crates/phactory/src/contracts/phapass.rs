use std::collections::BTreeMap;
use anyhow::Result;
use log::info;
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;

use super::{TransactionError, TransactionResult};
use crate::contracts;
use crate::contracts::{AccountId, NativeContext};
extern crate runtime as chain;

use phala_types::messaging::PhaPassCommand;

type Command = PhaPassCommand;

#[derive(Encode, Decode, Debug, Clone)]
pub struct Credential {
    username: String,
    password: String,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct ListedCredential {
    url: String,
    username: String,
}
pub struct UserVault {
    keys: String,
    credentials: BTreeMap<String, Credential>,
}

pub struct PhaPass {
    vaults: BTreeMap<AccountId, UserVault>,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Request {
    HasAVault,
    GetKeys,
    GetCredential { url: String},
    ListCredentials,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum Response {
    HasAVault(bool),
    Keys(String),
    ExistingCredential(Credential),
    Credentials(Vec<ListedCredential>)
}

#[derive(Encode, Decode, Debug)]
pub enum Error {
    OriginUnavailable,
    VaultAlreadyExists,
    NoVault,
    NoCredential,
    NotAuthorized,
}

impl ListedCredential {
    pub fn new(url: String, username: String) -> Self {
        ListedCredential  { url, username}
    }
}

impl Credential {
    pub fn new(username: String, password: String) -> Self {
        Credential  { username, password}
    }
}


impl UserVault {
    pub fn new(keys: String) -> Self {
        UserVault {
            keys,
            credentials: BTreeMap::new(),
        }
    }
}

impl PhaPass {
    pub fn new() -> Self {
        PhaPass {
            vaults: BTreeMap::new(),
        }
    }
}

impl contracts::NativeContract for PhaPass {
    type Cmd = Command;
    type QReq = Request;
    type QResp = Result<Response, Error>;

    /// Return the contract id which uniquely identifies the contract
    fn id(&self) -> contracts::ContractId32 {
        contracts::PHAPASS
    }

    /// Handle the Commands from transactions on the blockchain. This method doesn't respond.
    ///
    /// # Arguments
    ///
    /// * `context` - The current block info with the necessary egress channel
    /// * `origin` - The sender of the Command, can be Pallet, pRuntime, Contract, Account or even entities from other chain
    /// * `cmd` - The on-chain Command to process
    fn handle_command(
        &mut self,
        _context: &mut NativeContext,
        origin: MessageOrigin,
        cmd: Command,
    ) -> TransactionResult {
        info!("*******************************************************************************************");
        info!("Command received: {:?}", &cmd);
        let sender = match &origin {
            MessageOrigin::AccountId(account) => AccountId::from(*account.as_fixed_bytes()),
            _ => return Err(TransactionError::BadOrigin),
        };
        match cmd {
            Command::CreateVault { keys } => {
                if self.vaults.contains_key(&sender){
                    return Err(TransactionError::VaultAlreadyExists);
                }
                self.vaults.insert(sender, UserVault::new(keys));
                Ok(())
            },
            Command::AddCredential { url, username, password } => {
                if let Some(user_vault) = self.vaults.get_mut(&sender) {
                    let credential = Credential::new(username, password);
                    user_vault.credentials.insert(url, credential);
                    Ok(())
                } else {
                    Err(TransactionError::NoVault)
                }
            },
            Command::RemoveCredential { url } => {
                if let Some(user_vault) = self.vaults.get_mut(&sender) {
                    if user_vault.credentials.contains_key(&url) {
                        user_vault.credentials.remove(&url);
                        Ok(())
                    }else{
                        Err(TransactionError::NoCredential)
                    }
                } else {
                    Err(TransactionError::NoVault)
                }
            }
        }
    }

    /// Handle a direct Query and respond to it. It shouldn't modify the contract state.
    ///
    /// # Arguments
    ///
    /// * `origin` - For off-chain Query, the sender can only be AccountId
    /// * `req` — Off-chain Query to handle
    fn handle_query(
        &mut self,
        origin: Option<&chain::AccountId>,
        req: Request,
    ) -> Result<Response, Error> {
        info!("*******************************************************************************************");
        info!("Query received: {:?}", &req);
        match req {
            Request::HasAVault => {
                info!("HasAVault Query received: {:?}", &req);
                let owner = origin.ok_or(Error::OriginUnavailable)?;
                info!("owner: {:?}", &owner);
                Ok(Response::HasAVault(self.vaults.contains_key(owner)))
            },
            Request::GetKeys => {
                info!("GetKeys Query received");
                let owner = origin.ok_or(Error::OriginUnavailable)?;
                info!("owner: {:?}", &owner);
                if let Some(user_vault) = self.vaults.get(owner) {
                    Ok(Response::Keys(user_vault.keys.clone()))
                } else {
                    Err(Error::NoVault)
                }
            },
            Request::GetCredential{ url } => {
                info!("GetCredential Query received");
                let owner = origin.ok_or(Error::OriginUnavailable)?;
                info!("owner: {:?}", &owner);
                if let Some(user_vault) = self.vaults.get(owner) {
                    if let Some(credential) = user_vault.credentials.get(&url) {
                        info!("Credential: {:?}", credential.clone());
                        Ok(Response::ExistingCredential(credential.clone()))
                    }else{
                        Err(Error::NoCredential)
                    }
                } else {
                    Err(Error::NoVault)
                }
            },
            Request::ListCredentials => {
                info!("ListCredentials Query received");
                let owner = origin.ok_or(Error::OriginUnavailable)?;
                info!("owner: {:?}", &owner);
                if let Some(user_vault) = self.vaults.get(owner) {
                    let mut credentials_list = Vec::<ListedCredential>::new();
                    for (url, credential) in user_vault.credentials.iter(){
                        credentials_list.push(ListedCredential::new(url.clone(), credential.username.clone()))        
                    }
                    Ok(Response::Credentials(credentials_list))
                } else {
                    Err(Error::NoVault)
                }
            }
        }
    }
}
