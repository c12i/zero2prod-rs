mod config;
mod routes;
mod startup;

use routes::{health_check, subscribe};
pub use startup::run;
