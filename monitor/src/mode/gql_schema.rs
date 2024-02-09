//! Outputs the SDL schema definitions of the backend graphql API

use crate::webserver::get_schema_sdl;

pub fn run_gql_schema_mode() {
    println!("{}", get_schema_sdl());
}
