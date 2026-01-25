{
  description = "Tern: Modular batch conversion interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.nightly."2026-01-21".default;

        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustVersion;
          cargo = rustVersion;
        };

        tern = rustPlatform.buildRustPackage {
          pname = "tern";
          version = "1.7.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustVersion
            cmake
          ];

          buildInputs = with pkgs; [
            lua5_4
            sqlite
            openssl
            fontconfig
            wayland
            libxkbcommon
            libGL
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
          ];

          # Slint needs some environment variables during build if it can't find libraries
          # We might need to set SLINT_STYLE or similar if required.
        };
      in
      {
        packages.default = tern;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustVersion
            pkgs.pkg-config
            pkgs.lua5_4
            pkgs.sqlite
            pkgs.openssl
            pkgs.cmake
          ] ++ (with pkgs; [
            fontconfig
            wayland
            libxkbcommon
            libGL
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
          ]);

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath (with pkgs; [
              wayland
              libxkbcommon
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              fontconfig
            ])}
          '';
        };
      }
    );
}
