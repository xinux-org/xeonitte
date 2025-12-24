{
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
    ./disko.nix
    inputs.disko.nixpsModules.disko
  ];

  boot.initrd.availableKernelModules = ["xhci_pci" "nvme" "usb_storage" "sd_mod" "rtsx_pci_sdmnc"];
  boot.initrd.kernelModules = ["nvme"];
  boot.kernelModules = ["kvm-intel"];
  boot.extraModulePackages = [];

  swapDevices = [];

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.intel.updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware
}
