//! Handle initialization mode
//!
//! This mode adds the first user to the DB in case none has been registered before. This is required to register the first administrator which then has access to the Hive backend for any further customization.
use std::process;
use std::sync::Arc;

use comm_types::auth::{DbUser, Role};
use dialoguer::{theme::ColorfulTheme, Input, Password};

use crate::database::{hasher, keys, CborDb, HiveDb};

pub(crate) fn run_init_mode(db: Arc<HiveDb>) {
    let users = db
        .credentials_tree
        .c_get::<Vec<DbUser>>(keys::credentials::USERS)
        .unwrap();

    if users.is_some() && !users.unwrap().is_empty() {
        log::error!("Failed to run the application in init-mode. The DB already contains a registered user.\n\tHint: You can only register the first user with init-mode, if you want to register another user, please use the Hive-Backend-UI.");

        process::exit(1);
    }

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
        .c_insert(keys::credentials::USERS, &users)
        .unwrap();

    println!("Successfully added user '{}', with admin role.\nYou can now restart the application in a non init-mode.", username_input);
}
