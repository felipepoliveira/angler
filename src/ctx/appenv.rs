use std::{collections::HashSet, env, fmt::format, fs, path::{self, Path}, sync::OnceLock};

use clap::{Arg, ArgMatches, Command};

use crate::ctx::config::properties_separate_by_semicolon_to_map;

use super::config::Configuration;

/**
 * Create a thread-safe instance of Command that contains all
 * arguments, flags and parameter of the application
 */
pub fn app_args() -> &'static ArgMatches {
    static COMMAND: OnceLock<ArgMatches> = OnceLock::new();
    COMMAND.get_or_init(|| {
        Command::new("angler")
            .version("0.1")
            .arg(
                Arg::new("dev")
                    .long("dev")
                    .help("Indicates that the application is running on Development context")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("controller")
                    .long("controller")
                    .short('c')
                    .help("Indicates that this instance of node is the Controller instance")
                    .action(clap::ArgAction::SetTrue)
            )
            .get_matches()
    })
}

/// Return a thread-safe static reference of the AppEnvironment instance
pub fn app_env() -> &'static AppEnvironment {
    static APP_ENV: OnceLock<AppEnvironment> = OnceLock::new();
    APP_ENV.get_or_init(|| {
        let app_args = app_args();
        let context = match app_args.get_flag("dev") {
            true => AppContexts::Development,
            false => AppContexts::Production
        };
        
        // loading configuration from configuration file
        println!("{:?}", std::env::current_dir().unwrap());
        let path_to_conf_file = context.path_to_conf_file();
        println!("Loading configuration file from: {:?}", path_to_conf_file);
        let mut configuration = Configuration::from_properties_file(path_to_conf_file).unwrap();
        
        // merging with conf from environment variable
        if let Ok(env_var_value) = env::var("ANGLER_CFG") {
            println!("ANGLER_CFG environment variable found with value: {}", env_var_value);
            configuration.merge(&Configuration::from_map(&properties_separate_by_semicolon_to_map(&env_var_value)));
        }
        else {
            println!("ANGLER_CFG environment variable not found");
        }

        AppEnvironment { context, configuration }
    })
}

/// Store the context where the application is running
#[derive(Debug)]
pub enum AppContexts {
    /// For the app to run in development mode it should receive --dev flag
    Development,
    /// By default, the app always run on production mode
    Production,
}

/// Default implementations for AppContexts
impl AppContexts {
    pub fn path_to_conf_file(&self) -> String {
        match self {
            AppContexts::Development => format!("{}/src/dev/tests/resources/config.properties",  std::env::current_dir().unwrap().display()),
            AppContexts::Production => String::from("./conf/config.properties"),
        }
    }
}

/// The application roles defines witch functionalities will be made by the node
#[derive(Debug)]
pub enum ApplicationRoles {
    MessageProcessor,
    Storage,
}

/// Identify witch role this application will have in the Cluster
pub enum NodeType {
    /// This instance runs as a broker so it never expose an direct API to connection
    /// and will handshake with controller to make the application load-balance
    Broker,

    Controller,
}

#[derive(Debug)]
pub struct AppEnvironment {
    /// Store the configuration used in the application
    configuration: Configuration,
    /// Store the context where the app is currently executing
    context: AppContexts,
    /// Store all the roles that this application will have
    roles: HashSet<ApplicationRoles>,
}

impl AppEnvironment {
    /// Return a thread-safe static reference of the AppEnvironment instance
    pub fn get() -> &'static AppEnvironment {
        app_env()
    }

    
}

