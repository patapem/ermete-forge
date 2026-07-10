#!/bin/bash
set -euo pipefail

REGISTRY="ghcr.io"
OWNER="${GITHUB_REPOSITORY_OWNER:-patapem}"

CUSTOM_PKGS=("starship" "bat" "selinux" "ananicy" "base-config" "desktop-ui" "ide-bootstrap" "system-services" "nix-support" "system-config" "system-tweaks" "matugen" "bibata")
CACHYOS_PKGS=("bore-sysctl" "scx-scheds" "scx-tools")
UPSTREAM_CORE=("brightnessctl" "btrfs-progs" "dbus-tools" "dbus-x11" "distribution-gpg-keys" "drm_info" "file-roller" "firewalld" "fuse" "fwupd" "gnome-keyring" "gnome-keyring-pam" "greenboot" "greenboot-default-health-checks" "gvfs" "libnotify" "libxcrypt-compat" "lm_sensors" "mokutil" "nftables" "openssl" "qemu-kvm" "sbsigntools" "squashfuse" "sysstat" "upower" "virt-manager")
UPSTREAM_DESKTOP=("niri" "adw-gtk3-theme" "fontawesome-fonts-all" "foot" "greetd" "gtk4-layer-shell" "gtk-layer-shell" "jetbrains-mono-fonts" "papirus-icon-theme" "qt5-qtwayland" "qt6-qtwayland" "rsms-inter-fonts" "swaybg", "swaylock" "thunar" "thunar-archive-plugin" "thunar-volman" "wayland-utils" "xdg-desktop-portal-gnome" "xdg-desktop-portal-gtk" "xdg-user-dirs" "xdg-user-dirs-gtk" "xorg-x11-server-Xwayland")
UPSTREAM_MEDIA=("pipewire" "nodejs" "npm" "ffmpeg", "mpv", "wireplumber", "x264", "libva-nvidia-driver", "libva-utils", "imv", "mesa-dri-drivers", "mesa-vulkan-drivers")
UPSTREAM_CLI=("btop" "eza" "fd-find" "git" "inotify-tools" "just" "nushell" "parallel" "playerctl", "ripgrep", "rsync", "sqlite", "unzip", "wl-clipboard", "wl-mirror", "wlr-randr")

# Assicuriamoci che skopeo sia installato
if ! command -v skopeo >/dev/null 2>&1; then
  sudo apt-get update && sudo apt-get install -y skopeo
fi

process_array() {
  local prefix=$1
  shift
  local pkgs=("$@")
  
  local active_pkgs=()
  
  for pkg in "${pkgs[@]}"; do
    pkg=$(echo "$pkg" | tr -d ',')
    [[ -z "$pkg" ]] && continue
    
    local image_name="ermete-forge-${prefix}${pkg}"
    
    # Esegue check_idempotency.sh
    # Esso stamperà "CACHE_HIT=true/false" in stdout
    local out
    out=$(bash scripts/check_idempotency.sh --package "$pkg" --registry "$REGISTRY" --owner "$OWNER" --image-name "$image_name" 2>/dev/null)
    
    if echo "$out" | grep -q "CACHE_HIT=false"; then
      active_pkgs+=("\"$pkg\"")
      echo "  -> MISS (will build: $pkg)" >&2
    else
      echo "  -> HIT (skip: $pkg)" >&2
    fi
  done
  
  local json="[$(IFS=,; echo "${active_pkgs[*]}")]"
  echo "$json"
}

echo "Evaluating custom_packages..." >&2
J_CUSTOM=$(process_array "" "${CUSTOM_PKGS[@]}")

echo "Evaluating cachyos_addons..." >&2
J_CACHY=$(process_array "" "${CACHYOS_PKGS[@]}")

echo "Evaluating upstream_core..." >&2
J_U_CORE=$(process_array "rolling-" "${UPSTREAM_CORE[@]}")

echo "Evaluating upstream_desktop..." >&2
J_U_DESK=$(process_array "rolling-" "${UPSTREAM_DESKTOP[@]}")

echo "Evaluating upstream_media..." >&2
J_U_MEDIA=$(process_array "rolling-" "${UPSTREAM_MEDIA[@]}")

echo "Evaluating upstream_cli..." >&2
J_U_CLI=$(process_array "rolling-" "${UPSTREAM_CLI[@]}")

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "custom_packages=${J_CUSTOM}" >> "$GITHUB_OUTPUT"
  echo "cachyos_addons=${J_CACHY}" >> "$GITHUB_OUTPUT"
  echo "upstream_core=${J_U_CORE}" >> "$GITHUB_OUTPUT"
  echo "upstream_desktop=${J_U_DESK}" >> "$GITHUB_OUTPUT"
  echo "upstream_media=${J_U_MEDIA}" >> "$GITHUB_OUTPUT"
  echo "upstream_cli=${J_U_CLI}" >> "$GITHUB_OUTPUT"
fi

echo "JSON Outputs:"
echo "custom_packages=${J_CUSTOM}"
echo "cachyos_addons=${J_CACHY}"
echo "upstream_core=${J_U_CORE}"
echo "upstream_desktop=${J_U_DESK}"
echo "upstream_media=${J_U_MEDIA}"
echo "upstream_cli=${J_U_CLI}"
