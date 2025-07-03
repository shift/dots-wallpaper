{
  description = "A Nix flake for the dots wallpaper module and program";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let
      # Helper function to create a NixOS module with inputs
      nixosModule = { pkgs, lib, config, ... }:
        import ./nixos-module.nix {
          inherit self pkgs lib config;
          package = self.packages.${pkgs.system}.dots-wallpaper;
        };
        
      # Run all tests in one go
      runAllTests = system: pkgs: pkgs.writeShellScriptBin "run-all-tests" ''
        set -e
        echo "Running unit tests..."
        cd ${self}
        ${pkgs.cargo}/bin/cargo test
        
        echo "Running integration tests..."
        ${pkgs.cargo}/bin/cargo test --test integration_test
        
        echo "Running VM tests..."
        nix build .#checks.${system}.vm-test
        
        echo "All tests passed!"
      '';
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Get Git hash for the current repository state
        gitHash = if self ? rev then pkgs.lib.substring 0 7 self.rev else "dirty";

        # Define the Rust package itself
        dots-wallpaper = pkgs.rustPlatform.buildRustPackage rec {
          pname = "dots-wallpaper";
          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          # Set build-time environment variables
          GIT_HASH = gitHash;
          
          # Run the test suite
          doCheck = true;
          
        };
        
      in
      {
        # The default package built by `nix build`
        packages = {
          default = dots-wallpaper;
          dots-wallpaper = dots-wallpaper;
          test-runner = runAllTests system pkgs;
        };

        # Run checks for the flake
        checks = {
          # Include the package build as a check
          build = dots-wallpaper;
        };

        # Development shell for `nix develop`
        devShells.default = pkgs.mkShell {
          # Tools and libraries needed for development
          packages = [
            # Get the Rust toolchain (cargo, rustc, etc.) from the overlay
            pkgs.rust-bin.stable.latest.default
            # Include test dependencies
            pkgs.cargo-nextest
            pkgs.cargo-tarpaulin
            # Include the test runner
            self.packages.${system}.test-runner
          ];
        };
      }
    ) // {
      # NixOS module that can be imported in NixOS configurations
      nixosModules = {
        default = nixosModule;
        dots-wallpaper = nixosModule;
      };
    };
}
