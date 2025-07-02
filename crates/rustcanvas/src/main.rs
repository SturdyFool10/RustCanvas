use appstate::AppState;
use config::load_config;
use db::DatabaseConnection;
use macros::spawn_tasks;
use prettylogs::init_logging;
use std::{error::Error, path::Path};
use tokio::{select, task::JoinHandle};
use tracing::*;
use webserver::start_webserver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging first so all subsequent logs are captured
    init_logging();
    info!("RustCanvas starting up");
    let conf = load_config("config");
    debug!("Configuration loaded");
    info!("Attempting to load Database...");
    let pathstr = conf.database_path.clone();
    let path = Path::new(&pathstr);
    let db = DatabaseConnection::new(path)?;

    let state: AppState = AppState::new(conf, db);
    let handles: Vec<JoinHandle<()>> = spawn_tasks!(state.clone(), start_webserver);
    // Wait for any task to complete, which means it failed, all of my tasks exit on failure only
    if !handles.is_empty() {
        select! {
            (completed_task, index, _) = futures::future::select_all(handles) => {
                match completed_task {
                    Ok(_) => {
                        error!("Task {} completed unexpectedly. Tasks should run indefinitely.", index);
                    }
                    Err(err) => {
                        error!("Task {} terminated with an error: {:?}", index, err);
                    }
                }
            }
            else => {
                info!("All tasks completed successfully");
            }
        }
    }
    Ok(())
}
