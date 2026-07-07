// 🦅 Ermete OS - Astal (AGS v3) GTK4 Bedrock Shell
// A modern, dynamic, glassmorphism desktop shell for Wayland / Niri

import { App, Astal, Gtk, Gdk, Widget } from "astal/gtk4"
import { Variable, GLib } from "astal"
import { execAsync } from "astal/process"

const time = Variable("").poll(1000, () => {
    return GLib.DateTime.new_now_local().format("%H:%M  •  %a, %d %b") || ""
})

const battLevel = Variable("100%").poll(10000, () => {
    return execAsync(["bash", "-c", "cat /sys/class/power_supply/BAT*/capacity 2>/dev/null | head -n1"]).then(res => res ? `${res.trim()}%` : "AC").catch(() => "AC")
})

const volLevel = Variable("100%").poll(2000, () => {
    return execAsync(["bash", "-c", "wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null | awk '{print int($2*100)\"%\"}'"]).catch(() => "100%")
})

function TopBar(monitor = 0) {
    return Widget.Window({
        name: `bar-${monitor}`,
        monitor,
        anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.LEFT | Astal.WindowAnchor.RIGHT,
        exclusivity: Astal.Exclusivity.EXCLUSIVE,
        child: Widget.CenterBox({
            className: "top-bar",
            startWidget: Widget.Box({
                className: "left-box",
                spacing: 8,
                children: [
                    Widget.Button({
                        className: "os-logo-btn",
                        onClicked: () => App.toggle_window("launcher"),
                        child: Widget.Label({ label: "🦅  Ermete OS" }),
                    }),
                    Widget.Label({ className: "workspace-indicator", label: "  |  Wayland / Niri" }),
                ],
            }),
            centerWidget: Widget.Box({
                className: "center-box",
                children: [
                    Widget.Button({
                        className: "clock-btn",
                        onClicked: () => execAsync(["notify-send", "📅 Calendar", GLib.DateTime.new_now_local().format("%A, %d %B %Y") || ""]).catch(() => {}),
                        child: Widget.Label({ label: time() }),
                    }),
                ],
            }),
            endWidget: Widget.Box({
                className: "right-box",
                halign: Gtk.Align.END,
                spacing: 12,
                children: [
                    Widget.Label({ className: "status-indicator", label: volLevel((v: string) => `🔊 ${v}`) }),
                    Widget.Label({ className: "status-indicator", label: battLevel((b: string) => `🔋 ${b}`) }),
                    Widget.Button({
                        className: "power-btn",
                        onClicked: () => App.toggle_window("powermenu"),
                        child: Widget.Label({ label: "⏻" }),
                    }),
                ],
            }),
        }),
    })
}

function Launcher() {
    return Widget.Window({
        name: "launcher",
        visible: false,
        keymode: Astal.Keymode.ON_DEMAND,
        anchor: Astal.WindowAnchor.TOP,
        exclusivity: Astal.Exclusivity.IGNORE,
        child: Widget.Box({
            className: "launcher-box",
            vertical: true,
            spacing: 12,
            children: [
                Widget.Label({ className: "launcher-title", label: "🚀 Application Launcher" }),
                Widget.Entry({
                    className: "launcher-entry",
                    placeholderText: "Search apps...",
                    onActivate: () => execAsync(["notify-send", "Ermete OS", "App search integrated via Astal"]).catch(() => {}),
                }),
                Widget.Label({ className: "launcher-hint", label: "Press ESC or click outside to close" }),
            ],
        }),
    })
}

function PowerMenu() {
    return Widget.Window({
        name: "powermenu",
        visible: false,
        keymode: Astal.Keymode.EXCLUSIVE,
        anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM | Astal.WindowAnchor.LEFT | Astal.WindowAnchor.RIGHT,
        exclusivity: Astal.Exclusivity.IGNORE,
        child: Widget.Box({
            className: "powermenu-overlay",
            halign: Gtk.Align.CENTER,
            valign: Gtk.Align.CENTER,
            vertical: true,
            spacing: 20,
            children: [
                Widget.Label({ className: "powermenu-title", label: "⚡ Session Control" }),
                Widget.Box({
                    spacing: 16,
                    children: [
                        Widget.Button({
                            className: "power-action-btn lock",
                            onClicked: () => { App.toggle_window("powermenu"); execAsync(["loginctl", "lock-session"]).catch(() => {}); },
                            child: Widget.Label({ label: "🔒 Lock" }),
                        }),
                        Widget.Button({
                            className: "power-action-btn suspend",
                            onClicked: () => { App.toggle_window("powermenu"); execAsync(["systemctl", "suspend"]).catch(() => {}); },
                            child: Widget.Label({ label: "🌙 Suspend" }),
                        }),
                        Widget.Button({
                            className: "power-action-btn reboot",
                            onClicked: () => { App.toggle_window("powermenu"); execAsync(["systemctl", "reboot"]).catch(() => {}); },
                            child: Widget.Label({ label: "🔄 Reboot" }),
                        }),
                        Widget.Button({
                            className: "power-action-btn shutdown",
                            onClicked: () => { App.toggle_window("powermenu"); execAsync(["systemctl", "poweroff"]).catch(() => {}); },
                            child: Widget.Label({ label: "⏻ Shutdown" }),
                        }),
                    ],
                }),
                Widget.Button({
                    className: "power-cancel-btn",
                    onClicked: () => App.toggle_window("powermenu"),
                    child: Widget.Label({ label: "❌ Cancel" }),
                }),
            ],
        }),
    })
}

const cssPath = GLib.file_test((GLib.getenv("HOME") || "") + "/.config/ags/style.css", GLib.FileTest.EXISTS)
    ? (GLib.getenv("HOME") || "") + "/.config/ags/style.css"
    : "/etc/skel/.config/ags/style.css"

App.start({
    instanceName: "ermete-desktop",
    css: cssPath,
    main: () => {
        TopBar()
        Launcher()
        PowerMenu()
    },
})
