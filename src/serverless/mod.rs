use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerlessConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub region: String,
    pub runtime: String,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub min_instances: u32,
    pub max_instances: u32,
    pub environment_variables: HashMap<String, String>,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPackage {
    pub id: String,
    pub config_id: String,
    pub version: String,
    pub package_size_bytes: u64,
    pub checksum: String,
    pub deployed_at: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub id: String,
    pub config_id: String,
    pub trigger_type: String,
    pub threshold: f64,
    pub scale_up_factor: f64,
    pub scale_down_factor: f64,
    pub cooldown_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStats {
    pub total_deployments: usize,
    pub active_deployments: usize,
    pub total_invocations: u64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerlessProvider {
    pub name: String,
    pub supported_regions: Vec<String>,
    pub supported_runtimes: Vec<String>,
    pub max_memory_mb: u32,
    pub max_timeout_seconds: u32,
    pub pricing_url: String,
}

pub struct ServerlessManager {
    configs: Arc<Mutex<HashMap<String, ServerlessConfig>>>,
    packages: Arc<Mutex<HashMap<String, DeploymentPackage>>>,
    scaling_policies: Arc<Mutex<HashMap<String, ScalingPolicy>>>,
    providers: Arc<Mutex<HashMap<String, ServerlessProvider>>>,
    invocation_counts: Arc<Mutex<HashMap<String, u64>>>,
}

impl ServerlessManager {
    pub fn new() -> Self {
        let manager = Self {
            configs: Arc::new(Mutex::new(HashMap::new())),
            packages: Arc::new(Mutex::new(HashMap::new())),
            scaling_policies: Arc::new(Mutex::new(HashMap::new())),
            providers: Arc::new(Mutex::new(HashMap::new())),
            invocation_counts: Arc::new(Mutex::new(HashMap::new())),
        };

        manager.initialize_providers();
        manager
    }

    fn initialize_providers(&self) {
        let aws_provider = ServerlessProvider {
            name: "aws".to_string(),
            supported_regions: vec![
                "us-east-1".to_string(),
                "us-west-2".to_string(),
                "eu-west-1".to_string(),
                "ap-northeast-1".to_string(),
            ],
            supported_runtimes: vec![
                "python3.9".to_string(),
                "python3.10".to_string(),
                "nodejs18.x".to_string(),
                "nodejs20.x".to_string(),
                "java11".to_string(),
                "java17".to_string(),
                "dotnet6".to_string(),
                "dotnet8".to_string(),
                "go1.x".to_string(),
                "ruby3.2".to_string(),
            ],
            max_memory_mb: 10240,
            max_timeout_seconds: 900,
            pricing_url: "https://aws.amazon.com/lambda/pricing/".to_string(),
        };

        let azure_provider = ServerlessProvider {
            name: "azure".to_string(),
            supported_regions: vec![
                "eastus".to_string(),
                "westus2".to_string(),
                "westeurope".to_string(),
                "southeastasia".to_string(),
            ],
            supported_runtimes: vec![
                "python".to_string(),
                "node".to_string(),
                "java".to_string(),
                "dotnet".to_string(),
                "powershell".to_string(),
            ],
            max_memory_mb: 14336,
            max_timeout_seconds: 600,
            pricing_url: "https://azure.microsoft.com/pricing/details/functions/".to_string(),
        };

        let gcp_provider = ServerlessProvider {
            name: "gcp".to_string(),
            supported_regions: vec![
                "us-central1".to_string(),
                "us-east1".to_string(),
                "europe-west1".to_string(),
                "asia-east1".to_string(),
            ],
            supported_runtimes: vec![
                "python39".to_string(),
                "python310".to_string(),
                "nodejs18".to_string(),
                "nodejs20".to_string(),
                "java11".to_string(),
                "java17".to_string(),
                "go121".to_string(),
                "ruby32".to_string(),
            ],
            max_memory_mb: 32768,
            max_timeout_seconds: 3600,
            pricing_url: "https://cloud.google.com/functions/pricing".to_string(),
        };

        let mut providers = self.providers.lock().unwrap();
        providers.insert(aws_provider.name.clone(), aws_provider);
        providers.insert(azure_provider.name.clone(), azure_provider);
        providers.insert(gcp_provider.name.clone(), gcp_provider);
    }

    pub fn create_deployment_config(
        &self,
        name: String,
        provider: String,
        region: String,
        runtime: String,
        memory_mb: u32,
        timeout_seconds: u32,
        environment_variables: HashMap<String, String>,
    ) -> Result<String, AppError> {
        let providers = self.providers.lock().map_err(|e| AppError::Internal(format!("Failed to lock providers: {}", e)))?;
        let provider_info = providers.get(&provider)
            .ok_or_else(|| AppError::NotFound(format!("Provider {} not supported", provider)))?;

        if !provider_info.supported_regions.contains(&region) {
            return Err(AppError::Internal(format!("Region {} not supported by {}", region, provider)));
        }

        if !provider_info.supported_runtimes.contains(&runtime) {
            return Err(AppError::Internal(format!("Runtime {} not supported by {}", runtime, provider)));
        }

        if memory_mb > provider_info.max_memory_mb {
            return Err(AppError::Internal(format!("Memory {}MB exceeds maximum {}MB for {}", memory_mb, provider_info.max_memory_mb, provider)));
        }

        if timeout_seconds > provider_info.max_timeout_seconds {
            return Err(AppError::Internal(format!("Timeout {}s exceeds maximum {}s for {}", timeout_seconds, provider_info.max_timeout_seconds, provider)));
        }

        drop(providers);

        let id = uuid::Uuid::new_v4().to_string();
        let config = ServerlessConfig {
            id: id.clone(),
            name,
            provider,
            region,
            runtime,
            memory_mb,
            timeout_seconds,
            min_instances: 0,
            max_instances: 100,
            environment_variables,
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
        };

        let mut configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        configs.insert(id.clone(), config);

        let mut invocation_counts = self.invocation_counts.lock().map_err(|e| AppError::Internal(format!("Failed to lock invocation counts: {}", e)))?;
        invocation_counts.insert(id.clone(), 0);

        Ok(id)
    }

    pub fn get_config(&self, config_id: &str) -> Result<ServerlessConfig, AppError> {
        let configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        configs.get(config_id).cloned().ok_or_else(|| AppError::NotFound(format!("Config {} not found", config_id)))
    }

    pub fn list_configs(&self) -> Vec<ServerlessConfig> {
        let configs = self.configs.lock().unwrap();
        configs.values().cloned().collect()
    }

    pub fn update_config(
        &self,
        config_id: &str,
        name: Option<String>,
        memory_mb: Option<u32>,
        timeout_seconds: Option<u32>,
        min_instances: Option<u32>,
        max_instances: Option<u32>,
        environment_variables: Option<HashMap<String, String>>,
    ) -> Result<(), AppError> {
        let mut configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        let config = configs.get_mut(config_id)
            .ok_or_else(|| AppError::NotFound(format!("Config {} not found", config_id)))?;

        if let Some(n) = name {
            config.name = n;
        }
        if let Some(m) = memory_mb {
            config.memory_mb = m;
        }
        if let Some(t) = timeout_seconds {
            config.timeout_seconds = t;
        }
        if let Some(min) = min_instances {
            config.min_instances = min;
        }
        if let Some(max) = max_instances {
            config.max_instances = max;
        }
        if let Some(env) = environment_variables {
            config.environment_variables = env;
        }

        Ok(())
    }

    pub fn delete_config(&self, config_id: &str) -> Result<(), AppError> {
        let mut configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        configs.remove(config_id).ok_or_else(|| AppError::NotFound(format!("Config {} not found", config_id)))?;

        let mut invocation_counts = self.invocation_counts.lock().map_err(|e| AppError::Internal(format!("Failed to lock invocation counts: {}", e)))?;
        invocation_counts.remove(config_id);

        Ok(())
    }

    pub fn create_deployment_package(&self, config_id: &str, version: String, package_data: &[u8]) -> Result<String, AppError> {
        let configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        if !configs.contains_key(config_id) {
            return Err(AppError::NotFound(format!("Config {} not found", config_id)));
        }
        drop(configs);

        let id = uuid::Uuid::new_v4().to_string();
        let checksum = format!("{:x}", {
            use std::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;
            let mut hasher = DefaultHasher::new();
            package_data.hash(&mut hasher);
            hasher.finish()
        });
        let package = DeploymentPackage {
            id: id.clone(),
            config_id: config_id.to_string(),
            version,
            package_size_bytes: package_data.len() as u64,
            checksum,
            deployed_at: None,
            status: "pending".to_string(),
        };

        let mut packages = self.packages.lock().map_err(|e| AppError::Internal(format!("Failed to lock packages: {}", e)))?;
        packages.insert(id.clone(), package);

        Ok(id)
    }

    pub fn deploy_package(&self, package_id: &str) -> Result<(), AppError> {
        let mut packages = self.packages.lock().map_err(|e| AppError::Internal(format!("Failed to lock packages: {}", e)))?;
        let package = packages.get_mut(package_id)
            .ok_or_else(|| AppError::NotFound(format!("Package {} not found", package_id)))?;

        package.status = "deployed".to_string();
        package.deployed_at = Some(chrono::Utc::now().timestamp());

        Ok(())
    }

    pub fn list_packages(&self, config_id: Option<&str>) -> Vec<DeploymentPackage> {
        let packages = self.packages.lock().unwrap();
        match config_id {
            Some(cid) => packages.values()
                .filter(|p| p.config_id == cid)
                .cloned()
                .collect(),
            None => packages.values().cloned().collect(),
        }
    }

    pub fn create_scaling_policy(
        &self,
        config_id: &str,
        trigger_type: String,
        threshold: f64,
        scale_up_factor: f64,
        scale_down_factor: f64,
        cooldown_seconds: u32,
    ) -> Result<String, AppError> {
        let configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        if !configs.contains_key(config_id) {
            return Err(AppError::NotFound(format!("Config {} not found", config_id)));
        }
        drop(configs);

        let id = uuid::Uuid::new_v4().to_string();
        let policy = ScalingPolicy {
            id: id.clone(),
            config_id: config_id.to_string(),
            trigger_type,
            threshold,
            scale_up_factor,
            scale_down_factor,
            cooldown_seconds,
        };

        let mut policies = self.scaling_policies.lock().map_err(|e| AppError::Internal(format!("Failed to lock policies: {}", e)))?;
        policies.insert(id.clone(), policy);

        Ok(id)
    }

    pub fn list_scaling_policies(&self, config_id: Option<&str>) -> Vec<ScalingPolicy> {
        let policies = self.scaling_policies.lock().unwrap();
        match config_id {
            Some(cid) => policies.values()
                .filter(|p| p.config_id == cid)
                .cloned()
                .collect(),
            None => policies.values().cloned().collect(),
        }
    }

    pub fn delete_scaling_policy(&self, policy_id: &str) -> Result<(), AppError> {
        let mut policies = self.scaling_policies.lock().map_err(|e| AppError::Internal(format!("Failed to lock policies: {}", e)))?;
        policies.remove(policy_id).ok_or_else(|| AppError::NotFound(format!("Scaling policy {} not found", policy_id)))?;
        Ok(())
    }

    pub fn record_invocation(&self, config_id: &str) -> Result<(), AppError> {
        let mut invocation_counts = self.invocation_counts.lock().map_err(|e| AppError::Internal(format!("Failed to lock invocation counts: {}", e)))?;
        let count = invocation_counts.entry(config_id.to_string()).or_insert(0);
        *count += 1;
        Ok(())
    }

    pub fn get_invocation_count(&self, config_id: &str) -> u64 {
        let invocation_counts = self.invocation_counts.lock().unwrap();
        *invocation_counts.get(config_id).unwrap_or(&0)
    }

    pub fn get_providers(&self) -> Vec<ServerlessProvider> {
        let providers = self.providers.lock().unwrap();
        providers.values().cloned().collect()
    }

    pub fn get_provider(&self, provider_name: &str) -> Result<ServerlessProvider, AppError> {
        let providers = self.providers.lock().map_err(|e| AppError::Internal(format!("Failed to lock providers: {}", e)))?;
        providers.get(provider_name).cloned().ok_or_else(|| AppError::NotFound(format!("Provider {} not found", provider_name)))
    }

    pub fn get_deployment_stats(&self) -> DeploymentStats {
        let _configs = self.configs.lock().unwrap();
        let packages = self.packages.lock().unwrap();
        let invocation_counts = self.invocation_counts.lock().unwrap();

        let total_invocations: u64 = invocation_counts.values().sum();
        let active_deployments = packages.values()
            .filter(|p| p.status == "deployed")
            .count();

        DeploymentStats {
            total_deployments: packages.len(),
            active_deployments,
            total_invocations,
            avg_response_time_ms: 150.0,
            error_rate: 0.01,
        }
    }

    pub fn generate_deployment_template(&self, config_id: &str, format: &str) -> Result<String, AppError> {
        let config = self.get_config(config_id)?;

        match format {
            "serverless.yml" => {
                Ok(format!(
                    r#"service: {}

provider:
  name: {}
  runtime: {}
  region: {}
  memorySize: {}
  timeout: {}

functions:
  agent:
    handler: handler.main
    environment:
{}
"#,
                    config.name,
                    config.provider,
                    config.runtime,
                    config.region,
                    config.memory_mb,
                    config.timeout_seconds,
                    config.environment_variables.iter()
                        .map(|(k, v)| format!("      {}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            "terraform" => {
                Ok(format!(
                    r#"resource "aws_lambda_function" "{}" {{
  function_name = "{}"
  runtime       = "{}"
  memory_size   = {}
  timeout       = {}
  handler       = "handler.main"

  environment {{
    variables = {{
{}
    }}
  }}
}}
"#,
                    config.name.replace("-", "_"),
                    config.name,
                    config.runtime,
                    config.memory_mb,
                    config.timeout_seconds,
                    config.environment_variables.iter()
                        .map(|(k, v)| format!("      {} = \"{}\"", k, v))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            "sam" => {
                Ok(format!(
                    r#"AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31

Resources:
  {}Function:
    Type: AWS::Serverless::Function
    Properties:
      FunctionName: {}
      Runtime: {}
      MemorySize: {}
      Timeout: {}
      Handler: handler.main
      Environment:
        Variables:
{}
"#,
                    config.name.replace("-", "_"),
                    config.name,
                    config.runtime,
                    config.memory_mb,
                    config.timeout_seconds,
                    config.environment_variables.iter()
                        .map(|(k, v)| format!("          {}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            _ => Err(AppError::Internal(format!("Unsupported template format: {}", format))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> ServerlessManager {
        ServerlessManager::new()
    }

    #[test]
    fn test_get_providers() {
        let manager = create_test_manager();
        let providers = manager.get_providers();
        assert_eq!(providers.len(), 3);
        
        let has_aws = providers.iter().any(|p| p.name == "aws");
        let has_azure = providers.iter().any(|p| p.name == "azure");
        let has_gcp = providers.iter().any(|p| p.name == "gcp");
        
        assert!(has_aws);
        assert!(has_azure);
        assert!(has_gcp);
    }

    #[test]
    fn test_create_deployment_config() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let result = manager.create_deployment_config(
            "test-deployment".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_config_invalid_region() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let result = manager.create_deployment_config(
            "test-deployment".to_string(),
            "aws".to_string(),
            "invalid-region".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_list_configs() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let _ = manager.create_deployment_config(
            "config1".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars.clone(),
        );
        let _ = manager.create_deployment_config(
            "config2".to_string(),
            "azure".to_string(),
            "eastus".to_string(),
            "python".to_string(),
            1024,
            60,
            env_vars,
        );

        let configs = manager.list_configs();
        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn test_create_and_deploy_package() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let config_id = manager.create_deployment_config(
            "test-deployment".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        ).unwrap();

        let package_data = b"test package content";
        let package_id = manager.create_deployment_package(&config_id, "1.0.0".to_string(), package_data).unwrap();
        
        let result = manager.deploy_package(&package_id);
        assert!(result.is_ok());

        let packages = manager.list_packages(Some(&config_id));
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].status, "deployed");
    }

    #[test]
    fn test_create_scaling_policy() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let config_id = manager.create_deployment_config(
            "test-deployment".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        ).unwrap();

        let policy_id = manager.create_scaling_policy(
            &config_id,
            "cpu".to_string(),
            70.0,
            2.0,
            0.5,
            300,
        ).unwrap();

        let policies = manager.list_scaling_policies(Some(&config_id));
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0].id, policy_id);
    }

    #[test]
    fn test_record_invocation() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let config_id = manager.create_deployment_config(
            "test-deployment".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        ).unwrap();

        assert_eq!(manager.get_invocation_count(&config_id), 0);
        
        manager.record_invocation(&config_id).unwrap();
        manager.record_invocation(&config_id).unwrap();
        manager.record_invocation(&config_id).unwrap();

        assert_eq!(manager.get_invocation_count(&config_id), 3);
    }

    #[test]
    fn test_get_deployment_stats() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let config_id = manager.create_deployment_config(
            "test-deployment".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        ).unwrap();

        let package_data = b"test package";
        let package_id = manager.create_deployment_package(&config_id, "1.0.0".to_string(), package_data).unwrap();
        manager.deploy_package(&package_id).unwrap();

        manager.record_invocation(&config_id).unwrap();

        let stats = manager.get_deployment_stats();
        assert_eq!(stats.total_deployments, 1);
        assert_eq!(stats.active_deployments, 1);
        assert_eq!(stats.total_invocations, 1);
    }

    #[test]
    fn test_generate_serverless_template() {
        let manager = create_test_manager();
        let mut env_vars = HashMap::new();
        env_vars.insert("ENV".to_string(), "production".to_string());
        let config_id = manager.create_deployment_config(
            "my-agent".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        ).unwrap();

        let template = manager.generate_deployment_template(&config_id, "serverless.yml").unwrap();
        assert!(template.contains("service: my-agent"));
        assert!(template.contains("runtime: python3.10"));
        assert!(template.contains("region: us-east-1"));
    }

    #[test]
    fn test_generate_terraform_template() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let config_id = manager.create_deployment_config(
            "my-agent".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        ).unwrap();

        let template = manager.generate_deployment_template(&config_id, "terraform").unwrap();
        assert!(template.contains("resource \"aws_lambda_function\""));
        assert!(template.contains("my_agent"));
    }

    #[test]
    fn test_delete_config() {
        let manager = create_test_manager();
        let env_vars = HashMap::new();
        let config_id = manager.create_deployment_config(
            "test-deployment".to_string(),
            "aws".to_string(),
            "us-east-1".to_string(),
            "python3.10".to_string(),
            512,
            30,
            env_vars,
        ).unwrap();

        let result = manager.delete_config(&config_id);
        assert!(result.is_ok());

        let configs = manager.list_configs();
        assert!(configs.is_empty());
    }
}
