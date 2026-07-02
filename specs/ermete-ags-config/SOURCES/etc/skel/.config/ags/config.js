// 🦅 Ermete OS - AGS (Aylur's Gtk Shell) Base Config
// Inspired by top community dotfiles (e.g., end-4/dots-hyprland, Aylur/dotfiles)
// This is the OCI Bedrock default. Feel free to modify in your home folder.

const { app, Window, Box, Label, Icon } = ags;
const { execAsync } = ags.Utils;

const Clock = () => Label({
    className: "clock",
    setup: self => self.poll(1000, self => {
        execAsync(['date', '+%H:%M:%S - %A %e. %B']).then(date => {
            self.label = date;
        }).catch(console.error);
    }),
});

const TopBar = (monitor = 0) => Window({
    name: `bar-${monitor}`,
    monitor,
    anchor: ["top", "left", "right"],
    exclusivity: "exclusive",
    child: Box({
        className: "top-bar",
        children: [
            Label({ label: "🦅 Ermete OS", className: "os-logo" }),
            Box({ hexpand: true }), // Spacer
            Clock(),
            Box({ hexpand: true }), // Spacer
            Icon({ icon: "system-shutdown-symbolic", className: "power-icon" })
        ],
    }),
});

app.config({
    style: "./style.css",
    windows: [
        TopBar(),
    ],
});
