{
  description = "A development environment for Rust projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixgl.url = "github:guibou/nixGL"; # Allows you to run OpenGL and or Vulkan applications in a nix shell
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    nixgl,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay) nixgl.overlay];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-edit
            gcc
            alsa-lib
            cmake
            libGL
            libudev-zero
            libxkbcommon
            automake
            autoconf
            perl
            pkg-config
            wayland
          ];

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.libxkbcommon}/lib:$LD_LIBRARY_PATH
            export LD_LIBRARY_PATH=${pkgs.libGL}/lib:$LD_LIBRARY_PATH

            # Workaround for "Failed to set cursor to Default"
            # https://github.com/bevyengine/bevy/issues/4768
            export XCURSOR_THEME=Adwaita
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ocean";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          nativeBuildInputs = [
            pkgs.alsa-lib
            pkgs.pkg-config
            pkgs.perl
            pkgs.cmake
          ];
          buildInputs = [
            pkgs.alsa-lib
            pkgs.libGL
            pkgs.libxkbcommon
            pkgs.libudev-zero
            pkgs.wayland
          ];
        };
      }
    );
}
