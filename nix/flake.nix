{
  description = "KidneyOS";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "utils";
      };
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          overlays = [ (import rust-overlay) ];
          inherit system;
        };
        inherit (pkgs) bochs gdb gnumake grub2 mkShell mtools qemu rust-analyzer
          rust-bin unixtools xorriso;
        inherit (unixtools) xxd;
        rust = rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
        i686-cc = (import nixpkgs {
          crossSystem = "i686-linux";
          inherit system;
        }).stdenv.cc;
      in
      {
        packages.kidneyos-builder = pkgs.dockerTools.buildNixShellImage {
          name = "kidneyos-builder";
          tag = "latest";
          drv = self.devShells.${system}.build;
        };

        devShells = {
          build = mkShell {
            packages = [
              gnumake
              grub2
              i686-cc
              mtools
              qemu
              rust
              xorriso
            ];

            CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER = "${i686-cc.targetPrefix}cc";
            CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER = "${qemu}/bin/qemu-i386";
          };

          default = self.devShells.${system}.build.overrideAttrs (oldAttrs: {
            nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
              bochs
              gdb
              rust-analyzer
              xxd
            ];
          });
        };
      });
}
