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

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Credential {
    username: String,
    password: String,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
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

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
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

    // Nota : we add this method for unit testing purpose, as the NativeContext
    // object needed in the NativeContract::handle_command() is not easily mockable
    fn handle_command(&mut self,
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
        self.handle_command(origin, cmd)
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

#[cfg(test)]
pub mod tests {
    use crate::contracts::support::NativeContract;
    use crate::contracts;
    use super::{ PhaPass, Request, Response, Command };
    use crate::contracts::{AccountId};
    use phala_mq::{ MessageOrigin, AccountId as PhalaMqAccountId};


    const ALICE_ADDRESS: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
    const BOB_ADDRESS: &str = "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";

    fn alice_account() -> AccountId{
        contracts::account_id_from_hex(ALICE_ADDRESS).expect("Bad initial alice address")
    }

    fn bob_account() -> AccountId{
        contracts::account_id_from_hex(BOB_ADDRESS).expect("Bad initial bob address")
    }
   
    #[test]
    fn module_should_have_a_correct_contract_id() {
        let testing_module: PhaPass = PhaPass::new();
        assert_eq!(
            testing_module.id(), contracts::PHAPASS,
            "Module should have a correct contract id"
        );
    }

    #[test]
    fn all_queries_need_an_origin() {
        let mut testing_module: PhaPass = PhaPass::new();
        assert!(
            testing_module.handle_query(Option::None, Request::HasAVault).is_err(),
            "HasAVault request needs an origin"
        );        
        assert!(
            testing_module.handle_query(Option::None, Request::GetKeys).is_err(),
            "GetKeys request needs an origin"
        );        
        assert!(
            testing_module.handle_query(Option::None, Request::GetCredential { url:"anyUrl".to_string() }).is_err(),
            "GetCredential request needs an origin"
        );        
        assert!(
            testing_module.handle_query(Option::None, Request::ListCredentials).is_err(),
            "ListCredentials request needs an origin"
        );        
    }    

    #[test]
    fn new_module_should_have_no_vault() {
        let mut testing_module: PhaPass = PhaPass::new();
        assert_eq!(
            testing_module.vaults.len(), 0,
            "Module should have no vault"
        );
        match testing_module.handle_query(Option::Some(&alice_account()), Request::HasAVault) {
            Ok(result) => assert_eq!(result, Response::HasAVault(false), "HasAVault query should return false"),
            Err(_) => assert!(false, "HasAVault query should fail"),
        }     
        match testing_module.handle_query(Option::Some(&alice_account()), Request::GetKeys) {
            Ok(_) => assert!(false, "GetKeys query should fail"),
            Err(_) => assert!(true),
        }     
        match testing_module.handle_query(Option::Some(&alice_account()), Request::ListCredentials) {
            Ok(_) => assert!(false, "ListCredentials query should fail"),
            Err(_) => assert!(true),
        }     
    }

    #[test]
    fn vault_creation_should_succeed() {
        let mut testing_module: PhaPass = PhaPass::new();
        let alice_private_keys = "Super Private Keys".to_string();
        let alice_address_bytes = hex::decode(ALICE_ADDRESS).expect("Failed to decode AccountId hex");
        let message_origin = MessageOrigin::AccountId(PhalaMqAccountId::from_slice(alice_address_bytes.as_slice()));
        let command = Command::CreateVault { keys: alice_private_keys.clone()};
        match testing_module.handle_command(message_origin, command){
            Ok(_) => assert!(true),
            Err(_) => assert!(false, "Vault creation command should succeed"),
        }
        assert_eq!(
            testing_module.vaults.len(), 1,
            "Module should have one vault"
        );
        match testing_module.handle_query(Option::Some(&alice_account()), Request::HasAVault) {
            Ok(result) => assert_eq!(result, Response::HasAVault(true), "Alice should have a vault"),
            Err(_) => assert!(false, "Alice query should succeed"),
        }     
        match testing_module.handle_query(Option::Some(&bob_account()), Request::HasAVault) {
            Ok(result) => assert_eq!(result, Response::HasAVault(false), "Bob should not have a vault"),
            Err(_) => assert!(false, "Bob query should succeed"),
        }       
        match testing_module.handle_query(Option::Some(&alice_account()), Request::GetKeys) {
            Ok(result) => {
                match result {
                    Response::Keys(keys) => assert_eq!(keys, alice_private_keys.clone(), "Alice private keys should match"),
                    _ => assert!(false, "GetKeys should return keys")
                }
            },
            Err(_) => assert!(false, "GetKeys query should succeed"),
        }     
        match testing_module.handle_query(Option::Some(&alice_account()), Request::ListCredentials) {
            Ok(result) => {
                match result {
                    Response::Credentials(credentials) => {
                        assert_eq!(credentials.len(), 0, "No credential should be returned");
                    }
                    _ => assert!(false, "ListCredentials should return credentials")
                }
            },
            Err(_) => assert!(false, "ListCredentials query should succeed"),
        }     
    }
}