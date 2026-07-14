use crate::core::dock_config::{get_dock_config_path, load_dock_config, DockConfig};
use crate::core::dock_data::NiriWindowInfo;
use notify::{RecursiveMode, Watcher};
use std::io::BufRead;
use std::process::Command;

pub fn fetch_current_niri_windows() -> Vec<NiriWindowInfo> {
    if let Ok(output) = Command::new("niri").args(["msg", "-j", "windows"]).output() {
        if output.status.success() {
            if let Ok(windows) = serde_json::from_slice::<Vec<NiriWindowInfo>>(&output.stdout) {
                return windows;
            }
        }
    }
    Vec::new()
}

pub fn spawn_dock_watchers(
    sender_windows: glib::Sender<Vec<NiriWindowInfo>>,
    sender_config: glib::Sender<DockConfig>,
) {
    // 1. Initial send
    let _ = sender_windows.send(fetch_current_niri_windows());
    let _ = sender_config.send(load_dock_config());

    // 2. Watch Niri event stream
    let win_sender = sender_windows.clone();
    std::thread::spawn(move || {
        loop {
            match Command::new("niri")
                .args(["msg", "-j", "event-stream"])
                .stdout(std::process::Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = std::io::BufReader::new(stdout);
                        for line in reader.lines() {
                            if let Ok(line_str) = line {
                                if line_str.contains("Window") || line_str.contains("Workspace") {
                                    let _ = win_sender.send(fetch_current_niri_windows());
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    let _ = child.wait();
                }
                Err(e) => {
                    eprintln!("Warning: niri event-stream disconnected or failed to start: {}. Retrying in 2s...", e);
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    // 3. Watch dock.json
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(_) => return,
        };
        let path = get_dock_config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
            let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
        }

        while let Ok(event) = rx.recv() {
            if let Ok(ev) = event {
                if (ev.kind.is_modify() || ev.kind.is_create())
                    && ev.paths.iter().any(|p| p.file_name() == path.file_name())
                {
                    let _ = sender_config.send(load_dock_config());
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HomeGuard {
        original: Option<String>,
    }

    impl HomeGuard {
        fn set(new_home: &std::path::Path) -> Self {
            let original = std::env::var("HOME").ok();
            std::env::set_var("HOME", new_home.to_str().unwrap());
            Self { original }
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(val) => std::env::set_var("HOME", val),
                None => std::env::remove_var("HOME"),
            }
        }
    }

    #[test]
    fn test_fetch_current_niri_windows_does_not_panic() {
        let windows = fetch_current_niri_windows();
        let _ = windows;
    }

    #[test]
    fn test_spawn_dock_watchers_initial_send() {
        let _lock = crate::core::dock_config::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp_dir = std::env::temp_dir().join("ermete_test_dock_watcher");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let _home_guard = HomeGuard::set(&tmp_dir);

        let (win_tx, win_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
        let (cfg_tx, cfg_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

        let win_received = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cfg_received = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let w_clone = win_received.clone();
        win_rx.attach(None, move |_wins| {
            w_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            glib::ControlFlow::Continue
        });

        let c_clone = cfg_received.clone();
        cfg_rx.attach(None, move |_cfg| {
            c_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            glib::ControlFlow::Continue
        });

        spawn_dock_watchers(win_tx, cfg_tx);

        let context = glib::MainContext::default();
        for _ in 0..20 {
            context.iteration(false);
            if win_received.load(std::sync::atomic::Ordering::SeqCst)
                && cfg_received.load(std::sync::atomic::Ordering::SeqCst)
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert!(win_received.load(std::sync::atomic::Ordering::SeqCst));
        assert!(cfg_received.load(std::sync::atomic::Ordering::SeqCst));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
