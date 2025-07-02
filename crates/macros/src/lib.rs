#[macro_export]
macro_rules! spawn_tasks {
    ($state:expr, $($func:expr),* $(,)?) => {{
        let mut handles = Vec::new();
        $(
            let state = $state.clone();
            handles.push(tokio::spawn($func(state)));
        )*
        let task_count = handles.len();
        tracing::info!("Spawned {} {}", task_count, if task_count == 1 { "task" } else { "tasks" });
        handles
    }};
}
