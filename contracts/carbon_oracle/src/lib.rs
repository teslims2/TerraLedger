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
pub struct MonitoringData {
    pub project_id: String,
    pub submitted_by: Address,
    pub data_hash: BytesN<32>,
    pub sequestration_tonnes: u64,
    pub period_start: u64,
    pub period_end: u64,
    pub submitted_at: u64,
    pub source: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub methodology: String,
    pub vintage_year: u32,
    pub price_per_tonne: u64,
    pub source: String,
    pub updated_at: u64,
}

#[contract]
pub struct CarbonOracle;

#[contractimpl]
impl CarbonOracle {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("admin"), &admin);
    }

    pub fn add_oracle(env: Env, admin: Address, oracle: Address) {
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("oracle"), &oracle);
    }

    pub fn submit_monitoring_data(env: Env, oracle: Address, project_id: String, data_hash: BytesN<32>, sequestration_tonnes: u64, period_start: u64, period_end: u64, source: String) -> Result<(), CarbonError> {
        oracle.require_auth();
        let expected_oracle: Address = env.storage().instance().get(&symbol_short!("oracle")).unwrap();
        if expected_oracle != oracle {
            return Err(CarbonError::UnauthorizedOracle);
        }
        let data = MonitoringData {
            project_id: project_id.clone(),
            submitted_by: oracle,
            data_hash,
            sequestration_tonnes,
            period_start,
            period_end,
            submitted_at: env.ledger().timestamp(),
            source,
        };
        env.storage().persistent().set(&project_id, &data);
        Ok(())
    }

    pub fn update_credit_price(env: Env, oracle: Address, methodology: String, vintage_year: u32, price_per_tonne: u64, source: String) -> Result<(), CarbonError> {
        oracle.require_auth();
        let expected_oracle: Address = env.storage().instance().get(&symbol_short!("oracle")).unwrap();
        if expected_oracle != oracle {
            return Err(CarbonError::UnauthorizedOracle);
        }
        let price_data = PriceData {
            methodology: methodology.clone(),
            vintage_year,
            price_per_tonne,
            source,
            updated_at: env.ledger().timestamp(),
        };
        let key = (methodology, vintage_year);
        env.storage().persistent().set(&key, &price_data);
        Ok(())
    }

    pub fn flag_project(env: Env, oracle: Address, project_id: String) -> Result<(), CarbonError> {
        oracle.require_auth();
        let expected_oracle: Address = env.storage().instance().get(&symbol_short!("oracle")).unwrap();
        if expected_oracle != oracle {
            return Err(CarbonError::UnauthorizedOracle);
        }
        env.storage().persistent().set(&(project_id.clone(), symbol_short!("flag")), &true);
        Ok(())
    }

    pub fn is_monitoring_current(env: Env, project_id: String) -> bool {
        if let Some(data) = env.storage().persistent().get::<_, MonitoringData>(&project_id) {
            let ts = env.ledger().timestamp();
            if ts > data.submitted_at + 31536000 { // 365 days
                return false;
            }
            return true;
        }
        false
    }

    pub fn get_monitoring_data(env: Env, project_id: String) -> Result<MonitoringData, CarbonError> {
        env.storage().persistent().get(&project_id).ok_or(CarbonError::MonitoringDataStale)
    }

    pub fn get_benchmark_price(env: Env, methodology: String, vintage_year: u32) -> Result<PriceData, CarbonError> {
        env.storage().persistent().get(&(methodology, vintage_year)).ok_or(CarbonError::PriceNotSet)
    }

    pub fn get_flagged_projects(env: Env, project_id: String) -> bool {
        env.storage().persistent().get(&(project_id, symbol_short!("flag"))).unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String, BytesN};

    #[test]
    fn submit_monitoring() {
        let env = Env::default();
        let contract_id = env.register(CarbonOracle, ());
        let client = CarbonOracleClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let oracle = Address::generate(&env);
        client.add_oracle(&admin, &oracle);
        let hash = BytesN::from_array(&env, &[0; 32]);
        client.submit_monitoring_data(&oracle, &String::from_str(&env, "P1"), &hash, &100, &0, &100, &String::from_str(&env, "SRC"));
    }

    #[test]
    #[should_panic]
    fn unauthorized_oracle_fails() {
        let env = Env::default();
        let contract_id = env.register(CarbonOracle, ());
        let client = CarbonOracleClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let oracle = Address::generate(&env);
        client.add_oracle(&admin, &oracle);
        let other = Address::generate(&env);
        let hash = BytesN::from_array(&env, &[0; 32]);
        client.submit_monitoring_data(&other, &String::from_str(&env, "P1"), &hash, &100, &0, &100, &String::from_str(&env, "SRC"));
    }

    #[test]
    fn update_price() {
        let env = Env::default();
        let contract_id = env.register(CarbonOracle, ());
        let client = CarbonOracleClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let oracle = Address::generate(&env);
        client.add_oracle(&admin, &oracle);
        client.update_credit_price(&oracle, &String::from_str(&env, "VCS"), &2023, &15, &String::from_str(&env, "SRC"));
    }

    #[test]
    fn staleness_check() {
        let env = Env::default();
        let contract_id = env.register(CarbonOracle, ());
        let client = CarbonOracleClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let oracle = Address::generate(&env);
        client.add_oracle(&admin, &oracle);
        let hash = BytesN::from_array(&env, &[0; 32]);
        client.submit_monitoring_data(&oracle, &String::from_str(&env, "P1"), &hash, &100, &0, &100, &String::from_str(&env, "SRC"));
        assert!(client.is_monitoring_current(&String::from_str(&env, "P1")));
    }

    #[test]
    fn flag_project() {
        let env = Env::default();
        let contract_id = env.register(CarbonOracle, ());
        let client = CarbonOracleClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let oracle = Address::generate(&env);
        client.add_oracle(&admin, &oracle);
        client.flag_project(&oracle, &String::from_str(&env, "P1"));
    }

    #[test]
    fn get_benchmark_price() {
        let env = Env::default();
        let contract_id = env.register(CarbonOracle, ());
        let client = CarbonOracleClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        env.mock_all_auths();
        let oracle = Address::generate(&env);
        client.add_oracle(&admin, &oracle);
        client.update_credit_price(&oracle, &String::from_str(&env, "VCS"), &2023, &15, &String::from_str(&env, "SRC"));
        client.get_benchmark_price(&String::from_str(&env, "VCS"), &2023);
    }
}
