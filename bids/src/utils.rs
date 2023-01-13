use async_trait::async_trait;
use mockall::{automock, predicate::*};
use std::env;
use tokio::sync::{Mutex, MutexGuard};

pub fn get_env_var(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("env var \"{}\" not set", name))
}

#[automock]
#[async_trait]
pub trait Connectable {
    async fn is_connected(&self) -> bool;
    async fn connect(&mut self) -> Result<(), String>;
    async fn ping(&mut self) -> Result<(), String>;
}

pub async fn lock_connectable_mutex_safely<T: Connectable>(
    mutex: &Mutex<T>,
) -> Result<MutexGuard<T>, String> {
    let mut mutex_guard = match mutex.try_lock() {
        Ok(mutex_guard) => mutex_guard,
        Err(_) => return Err("Failed to lock mutex".to_string()),
    };
    if !mutex_guard.is_connected().await {
        println!("Establishing new connection...");
        match mutex_guard.connect().await {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        };
    } else {
        println!("Reusing connection ⚡");
        match mutex_guard.ping().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Ping failed: {}. Attempting to reconnect...", e.to_string());
                match mutex_guard.connect().await {
                    Ok(_) => (),
                    Err(e) => return Err(e.to_string()),
                };
            }
        }
    };

    Ok(mutex_guard)
}
