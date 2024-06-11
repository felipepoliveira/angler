use std::{collections::{HashMap, HashSet}, error, fmt::Display, fs, path::Path};

use thiserror::Error;
use time::Duration;

use crate::utils::time::{DurationDeserializer, DurationSequence, DurationSequenceDeserializer};

/// Store cluster configurations nominated by `cluster.` prefix
#[derive(Debug)]
pub struct ClusterConfiguration {
    /// The authentication token used to authenticate brokers into the cluster
    pub auth_key: Option<String>,
    
    /// The host for the controller instance
    pub controller_host: Option<String>,

    /// The duration that a request should have to be sent until it is considered a timeout
    pub request_timeout: Option<Duration>,
}

impl ClusterConfiguration {
    fn new() -> ClusterConfiguration {
        ClusterConfiguration {
            auth_key: None,
            controller_host: None,
            request_timeout: None
        }
    }
}

/// Store database configurations nominated by `db.` prefix
#[derive(Debug)]
pub struct DatabaseConfigurations {
    /// The amount of time where dead messages will be stored until it will be deleted
    dead_messages_retention: Option<Duration>,

    /// The amount of time where delivered messages will be stored until it will be deleted
    delivered_messages_retention: Option<Duration>,
}

impl DatabaseConfigurations {
    fn new() -> DatabaseConfigurations {
        DatabaseConfigurations {
            dead_messages_retention: None,
            delivered_messages_retention: None
        }
    }
}

/// Store configurations about the message processor nominated by `msgproc.` prefix
#[derive(Debug)]
pub struct MessagesProcessorConfigurations {
    /// The duration that a message should wait in delivery process until it is considered a timeout
    message_delivery_timeout: Option<Duration>,

    /// The amount of workers that the broker should make available to send messages
    workers_count: Option<usize>,
}

impl MessagesProcessorConfigurations {
    fn new() -> MessagesProcessorConfigurations {
        MessagesProcessorConfigurations {
            message_delivery_timeout: None,
            workers_count: None
        }
    }
}

/// Store networking configuration nominated by `net.` prefix
#[derive(Debug)]
pub struct NetworkingConfiguration {
    /// The protocols that will be opened to the client API. Supported values are: `restful`
    client_protocols: Option<HashSet<String>>,

    /// The port that will be used to expose the RESTFul API when set in `net.client.protocols` config.
    restful_port: Option<u32>,
}

impl NetworkingConfiguration {
    fn new() -> NetworkingConfiguration {
        NetworkingConfiguration {
            client_protocols: None,
            restful_port: None,
        }
    }
}

#[derive(Debug)]
pub struct RetryPolicyConfiguration { 
    /// The default interval duration that will be applied when a message sent by a client
    /// does not have a config defined.
    pub default_interval: Option<DurationSequence>,

    /// The default max attempts that will be applied when a message sent by a client
    /// does not have a config defined
    pub default_max_attempts: Option<u16>,

    /// The limit of a max interval. This configuration will be used to restrict a maximum amount of interval
    /// that could be applied for a message sent by the client
    pub max_interval_limit: Option<Duration>,

    /// The maximum amount of attempts that a client could define in a sent message
    pub max_attempts_limit: Option<u16>,
}

impl RetryPolicyConfiguration {
    fn new() -> RetryPolicyConfiguration {
        RetryPolicyConfiguration {
            default_interval: None,
            default_max_attempts: None,
            max_attempts_limit: None,
            max_interval_limit: None,
        }
    }
}

#[derive(Debug, Error)]
enum ConfigurationErrorCauses {
    #[error("An error occur while trying to read the configuration file")]
    FailedToReadConfigurationFile
}

#[derive(Debug, Error)]
pub struct ConfigurationError {
    cause: ConfigurationErrorCauses,
    reason: String,
}

impl Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An {} error occur. Error message: {}. Details: {}", self.cause, self.cause.to_string(), self.reason)
    }
}

#[derive(Debug)]
pub struct Configuration {
    /// Configurations for angler cluster defined by `cluster.` prefix
    pub cluster: ClusterConfiguration,
    /// Configuration for messages database defined by `db.` prefix
    pub database: DatabaseConfigurations,
    /// Configuration applied to messages processor defined by `msgproc.` prefix
    pub messages_processor: MessagesProcessorConfigurations,
    /// Configuration for networking defined by `net.` prefix
    pub networking: NetworkingConfiguration,
    /// Configuration for retry policy defined by `retryPolicy.` prefix
    pub retry_policy: RetryPolicyConfiguration,
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration {
            cluster: ClusterConfiguration::new(),
            database: DatabaseConfigurations::new(),
            messages_processor: MessagesProcessorConfigurations::new(),
            networking: NetworkingConfiguration::new(),
            retry_policy: RetryPolicyConfiguration::new(),
        }
    }

    pub fn from_map(map: &HashMap<String, String>) -> Configuration {
        let mut configuration = Configuration::new();
        
        // cluster.
        configuration.cluster.auth_key = map.get("cluster.authKey").and_then(|v| Some(v.clone()));
        configuration.cluster.controller_host = map.get("cluster.controller.host").and_then(|v| Some(v.clone()));
        configuration.cluster.request_timeout = map.get("cluster.requestTimeout").and_then(|v| 
            Some(Duration::milliseconds(v.parse().expect("cluster.requestTimeout should be a time in milliseconds >= 0")))
        );
        
        // db.
        configuration.database.dead_messages_retention = map.get("db.deadMessages.retention").and_then(|v| 
            Some(v.as_str().to_duration().expect("db.deadMessages.retention has a invalid syntax for Duration"))
        );
        configuration.database.delivered_messages_retention = map.get("db.deliveredMessages.retention").and_then(|v| 
            Some(v.as_str().to_duration().expect("db.deliveredMessages.retention has a invalid syntax for Duration")))
        ;

        // msgproc.
        configuration.messages_processor.message_delivery_timeout = map.get("msgproc.message_delivery_timeout").and_then(|v| 
            Some(Duration::milliseconds(v.parse().expect("msgproc.message_delivery_timeout should be in milliseconds")))
        );
        configuration.messages_processor.workers_count = map.get("msgproc.workers").and_then(|v| 
            Some(v.parse().expect("msgproc.workers should be integer >= 1"))
        );

        // net.
        configuration.networking.client_protocols = map.get("net.client.protocols").and_then(|v| 
            Some(v.split(',').map(|v| String::from(v.trim())).collect()) //split("a, b") and transform it into Set["a", "b"]
        );
        configuration.networking.restful_port = map.get("net.client.restful.port").and_then(|v| 
            Some(v.parse().expect("net.client.restful.port should be a integer >= 1"))
        );

        // retryPolicy.defaults.
        configuration.retry_policy.default_interval = map.get("retryPolicy.defaults.interval").and_then(|v|
            Some(v.as_str().to_duration_sequence().expect("retryPolicy.defaults.interval should have a valid DurationSequence syntax. Example: [1m, 5m, 1d]"))
        );
        configuration.retry_policy.default_max_attempts = map.get("retryPolicy.defaults.maxAttempts").and_then(|v| 
            Some(v.parse().expect("retryPolicy.defaults.maxAttempts should be a valid integer >= 0"))
        );
        // retryPolicy.limit.
        configuration.retry_policy.max_interval_limit = map.get("retryPolicy.limit.maxInterval").and_then(|v|
            Some(v.as_str().to_duration().expect("retryPolicy.limit.maxInterval should have a valid Duration syntax. Example: 30m"))
        );
        configuration.retry_policy.max_attempts_limit = map.get("retryPolicy.limit.maxAttempts").and_then(|v|
            Some(v.parse().expect("retryPolicy.limit.maxAttempts should be a integer >= 1"))
        );

        configuration
    }

    /// Create a instance of Configuration based on the content of the file plus merging with the value
    /// of the environment variable `ANGLER_CFG` that should be in a format like 
    pub fn from_properties_file<P: AsRef<Path>>(file_path: P) -> Result<Configuration, ConfigurationError> {

        // read the file content and return error if it fails
        let file_content = match fs::read_to_string(file_path) {
            Ok(file_content) => file_content,
            Err(err) => return Err(ConfigurationError { 
                cause: ConfigurationErrorCauses::FailedToReadConfigurationFile,
                reason: err.to_string()
            })
        };

        let configuration = Configuration::from_map(&properties_file_content_to_map(file_content.as_str()));

        Ok(configuration)
    }

    pub fn merge(&mut self, other: &Configuration) {
         // Merge ClusterConfiguration
         if self.cluster.auth_key.is_none() {
            self.cluster.auth_key = other.cluster.auth_key.clone();
        }
        if self.cluster.controller_host.is_none() {
            self.cluster.controller_host = other.cluster.controller_host.clone();
        }
        if self.cluster.request_timeout.is_none() {
            self.cluster.request_timeout = other.cluster.request_timeout;
        }

        // Merge DatabaseConfigurations
        if self.database.dead_messages_retention.is_none() {
            self.database.dead_messages_retention = other.database.dead_messages_retention;
        }
        if self.database.delivered_messages_retention.is_none() {
            self.database.delivered_messages_retention = other.database.delivered_messages_retention;
        }

        // Merge MessagesProcessorConfigurations
        if self.messages_processor.message_delivery_timeout.is_none() {
            self.messages_processor.message_delivery_timeout = other.messages_processor.message_delivery_timeout;
        }
        if self.messages_processor.workers_count.is_none() {
            self.messages_processor.workers_count = other.messages_processor.workers_count;
        }

        // Merge NetworkingConfiguration
        if self.networking.client_protocols.is_none() {
            self.networking.client_protocols = other.networking.client_protocols.clone();
        }
        if self.networking.restful_port.is_none() {
            self.networking.restful_port = other.networking.restful_port;
        }

        // Merge RetryPolicyConfiguration
        if self.retry_policy.default_interval.is_none() {
            self.retry_policy.default_interval = other.retry_policy.default_interval.clone();
        }
        if self.retry_policy.default_max_attempts.is_none() {
            self.retry_policy.default_max_attempts = other.retry_policy.default_max_attempts;
        }
        if self.retry_policy.max_interval_limit.is_none() {
            self.retry_policy.max_interval_limit = other.retry_policy.max_interval_limit;
        }
        if self.retry_policy.max_attempts_limit.is_none() {
            self.retry_policy.max_attempts_limit = other.retry_policy.max_attempts_limit;
        }
    }
}

/// Parse a properties file content into a HashMap<String, String>. Here is a example of properties file:
/// ```properties
/// akey=avalue
/// bkey=bvalue
/// ckey=cvalue
/// ```
pub fn properties_file_content_to_map(file_content: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for line in file_content.split('\n') {
        let line = line.trim();

        // ignore comments # foo bar
        if line.trim().starts_with('#') {
            continue
        }
        
        // split key and value
        let key_and_value: Vec<&str> = line.split('=').collect();

        // skip if there is less than 2 elements (that means that the line does not have a '=' separator)
        if key_and_value.len() < 2 {
            continue;
        }

        // insert the [0] as the key and all the other values will be joined into a single value
        map.insert(key_and_value[0].to_string(), key_and_value[1..].join("=").trim().to_string());
    }

    map
}

/// Parse a properties separate by semicolon into a HashMap<String, String>. Here is a example of properties file:
/// `akey=avalue; bkey=bvalue; ckey=cvalue`
pub fn properties_separate_by_semicolon_to_map(content: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for part in content.split(';') {
        let part = part.trim();

        // ignore comments # foo bar
        if part.trim().starts_with('#') {
            continue
        }
        
        // split key and value
        let key_and_value: Vec<&str> = part.split('=').collect();

        // skip if there is less than 2 elements (that means that the line does not have a '=' separator)
        if key_and_value.len() < 2 {
            continue;
        }

        // insert the [0] as the key and all the other values will be joined into a single value
        map.insert(key_and_value[0].to_string(), key_and_value[1..].join("=").trim().to_string());
    }

    map
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::{properties_file_content_to_map, properties_separate_by_semicolon_to_map, Configuration};

    const TEST_CONF_PROPERTIES_FILE: &'static str  =r#"

# Cluster configurations
cluster.authKey=abcd1234
cluster.controller.host=webhooks.my-web.services
cluster.requestTimeout=10000

# Database properties
db.deadMessages.retention=30d
db.deliveredMessages.retention=30d

# Message Processor configurations
msgproc.message_delivery_timeout=10000
msgproc.workers=500

# Configuration about the client net communication interface
net.client.protocols=restful
net.client.restful.port=80

# The default values set on retryPolicy if not set by the client
retryPolicy.defaults.interval=1d
retryPolicy.defaults.maxAttempts=7

# The limit (max or min) of interval and resend attempts
retryPolicy.limit.maxInterval=30d
retryPolicy.limit.maxAttempts=20
    
    "#;

    const TEST_CONF_PROPERTIES_FILE_SEMICOLON: &'static str = "
cluster.authKey=abcd1234;
cluster.controller.host=webhooks.my-web.services;
cluster.requestTimeout=10000;
db.deadMessages.retention=30d;
db.deliveredMessages.retention=30d;
msgproc.message_delivery_timeout=10000;
msgproc.workers=500;
net.client.protocols=restful;
net.client.restful.port=80;
retryPolicy.defaults.interval=1d;
retryPolicy.defaults.maxAttempts=7;
retryPolicy.limit.maxInterval=30d;
retryPolicy.limit.maxAttempts=20;
";

    fn assert_configuration_has_all_props(conf: &Configuration) {
        assert_eq!(conf.cluster.auth_key.as_ref().unwrap(), "abcd1234");
        assert_eq!(conf.cluster.controller_host.as_ref().unwrap(), "webhooks.my-web.services");
        assert_eq!(conf.cluster.request_timeout.unwrap().whole_milliseconds(), 10000);

        assert_eq!(conf.database.dead_messages_retention.unwrap().whole_days(), 30);
        assert_eq!(conf.database.delivered_messages_retention.unwrap().whole_days(), 30);

        assert_eq!(conf.messages_processor.message_delivery_timeout.unwrap().whole_milliseconds(), 10000);
        assert_eq!(conf.messages_processor.workers_count.unwrap(), 500);

        assert!(conf.networking.client_protocols.as_ref().unwrap().contains("restful"));
        assert_eq!(conf.networking.restful_port.unwrap(), 80);

        assert_eq!(conf.retry_policy.default_interval.as_ref().unwrap().total_duration().whole_days(), 1);
        assert_eq!(conf.retry_policy.default_max_attempts.unwrap(), 7);

        assert_eq!(conf.retry_policy.max_interval_limit.unwrap().whole_days(), 30);
        assert_eq!(conf.retry_policy.max_attempts_limit.unwrap(), 20);
    }

    #[test]
    fn test_properties_file_content_to_map_from_a_config_file() {
        let map = properties_file_content_to_map(&TEST_CONF_PROPERTIES_FILE);
        assert_eq!(map.get("cluster.authKey").unwrap(), "abcd1234");
        assert_eq!(map.get("cluster.controller.host").unwrap(), "webhooks.my-web.services");
        assert_eq!(map.get("cluster.requestTimeout").unwrap(), "10000");

        assert_eq!(map.get("db.deadMessages.retention").unwrap(), "30d");
        assert_eq!(map.get("db.deliveredMessages.retention").unwrap(), "30d");

        assert_eq!(map.get("msgproc.message_delivery_timeout").unwrap(), "10000");
        assert_eq!(map.get("msgproc.workers").unwrap(), "500");

        assert_eq!(map.get("net.client.protocols").unwrap(), "restful");
        assert_eq!(map.get("net.client.restful.port").unwrap(), "80");

        assert_eq!(map.get("retryPolicy.defaults.interval").unwrap(), "1d");
        assert_eq!(map.get("retryPolicy.defaults.maxAttempts").unwrap(), "7");

        assert_eq!(map.get("retryPolicy.limit.maxInterval").unwrap(), "30d");
        assert_eq!(map.get("retryPolicy.limit.maxAttempts").unwrap(), "20");
    }

    #[test]
    fn test_if_all_configurations_are_set_in_configuration_struct_from_hash_map() {
        let conf = Configuration::from_map(&properties_file_content_to_map(&TEST_CONF_PROPERTIES_FILE));
        assert_configuration_has_all_props(&conf);
    }

    #[test]
    fn test_if_all_configurations_are_set_in_semicolon_conf_string_from_hash_map() {
        let conf = Configuration::from_map(&properties_separate_by_semicolon_to_map(&TEST_CONF_PROPERTIES_FILE_SEMICOLON));
        assert_configuration_has_all_props(&conf);
    }

    #[test]
    fn test_if_configurations_are_correctly_loaded_from_file() {
        let conf = Configuration::from_properties_file("./src/dev/tests/resources/config.properties").unwrap();
        assert_configuration_has_all_props(&conf);
    }

    #[test]
    fn test_configuration_merge() {
        let mut will_be_merged_conf = Configuration::new();
        let conf_with_all_fields = Configuration::from_properties_file("./src/dev/tests/resources/config.properties").unwrap();
        will_be_merged_conf.merge(&conf_with_all_fields);

        // ClusterConfiguration assertions
        assert_ne!(will_be_merged_conf.cluster.auth_key, None);
        assert_ne!(will_be_merged_conf.cluster.controller_host, None);
        assert_ne!(will_be_merged_conf.cluster.request_timeout, None);

        // DatabaseConfigurations assertions
        assert_ne!(will_be_merged_conf.database.dead_messages_retention, None);
        assert_ne!(will_be_merged_conf.database.delivered_messages_retention, None);

        // MessagesProcessorConfigurations assertions
        assert_ne!(will_be_merged_conf.messages_processor.message_delivery_timeout, None);
        assert_ne!(will_be_merged_conf.messages_processor.workers_count, None);

        // NetworkingConfiguration assertions
        assert_ne!(will_be_merged_conf.networking.client_protocols, None);
        assert_ne!(will_be_merged_conf.networking.restful_port, None);

        // RetryPolicyConfiguration assertions
        assert_ne!(will_be_merged_conf.retry_policy.default_interval, None);
        assert_ne!(will_be_merged_conf.retry_policy.default_max_attempts, None);
        assert_ne!(will_be_merged_conf.retry_policy.max_interval_limit, None);
        assert_ne!(will_be_merged_conf.retry_policy.max_attempts_limit, None);
    }
}