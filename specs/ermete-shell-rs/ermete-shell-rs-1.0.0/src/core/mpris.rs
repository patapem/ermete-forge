use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MprisState {
    pub title: String,
    pub artist: String,
    pub status: String,
}

pub fn get_mpris_state() -> Option<MprisState> {
    let status_output = Command::new("playerctl")
        .arg("status")
        .output()
        .ok()?;

    if !status_output.status.success() {
        return None;
    }

    let status = String::from_utf8_lossy(&status_output.stdout).trim().to_string();

    let title_output = Command::new("playerctl")
        .arg("metadata")
        .arg("xesam:title")
        .output()
        .ok()?;

    let title = String::from_utf8_lossy(&title_output.stdout).trim().to_string();

    let artist_output = Command::new("playerctl")
        .arg("metadata")
        .arg("xesam:artist")
        .output()
        .ok()?;

    let artist = String::from_utf8_lossy(&artist_output.stdout).trim().to_string();

    Some(MprisState {
        title,
        artist,
        status,
    })
}
