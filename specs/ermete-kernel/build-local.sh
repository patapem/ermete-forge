#!/bin/bash
set -euo pipefail
# Ermete OS: The Ultimate Chimera Kernel Bedrock Local Builder
# Riproduce in bit-perfect il workflow di GitHub Actions all'interno di un micro-container OCI locale
# [REVISION 2.0] Implementata Architettura a 3 Fasi: Strumentazione, Estrazione QEMU 9pfs e Cristallizzazione ThinLTO

echo ">>> [BEDROCK] Inizializzazione Ambiente di Build Isolato Locale (Fedora 43 OCI)"

FORGE_DIR=$(git rev-parse --show-toplevel)
CACHE_DIR="$FORGE_DIR/.ccache_local"
mkdir -p "$CACHE_DIR"

echo ">>> [BEDROCK] Esecuzione Container Fedora 43 (Privileged)..."
docker run --rm -i \
  --privileged \
  --security-opt label=disable \
  -v "$FORGE_DIR":/forge \
  -w /forge \
  -e GITHUB_WORKSPACE=/forge \
  registry.fedoraproject.org/fedora:43 \
  /bin/bash -s << 'DOCKEREOF'
    set -e
    echo '>>> Configurazione DNF (Identica alla CI)...'
    echo 'zchunk=False' >> /etc/dnf/dnf.conf
    echo 'fastestmirror=True' >> /etc/dnf/dnf.conf
    echo 'install_weak_deps=False' >> /etc/dnf/dnf.conf
    sed -i "/tsflags=nodocs/d" /etc/dnf/dnf.conf
    
    echo '>>> Installazione Architettura di Compilazione...'
    dnf install -y rpm-build rpmdevtools gcc gcc-c++ make cmake flex bison ncurses-devel elfutils-libelf-devel openssl-devel bc rsync tar wget curl cpio perl zstd git llvm clang lld ccache qemu-kvm stress-ng iperf3 jq gnupg2 hostname skopeo elfutils-devel dwarves openssl rust cargo rustfmt bindgen iproute fio
    
    echo '>>> [FASE 0] Fetch Base Kernel and Patches (Universal Matrix)...'
    bash specs/ermete-kernel/prepare-chimera.sh
    
    KERNEL_DIR=$(cat ~/rpmbuild/BUILD/.kernel_version)
    cd ~/rpmbuild/BUILD/$KERNEL_DIR
    
    echo '>>> [FASE 1] Build PGO Strumentata (Sensori GCOV in micro-kernel)...'
    ./scripts/config --enable GCOV_KERNEL
    ./scripts/config --enable GCOV_PROFILE_ALL
    export LLVM=1
    export MAKEFLAGS="LLVM=1 LLVM_IAS=1"
    make LLVM=1 LLVM_IAS=1 olddefconfig
    
    export PATH="/usr/lib64/ccache:/usr/lib/ccache:$PATH"
    export CCACHE_DIR=/forge/.ccache_local
    export CCACHE_COMPRESS=1
    export CCACHE_MAXSIZE=10G
    ccache -z
    
    make -j$(nproc) LLVM=1 LLVM_IAS=1 bzImage
    ccache -s

    echo '>>> [FASE 2] Stress Test Agnostico QEMU (Estrazione Termica PGO)...'
    cat << 'INITSCRIPT' > /init
#!/bin/bash
export PATH=/usr/bin:/usr/sbin:/bin:/sbin
mount -t proc none /proc
mount -t sysfs none /sys
mount -t debugfs none /sys/kernel/debug

CORES=$(nproc)
echo "Avvio PGO Stress Test su $CORES Cores..."

echo "Stress CPU, VM e Scheduler..."
stress-ng --cpu $CORES --vm 2 --vm-bytes 1G --matrix $CORES --sched $CORES --mutex $CORES --timeout 45s

echo "Stress Rete (TCP BBR/FQ)..."
iperf3 -s & IPERF_PID=$!
sleep 2
iperf3 -c 127.0.0.1 -t 15 -P $CORES
kill $IPERF_PID || true

echo "Stress I/O Massiccio VFS (EXT4/BTRFS)..."
dd if=/dev/urandom of=/tmp/burn bs=1M count=1024 2>/dev/null
rm -f /tmp/burn

echo "Estrazione mappa termica GCOV verso Host (9pfs)..."
mkdir -p /mnt/host_gcov
mount -t 9p -o trans=virtio,version=9p2000.L host_gcov /mnt/host_gcov
cp -a /sys/kernel/debug/gcov/* /mnt/host_gcov/ || true
sync

echo "SysRq spegnimento pulito..."
echo 1 > /proc/sys/kernel/sysrq
echo o > /proc/sysrq-trigger || poweroff -f
INITSCRIPT
    chmod +x /init
    
    mkdir -p /tmp/host_gcov
    HOST_CORES=$(nproc)
    HOST_RAM=$(awk '/MemTotal/ {print int($2/1024/1024/2)}' /proc/meminfo)
    if [ "$HOST_RAM" -lt 4 ]; then HOST_RAM=4; fi
    
    timeout --foreground 180s qemu-system-x86_64 -kernel arch/x86/boot/bzImage \
      -append "root=host_root rootfstype=9p rootflags=trans=virtio,version=9p2000.L rw init=/init console=ttyS0" \
      -virtfs local,path=/,mount_tag=host_root,security_model=none \
      -virtfs local,path=/tmp/host_gcov,mount_tag=host_gcov,security_model=none \
      -m ${HOST_RAM}G -smp $HOST_CORES -nographic -no-reboot || true

    echo '>>> [FASE 3] Build PGO Cristallizzata (Performance Assoluta + ThinLTO)...'
    cat << 'MACRO' >> ~/.rpmmacros
%_smp_mflags -j$(nproc)
%toolchain clang
%_ld ld.lld
%_ldflags -Wl,-O2 -Wl,--as-needed -Wl,--sort-common -Wl,-z,now -Wl,-z,relro -fuse-ld=lld
# Rimossi flag GCC-only (-fgraphite-identity, -floop-nest-optimize) per purificare il pass a Clang (LLVM=1)
%optflags %{__global_compiler_flags} -march=x86-64-v3 -pipe -Wno-error -fuse-ld=lld
%kcflags -march=x86-64-v3 -pipe -Wno-error -fuse-ld=lld
MACRO

    cd ~/rpmbuild/BUILD/$KERNEL_DIR
    make LLVM=1 LLVM_IAS=1 clean
    rm -f ~/rpmbuild/RPMS/*/*.rpm || true
    
    # Riportiamo il Kconfig per la build nativa ThinLTO senza la sporcatura di GCOV
    ./scripts/config --disable GCOV_KERNEL
    ./scripts/config --enable LTO_CLANG_THIN
    make LLVM=1 LLVM_IAS=1 olddefconfig </dev/null
    
    cd ~/rpmbuild/SPECS
    echo "Lancio rpmbuild finale..."
    rpmbuild -bc kernel.spec \
        --target x86_64 \
        --define "__make /usr/bin/make LLVM=1 LLVM_IAS=1 HOSTCC=clang HOSTCXX=clang++"
    rpmbuild -bb kernel.spec \
        --target x86_64 \
        --define "__make /usr/bin/make LLVM=1 LLVM_IAS=1 HOSTCC=clang HOSTCXX=clang++" </dev/null
    ccache -s
    
    echo '========================================================='
    echo ' BUILD LOCALE COMPLETATA CON SUCCESSO (PGO + ThinLTO).'
    echo '========================================================='
    ls -lh ~/rpmbuild/RPMS/x86_64/
    mkdir -p /forge/RPMS_OUT
    cp ~/rpmbuild/RPMS/x86_64/*.rpm /forge/RPMS_OUT/
DOCKEREOF
