#!/usr/bin/env bash
# shellcheck disable=SC2034
# ewe live/install ISO — archiso profile (derived from releng v89).

iso_name="ewe"
iso_label="EWE_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="ewe <https://github.com/prj786/ewe>"
iso_application="ewe Linux Live/Install"
iso_version="$(cat "${profile:-$(dirname "${BASH_SOURCE[0]}")}/../VERSION" 2>/dev/null || echo 0.1-alpha)"
install_dir="ewe"
buildmodes=('iso')
bootmodes=('bios.syslinux'
           'uefi.systemd-boot')
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
bootstrap_tarball_compression=('zstd' '-c' '-T0' '--auto-threads=logical' '--long' '-19')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/root/.automated_script.sh"]="0:0:755"
  ["/root/.gnupg"]="0:0:700"
  ["/usr/local/bin/choose-mirror"]="0:0:755"
  ["/usr/local/bin/Installation_guide"]="0:0:755"
  ["/usr/local/bin/livecd-sound"]="0:0:755"
  ["/usr/local/bin/ewe-live-session"]="0:0:755"
  ["/usr/local/bin/ewe-install"]="0:0:755"
  ["/etc/sudoers.d/g_ewe-live"]="0:0:440"
  ["/usr/lib/ewe-installer/ewe-install-helper"]="0:0:755"
  ["/usr/bin/ewe-installer"]="0:0:755"
  ["/etc/NetworkManager/dispatcher.d/60-ewe-auto-timezone"]="0:0:755"
)
