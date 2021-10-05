mod configuration;
mod routes;
mod startup;

use routes::{health_check, subscribe};
pub use startup::run;
pub use configuration::get_configuration;
