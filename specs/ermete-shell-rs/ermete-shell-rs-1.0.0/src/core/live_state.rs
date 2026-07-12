use std::process::Command;
use std::fs;

#[derive(Debug, Clone)]
pub struct LiveState {
    pub volume: f64,
    pub brightness: f64,
    pub ram_percent: f64,
    pub battery_percent: f64,
    pub has_battery: bool,
}

impl Default for LiveState {
    fn default() -> Self {
        Self {
            volume: 0.0,
            brightness: 0.0,
            ram_percent: 0.0,
            battery_percent: 0.0,
            has_battery: false,
        }
    }
}

pub fn get_live_state() -> LiveState {
    let mut state = LiveState::default();

    // Volume
    if let Ok(output) = Command::new("wpctl")
        .arg("get-volume")
        .arg("@DEFAULT_AUDIO_SINK@")
        .output()
    {
        if let Ok(out_str) = String::from_utf8(output.stdout) {
            let parts: Vec<&str> = out_str.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == "Volume:" {
                if let Ok(vol) = parts[1].parse::<f64>() {
                    state.volume = vol;
                }
            }
        }
    }

    // Brightness
    if let Ok(output) = Command::new("brightnessctl")
        .arg("-m")
        .output()
    {
        if let Ok(out_str) = String::from_utf8(output.stdout) {
            let parts: Vec<&str> = out_str.trim().split(',').collect();
            if parts.len() >= 4 {
                let percent_str = parts[3].trim_end_matches('%');
                if let Ok(bright) = percent_str.parse::<f64>() {
                    state.brightness = bright;
                }
            }
        }
    }

    // RAM
    if let Ok(output) = Command::new("free")
        .output()
    {
        if let Ok(out_str) = String::from_utf8(output.stdout) {
            let lines: Vec<&str> = out_str.lines().collect();
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "Mem:" {
                    if let (Ok(total), Ok(used)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                        if total > 0.0 {
                            state.ram_percent = (used / total) * 100.0;
                        }
                    }
                }
            }
        }
    }

    // Battery
    if let Ok(bat_str) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
        if let Ok(bat) = bat_str.trim().parse::<f64>() {
            state.battery_percent = bat;
            state.has_battery = true;
        }
    } else if let Ok(bat_str) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity") {
        if let Ok(bat) = bat_str.trim().parse::<f64>() {
            state.battery_percent = bat;
            state.has_battery = true;
        }
    }

    state
}
