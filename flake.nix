{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          inherit (pkgs) lib;
          # cargo-component nixos package is available, just stuck on 0.20.0
          cargoComponent = pkgs.rustPlatform.buildRustPackage rec {
            pname = "cargo-component";
            version = "0.21.1";

            src = pkgs.fetchFromGitHub {
              owner = "bytecodealliance";
              repo = "cargo-component";
              rev = "v${version}";
              hash = "sha256-Tlx14q/2k/0jZZ1nECX7zF/xNTeMCZg/fN+fhRM4uhc=";
            };

            useFetchCargoVendor = true;
            cargoHash = "sha256-ZwxVhoqAzkaIgcH9GMR+IGkJ6IOQVtmt0qcDjdix6cU=";

            nativeBuildInputs = [
              pkgs.pkg-config
            ];

            buildInputs = [
              pkgs.openssl
            ];

            # requires the wasm32-wasi target
            doCheck = false;

            meta = with lib; {
              description = "Cargo subcommand for creating WebAssembly components based on the component model proposal";
              homepage = "https://github.com/bytecodealliance/cargo-component";
              changelog = "https://github.com/bytecodealliance/cargo-component/releases/tag/${src.rev}";
              license = licenses.asl20;
              maintainers = with maintainers; [ figsoda ];
              mainProgram = "cargo-component";
            };
          };
          # TODO: submit wkg to nix-packages
          wkg = pkgs.rustPlatform.buildRustPackage rec {
            pname = "wkg";
            version = "0.10.0";

            src = pkgs.fetchFromGitHub {
              owner = "bytecodealliance";
              repo = "wasm-pkg-tools";
              rev = "v${version}";
              hash = "sha256-VZ+rUZi6o2onMFxK/BMyi6ZjuDS0taJh5w3r33KCZTU=";
            };
            
            useFetchCargoVendor = true;
            cargoHash = "sha256-dHhJT/edEYagLQoUcXCLPA4fUJdN9ZoOITLpWAH5p/0=";

            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];

            doCheck = false;

            meta = with lib; {
              description = "WebAssembly Kit Generator - tools for working with WebAssembly components";
              homepage = "https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wasm-package-cli";
              license = licenses.asl20;
              mainProgram = "wkg";
            };
          };
          wkgConfigFile = pkgs.writeText "config.toml" ''
            default_registry = "wa.dev"
          '';
          wkgConfigHook = ''
            mkdir -p ~/.config/wasm-pkg
            cp ${wkgConfigFile} ~/.config/wasm-pkg/config.toml
          '';
          craneLib = (crane.mkLib pkgs).overrideToolchain (
            p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
          );
          buildInputs = with pkgs; [
            # Dev tools
            just
            cargoComponent
            wkg

            # Build tools
            pkg-config
          ] ++ lib.optionals stdenv.isLinux [
            alsa-lib
            libxkbcommon
            udev
            vulkan-loader
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk_11_0.frameworks.Cocoa
            rustPlatform
          ];
        in
        {
          devShells.default = craneLib.devShell {
            inherit buildInputs;

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
            
            shellHook = wkgConfigHook;
          };
        }
      );
}