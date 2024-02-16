//! Handle initialization mode
//!
//! This mode adds the first user to the DB in case none has been registered before. This is required to register the first administrator which then has access to the Hive backend for any further customization.
//!
//! Additionally this mode is used to set the OS users and groups required to run the application
use std::sync::Arc;

use comm_types::auth::{DbUser, Role};
use dialoguer::{theme::ColorfulTheme, Input, Password};
use hive_db::BincodeDb;
use nix::unistd::{Group, User};

use crate::{
    config::AppConfig,
    database::{
        hasher,
        keys::{self, config::APP_CONFIG},
        MonitorDb,
    },
};

pub fn run_init_mode(db: Arc<MonitorDb>) {
    let users = db
        .credentials_tree
        .b_get::<Vec<DbUser>>(&keys::credentials::USERS)
        .unwrap();

    if users.is_none() || users.unwrap().is_empty() {
        // No user found, ask user to register first hive user
        add_first_user_prompt(&db);
    }

    println!("Please provide the system user name used for running Hive");
    let hive_user = system_user_prompt();

    println!("Please provide the system group name used for running Hive");
    let hive_group = system_group_prompt();

    println!("Please provide the system user name used for running the Hive runner");
    let runner_user = system_user_prompt();

    let new_app_config = AppConfig {
        hive_user,
        hive_group,
        runner_user,
    };

    db.config_tree
        .b_insert(&APP_CONFIG, &new_app_config)
        .expect("Failed to save value to DB");

    println!("Initial Hive setup successful. You can now run the monitor binary in a productive mode: 'monitor standalone' (example)");
}

/// Prompts the user to register the inial user
///
/// Automatically creates the user with the provided username & password in the Hive database
fn add_first_user_prompt(db: &MonitorDb) {
    println!(
        "In order to initialize the Hive application, please register the initial admin user:"
    );

    let username_input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Username")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.contains(' ') {
                return Err("Whitespaces are not allowed");
            }

            if input.len() < 4 {
                return Err("Username too short, minimum 4 characters are required");
            }

            Ok(())
        })
        .interact_text()
        .unwrap();

    let password_input = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Password")
        .with_confirmation("Repeat password", "Error: The passwords don't match.")
        .interact()
        .unwrap();

    let hash = hasher::hash_password(&password_input);

    let users = vec![DbUser {
        username: username_input.clone(),
        hash,
        role: Role::ADMIN,
    }];

    db.credentials_tree
        .b_insert(&keys::credentials::USERS, &users)
        .unwrap();

    println!("Successfully added user '{}', with admin role.\nYou can now restart the application in a non init-mode.", username_input);
}

/// Prompts the user for a system user
///
/// This prompt verifies if the user exists on the system OS
fn system_user_prompt() -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Username")
        .validate_with(|input: &String| -> Result<(), String> {
            match User::from_name(input).expect(
                "Failed to search system users. This is likely a system configuration issue.",
            ) {
                Some(_) => Ok(()),
                None => Err(format!("User with name {} could not be found", input)),
            }
        })
        .interact_text()
        .unwrap()
}

/// Prompts the user for a system group
///
/// This prompt verifies if the group exists on the system OS
fn system_group_prompt() -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Group name")
        .validate_with(|input: &String| -> Result<(), String> {
            match Group::from_name(input).expect(
                "Failed to search system groups. This is likely a system configuration issue.",
            ) {
                Some(_) => Ok(()),
                None => Err(format!("Group with name {} could not be found", input)),
            }
        })
        .interact_text()
        .unwrap()
}
