use std::path::Path;

pub fn is_process_running(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

pub fn filter_running_services<T, F>(services: Vec<T>, get_pid: F) -> Vec<T>
where
    F: Fn(&T) -> Option<i32>,
{
    services
        .into_iter()
        .filter(|s| {
            get_pid(s)
                .and_then(|pid| u32::try_from(pid).ok())
                .is_some_and(is_process_running)
        })
        .collect()
}
