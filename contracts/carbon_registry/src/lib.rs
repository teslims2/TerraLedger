#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, contracttype, Address, Env, String, Symbol, symbol_short};

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
pub enum ProjectStatus {
    Pending,
    Verified,
    Rejected,
    Suspended,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: String,
    pub developer: Address,
    pub methodology: String,
    pub country: String,
    pub lat: String,
    pub lng: String,
    pub status: ProjectStatus,
    pub registered_at: u64,
    pub verified_at: u64,
    pub verifier: Option<Address>,
    pub total_issued: u64,
    pub total_retired: u64,
}

#[contract]
pub struct CarbonRegistry;

#[contractimpl]
impl CarbonRegistry {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("admin"), &admin);
    }

    pub fn add_verifier(env: Env, verifier: Address) {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        env.storage().instance().set(&Symbol::new(&env, "verifier"), &verifier);
    }

    pub fn add_oracle(env: Env, oracle: Address) {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        env.storage().instance().set(&Symbol::new(&env, "oracle"), &oracle);
    }

    pub fn register_project(env: Env, id: String, developer: Address, methodology: String, country: String, lat: String, lng: String) -> Result<(), CarbonError> {
        developer.require_auth();
        if env.storage().persistent().has(&id) {
            return Err(CarbonError::ProjectAlreadyExists);
        }
        let project = Project {
            id: id.clone(),
            developer,
            methodology,
            country,
            lat,
            lng,
            status: ProjectStatus::Pending,
            registered_at: env.ledger().timestamp(),
            verified_at: 0,
            verifier: None,
            total_issued: 0,
            total_retired: 0,
        };
        env.storage().persistent().set(&id, &project);
        env.events().publish((symbol_short!("register"),), id);
        Ok(())
    }

    pub fn verify_project(env: Env, verifier: Address, id: String) -> Result<(), CarbonError> {
        verifier.require_auth();
        let expected_verifier: Option<Address> = env.storage().instance().get(&Symbol::new(&env, "verifier"));
        if expected_verifier != Some(verifier.clone()) {
            return Err(CarbonError::UnauthorizedVerifier);
        }
        let mut project: Project = env.storage().persistent().get(&id).ok_or(CarbonError::ProjectNotFound)?;
        project.status = ProjectStatus::Verified;
        project.verified_at = env.ledger().timestamp();
        project.verifier = Some(verifier);
        env.storage().persistent().set(&id, &project);
        env.events().publish((symbol_short!("verify"),), id);
        Ok(())
    }

    pub fn reject_project(env: Env, verifier: Address, id: String) -> Result<(), CarbonError> {
        verifier.require_auth();
        let mut project: Project = env.storage().persistent().get(&id).ok_or(CarbonError::ProjectNotFound)?;
        project.status = ProjectStatus::Rejected;
        env.storage().persistent().set(&id, &project);
        env.events().publish((symbol_short!("reject"),), id);
        Ok(())
    }

    pub fn suspend_project(env: Env, admin: Address, id: String) -> Result<(), CarbonError> {
        admin.require_auth();
        let mut project: Project = env.storage().persistent().get(&id).ok_or(CarbonError::ProjectNotFound)?;
        project.status = ProjectStatus::Suspended;
        env.storage().persistent().set(&id, &project);
        env.events().publish((symbol_short!("suspend"),), id);
        Ok(())
    }

    pub fn increment_issued(env: Env, admin: Address, id: String, amount: u64) -> Result<(), CarbonError> {
        admin.require_auth();
        let mut project: Project = env.storage().persistent().get(&id).ok_or(CarbonError::ProjectNotFound)?;
        project.total_issued += amount;
        env.storage().persistent().set(&id, &project);
        Ok(())
    }

    pub fn increment_retired(env: Env, admin: Address, id: String, amount: u64) -> Result<(), CarbonError> {
        admin.require_auth();
        let mut project: Project = env.storage().persistent().get(&id).ok_or(CarbonError::ProjectNotFound)?;
        project.total_retired += amount;
        env.storage().persistent().set(&id, &project);
        Ok(())
    }

    pub fn get_project(env: Env, id: String) -> Result<Project, CarbonError> {
        env.storage().persistent().get(&id).ok_or(CarbonError::ProjectNotFound)
    }

    pub fn is_verified(env: Env, id: String) -> bool {
        if let Some(project) = env.storage().persistent().get::<_, Project>(&id) {
            project.status == ProjectStatus::Verified
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn register_success() {
        let env = Env::default();
        let contract_id = env.register(CarbonRegistry, ());
        let client = CarbonRegistryClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let dev = Address::generate(&env);
        env.mock_all_auths();
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
    }

    #[test]
    #[should_panic]
    fn duplicate_fails() {
        let env = Env::default();
        let contract_id = env.register(CarbonRegistry, ());
        let client = CarbonRegistryClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let dev = Address::generate(&env);
        env.mock_all_auths();
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
    }

    #[test]
    fn verify_success() {
        let env = Env::default();
        let contract_id = env.register(CarbonRegistry, ());
        let client = CarbonRegistryClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        client.initialize(&admin);
        
        let verifier = Address::generate(&env);
        env.mock_all_auths();
        client.add_verifier(&verifier);

        let dev = Address::generate(&env);
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
        client.verify_project(&verifier, &String::from_str(&env, "P1"));
    }

    #[test]
    fn reject() {
        let env = Env::default();
        let contract_id = env.register(CarbonRegistry, ());
        let client = CarbonRegistryClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        let verifier = Address::generate(&env);
        env.mock_all_auths();
        client.add_verifier(&verifier);
        let dev = Address::generate(&env);
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
        client.reject_project(&verifier, &String::from_str(&env, "P1"));
    }

    #[test]
    fn suspend() {
        let env = Env::default();
        let contract_id = env.register(CarbonRegistry, ());
        let client = CarbonRegistryClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        let dev = Address::generate(&env);
        env.mock_all_auths();
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
        client.suspend_project(&admin, &String::from_str(&env, "P1"));
    }

    #[test]
    #[should_panic]
    fn unauthorized_verifier_fails() {
        let env = Env::default();
        let contract_id = env.register(CarbonRegistry, ());
        let client = CarbonRegistryClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        let dev = Address::generate(&env);
        env.mock_all_auths();
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
        let verifier2 = Address::generate(&env);
        client.verify_project(&verifier2, &String::from_str(&env, "P1"));
    }

    #[test]
    fn increment_counters() {
        let env = Env::default();
        let contract_id = env.register(CarbonRegistry, ());
        let client = CarbonRegistryClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        let dev = Address::generate(&env);
        env.mock_all_auths();
        client.register_project(&String::from_str(&env, "P1"), &dev, &String::from_str(&env, "VCS"), &String::from_str(&env, "US"), &String::from_str(&env, "0.0"), &String::from_str(&env, "0.0"));
        client.increment_issued(&admin, &String::from_str(&env, "P1"), &100);
        client.increment_retired(&admin, &String::from_str(&env, "P1"), &50);
    }
}
