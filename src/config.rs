use std::env;
use dotenv::dotenv;
pub struct Config{
    pub port: u16,
    pub host: String
}

pub fn read_config() -> Config{
    dotenv().ok();

    Config{
        port: env::var("PORT").expect("No port defined").parse().expect("Bad port definition"),
        host: env::var("PORT").expect("No host defined"),
    }
}