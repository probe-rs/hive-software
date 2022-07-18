//! Various validators used to validate user input
use validators::models::Host;
use validators::prelude::{validators_prelude, ValidateString, Validator};

#[derive(Debug, Validator)]
#[validator(host(local(Allow), port(Allow), at_least_two_labels(Must)))]
pub struct ValidHost {
    pub host: Host,
    pub port: Option<u16>,
    #[allow(dead_code)]
    pub is_local: bool,
}

pub fn ip_or_url(arg: &str) -> Result<ValidHost, String> {
    match ValidHost::parse_str(arg) {
        Ok(valid) => Ok(valid),
        Err(_) => Err("Invalid testserver address".to_owned()),
    }
}
