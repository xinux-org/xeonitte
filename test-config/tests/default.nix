{ pkgs, diskoLib }:
let
  disk = 512 * 1024;

  make = name: extraTestScript:
    diskoLib.testLib.makeDiskoTest {
      inherit pkgs name extraTestScript;
      disko-config = ../generated/${name}.nix;
      diskSize = disk;
    };

  make2 = name: extraTestScript:
    diskoLib.testLib.makeDiskoTest {
      inherit pkgs name extraTestScript;
      disko-config = ../generated/${name}.nix;
      diskSize = disk;
      extraInstallerConfig.virtualisation.emptyDiskImages = [ disk ];
    };
in
{
  canonical-no-swap = make "canonical-no-swap" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /boot")
    machine.succeed("test -z \"$(swapon --show --noheadings)\"")
  '';

  canonical-with-swap = make "canonical-with-swap" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /boot")
    machine.succeed("swapon --show --noheadings | grep -q .")
  '';

  luks-no-swap = make "luks-no-swap" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /boot")
    machine.succeed("lsblk -o TYPE | grep -q crypt")
    machine.succeed("test -z \"$(swapon --show --noheadings)\"")
  '';

  luks-with-swap = make "luks-with-swap" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /boot")
    machine.succeed("lsblk -o TYPE | grep -q crypt")
    machine.succeed("swapon --show --noheadings | grep -q .")
  '';

  xfs-root = make "xfs-root" ''
    machine.succeed("mountpoint /")
    machine.succeed("findmnt -n -o FSTYPE / | grep -q xfs")
  '';

  f2fs-root = make "f2fs-root" ''
    machine.succeed("mountpoint /")
    machine.succeed("findmnt -n -o FSTYPE / | grep -q f2fs")
  '';

  bcachefs-root = make "bcachefs-root" ''
    machine.succeed("mountpoint /")
    machine.succeed("findmnt -n -o FSTYPE / | grep -q bcachefs")
  '';

  btrfs-subvolumes = make "btrfs-subvolumes" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /home")
    machine.succeed("mountpoint /nix")
    machine.succeed("mountpoint /var")
    machine.succeed("mountpoint /.snapshots")
    machine.succeed("findmnt -n -o FSTYPE / | grep -q btrfs")
    machine.succeed("btrfs subvolume list / | grep -q '@home'")
    machine.succeed("btrfs subvolume list / | grep -q '@nix'")
  '';

  btrfs-no-subvols = make "btrfs-no-subvols" ''
    machine.succeed("mountpoint /")
    machine.succeed("findmnt -n -o FSTYPE / | grep -q btrfs")
  '';

  luks-btrfs = make "luks-btrfs" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /home")
    machine.succeed("mountpoint /nix")
    machine.succeed("lsblk -o TYPE | grep -q crypt")
    machine.succeed("findmnt -n -o FSTYPE / | grep -q btrfs")
    machine.succeed("btrfs subvolume list / | grep -q '@home'")
  '';

  swap-random-enc = make "swap-random-enc" ''
    machine.succeed("mountpoint /")
    machine.succeed("swapon --show --noheadings | grep -q .")
  '';

  swap-discard-once = make "swap-discard-once" ''
    machine.succeed("mountpoint /")
    machine.succeed("swapon --show --noheadings | grep -q .")
  '';

  swap-discard-pages = make "swap-discard-pages" ''
    machine.succeed("mountpoint /")
    machine.succeed("swapon --show --noheadings | grep -q .")
  '';

  swap-priority = make "swap-priority" ''
    machine.succeed("mountpoint /")
    machine.succeed("swapon --show --noheadings | grep -q .")
    machine.succeed("swapon --show --noheadings --raw | awk '{print $5}' | grep -q 10")
  '';

  multi-disk-plain = make2 "multi-disk-plain" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /home")
    # root and /home must be on different block devices
    machine.succeed("[ \"$(findmnt -n -o SOURCE /)\" != \"$(findmnt -n -o SOURCE /home)\" ]")
  '';

  multi-disk-luks-root = make2 "multi-disk-luks-root" ''
    machine.succeed("mountpoint /")
    machine.succeed("mountpoint /home")
    machine.succeed("lsblk -o TYPE | grep -q crypt")
    machine.succeed("[ \"$(findmnt -n -o SOURCE /)\" != \"$(findmnt -n -o SOURCE /home)\" ]")
  '';
}
