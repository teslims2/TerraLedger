#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, contracttype, Address, Env, String, symbol_short, BytesN};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CarbonError {
    ProjectNotFound=1, ProjectNotVerified=2, ProjectSuspended=3,
    InsufficientCredits=4, AlreadyRetired=5, SerialNumberConflict=6,
    UnauthorizedVerifier=7, UnauthorizedOracle=8, InvalidVintageYear=9,
    ListingNotFound=10, InsufficientLiquidity=11, PriceNotSet=12,
    MonitoringDataStale=13, DoubleCountingDetected=14,
    RetirementIrreversible=15, ZeroAmountNotAllowed=16,
    ProjectAlreadyExists=17, InvalidSerialRange=18,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreditBatch {
    pub id: String,
    pub project_id: String,
    pub vintage_year: u32,
    pub methodology: String,
    pub serial_start: u64,
    pub serial_end: u64,
    pub total_amount: u64,
    pub available: u64,
    pub retired: u64,
    pub minted_at: u64,
    pub price_per_tonne: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetirementCertificate {
    pub cert_id: BytesN<32>,
    pub batch_id: String,
    pub project_id: String,
    pub retired_by: Address,
    pub beneficiary: String,
    pub reason: String,
    pub amount: u64,
    pub vintage_year: u32,
    pub serial_start: u64,
    pub serial_end: u64,
    pub retired_at: u64,
    pub methodology: String,
}

#[contract]
pub struct CarbonCredit;

#[contractimpl]
impl CarbonCredit {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("counter"), &0u64);
    }

    pub fn mint_credits(env: Env, admin: Address, id: String, project_id: String, vintage_year: u32, methodology: String, total_amount: u64, price_per_tonne: u64) -> Result<(), CarbonError> {
        admin.require_auth();
        if vintage_year < 2000 || vintage_year > 2100 {
            return Err(CarbonError::InvalidVintageYear);
        }
        if total_amount == 0 {
            return Err(CarbonError::ZeroAmountNotAllowed);
        }
        
        let mut counter: u64 = env.storage().instance().get(&symbol_short!("counter")).unwrap_or(0);
        let serial_start = (counter * 1_000_000) + 1;
        let serial_end = serial_start + total_amount - 1;
        counter += 1;
        env.storage().instance().set(&symbol_short!("counter"), &counter);

        let batch = CreditBatch {
            id: id.clone(),
            project_id,
            vintage_year,
            methodology,
            serial_start,
            serial_end,
            total_amount,
            available: total_amount,
            retired: 0,
            minted_at: env.ledger().timestamp(),
            price_per_tonne,
        };
        env.storage().persistent().set(&id, &batch);
        env.events().publish((symbol_short!("mint"),), id);
        Ok(())
    }

    pub fn transfer_credits(env: Env, from: Address, to: Address, batch_id: String, amount: u64) -> Result<(), CarbonError> {
        from.require_auth();
        if amount == 0 {
            return Err(CarbonError::ZeroAmountNotAllowed);
        }
        let balance_key = (from.clone(), batch_id.clone());
        let mut balance: u64 = env.storage().persistent().get(&balance_key).unwrap_or(0);
        if balance < amount {
            return Err(CarbonError::InsufficientCredits);
        }
        balance -= amount;
        env.storage().persistent().set(&balance_key, &balance);

        let to_key = (to.clone(), batch_id.clone());
        let mut to_balance: u64 = env.storage().persistent().get(&to_key).unwrap_or(0);
        to_balance += amount;
        env.storage().persistent().set(&to_key, &to_balance);
        
        env.events().publish((symbol_short!("transfer"),), batch_id);
        Ok(())
    }

    pub fn retire_credits(env: Env, owner: Address, batch_id: String, amount: u64, beneficiary: String, reason: String) -> Result<BytesN<32>, CarbonError> {
        owner.require_auth();
        if amount == 0 {
            return Err(CarbonError::ZeroAmountNotAllowed);
        }
        let balance_key = (owner.clone(), batch_id.clone());
        let mut balance: u64 = env.storage().persistent().get(&balance_key).unwrap_or(0);
        if balance < amount {
            return Err(CarbonError::InsufficientCredits);
        }
        balance -= amount;
        env.storage().persistent().set(&balance_key, &balance);

        let mut batch: CreditBatch = env.storage().persistent().get(&batch_id).ok_or(CarbonError::ListingNotFound)?;
        batch.available -= amount;
        batch.retired += amount;
        
        let retired_serial_start = batch.serial_start + batch.retired - amount;
        let retired_serial_end = batch.serial_start + batch.retired - 1;
        env.storage().persistent().set(&batch_id, &batch);

        let ts = env.ledger().timestamp();
        let cert_id = env.crypto().sha256(&soroban_sdk::Bytes::from_slice(&env, &[0; 32])); // Pseudo hash
        
        let cert = RetirementCertificate {
            cert_id: cert_id.clone(),
            batch_id: batch_id.clone(),
            project_id: batch.project_id,
            retired_by: owner,
            beneficiary,
            reason,
            amount,
            vintage_year: batch.vintage_year,
            serial_start: retired_serial_start,
            serial_end: retired_serial_end,
            retired_at: ts,
            methodology: batch.methodology,
        };
        env.storage().persistent().set(&cert_id, &cert);
        env.events().publish((symbol_short!("retire"),), cert_id.clone());
        Ok(cert_id)
    }

    pub fn credit_to_address(env: Env, admin: Address, to: Address, batch_id: String, amount: u64) -> Result<(), CarbonError> {
        admin.require_auth();
        let to_key = (to.clone(), batch_id.clone());
        let mut to_balance: u64 = env.storage().persistent().get(&to_key).unwrap_or(0);
        to_balance += amount;
        env.storage().persistent().set(&to_key, &to_balance);
        Ok(())
    }

    pub fn get_balance(env: Env, owner: Address, batch_id: String) -> u64 {
        env.storage().persistent().get(&(owner, batch_id)).unwrap_or(0)
    }

    pub fn get_credit_batch(env: Env, batch_id: String) -> Result<CreditBatch, CarbonError> {
        env.storage().persistent().get(&batch_id).ok_or(CarbonError::ListingNotFound)
    }

    pub fn get_retirement_certificate(env: Env, cert_id: BytesN<32>) -> Result<RetirementCertificate, CarbonError> {
        env.storage().persistent().get(&cert_id).ok_or(CarbonError::ListingNotFound)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn mint_success() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
    }
    
    #[test]
    #[should_panic]
    fn invalid_vintage_fails() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &1999, &String::from_str(&env, "VCS"), &1000, &10);
    }

    #[test]
    fn transfer() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        client.credit_to_address(&admin, &from, &String::from_str(&env, "B1"), &100);
        client.transfer_credits(&from, &to, &String::from_str(&env, "B1"), &50);
        assert_eq!(client.get_balance(&to, &String::from_str(&env, "B1")), 50);
    }

    #[test]
    #[should_panic]
    fn transfer_insufficient_fails() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        client.transfer_credits(&from, &to, &String::from_str(&env, "B1"), &50);
    }

    #[test]
    fn retire_generates_cert() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let owner = Address::generate(&env);
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
        client.credit_to_address(&admin, &owner, &String::from_str(&env, "B1"), &100);
        client.retire_credits(&owner, &String::from_str(&env, "B1"), &50, &String::from_str(&env, "Ben"), &String::from_str(&env, "ESG"));
    }

    #[test]
    #[should_panic]
    fn retire_more_than_owned_fails() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let owner = Address::generate(&env);
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
        client.credit_to_address(&admin, &owner, &String::from_str(&env, "B1"), &100);
        client.retire_credits(&owner, &String::from_str(&env, "B1"), &150, &String::from_str(&env, "Ben"), &String::from_str(&env, "ESG"));
    }

    #[test]
    #[should_panic]
    fn zero_amount_fails() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let owner = Address::generate(&env);
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
        client.credit_to_address(&admin, &owner, &String::from_str(&env, "B1"), &100);
        client.retire_credits(&owner, &String::from_str(&env, "B1"), &0, &String::from_str(&env, "Ben"), &String::from_str(&env, "ESG"));
    }

    #[test]
    fn balance_after_retire() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let owner = Address::generate(&env);
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
        client.credit_to_address(&admin, &owner, &String::from_str(&env, "B1"), &100);
        client.retire_credits(&owner, &String::from_str(&env, "B1"), &50, &String::from_str(&env, "Ben"), &String::from_str(&env, "ESG"));
        assert_eq!(client.get_balance(&owner, &String::from_str(&env, "B1")), 50);
    }

    #[test]
    fn multiple_batches_unique_serials() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
        client.mint_credits(&admin, &String::from_str(&env, "B2"), &String::from_str(&env, "P2"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
        let b1 = client.get_credit_batch(&String::from_str(&env, "B1"));
        let b2 = client.get_credit_batch(&String::from_str(&env, "B2"));
        assert!(b1.serial_start != b2.serial_start);
    }

    #[test]
    fn get_certificate() {
        let env = Env::default();
        let contract_id = env.register(CarbonCredit, ());
        let client = CarbonCreditClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let owner = Address::generate(&env);
        client.mint_credits(&admin, &String::from_str(&env, "B1"), &String::from_str(&env, "P1"), &2023, &String::from_str(&env, "VCS"), &1000, &10);
        client.credit_to_address(&admin, &owner, &String::from_str(&env, "B1"), &100);
        let cert_id = client.retire_credits(&owner, &String::from_str(&env, "B1"), &50, &String::from_str(&env, "Ben"), &String::from_str(&env, "ESG"));
        client.get_retirement_certificate(&cert_id);
    }
}
