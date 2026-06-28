# 🌋 Ermete Forge

**The ultimate Zero-Trust, high-performance OCI Artifact Builder for Ermete OS.**

This repository acts as the sole compiler for Ermete OS. It enforces extreme CachyOS-level compiler flags (`-O3`, `-march=x86-64-v3`, `-flto=auto`, `mold` linker) across all built packages, ensuring that Ermete OS receives binaries executing at the absolute physical limit of modern silicon.

## 🏗️ Architecture: OCI-as-a-Repo
Ermete Forge does **not** rely on legacy HTTP YUM repositories (COPR). 
Instead, it utilizes GitHub Actions to compile RPM packages from source and bundles them directly into a minimalistic `scratch` OCI image (`ghcr.io/patapem/ermete-forge:latest`).

Ermete OS simply mounts this OCI image locally during its own build process to perform a Zero-Network-Failure offline installation of the packages, guaranteeing an unbreakable atomic supply chain.

## 🚀 Extreme Optimizations (The Bedrock)
All RPMs built here inherit the global `config/rpmmacros`:
- **C/C++**: `-O3 -march=x86-64-v3 -flto=auto -fuse-ld=mold`
- **Linker**: `-Wl,--as-needed -Wl,--sort-common -Wl,-O2`
- **Rust**: `-C target-cpu=x86-64-v3 -C opt-level=3 -C lto=thin`

## 📦 Assembly Lines
1. **Ring 0 (Kernel Modules)**: e.g., `akmod-nvidia` compiled against `kernel-cachyos`.
2. **Ring 3 (UX & UI)**: e.g., `aylurs-gtk-shell` (AGS).
3. **Rust Ecosystem**: e.g., `starship`, `matugen`.
4. **Security & Assets**: Custom SELinux policies (`.pp`) and cursor tarballs converted into clean RPMs.
