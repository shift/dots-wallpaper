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
        
        echo "Running integration checks..."
        nix build .#checks.${system}.integration-test
        
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
          
          # Simple syntax check for the module
          module-syntax = pkgs.runCommand "module-syntax-check" {} ''
            # Check that our module can be imported without syntax errors
            ${pkgs.nix}/bin/nix-instantiate --parse ${./nixos-module.nix} > /dev/null
            touch $out
          '';
          
          # Basic integration test - just verify the binary works
          integration-test = pkgs.runCommand "integration-test" 
            { 
              buildInputs = [ dots-wallpaper pkgs.imagemagick ]; 
            } ''
            set -e
            
            # Create test directory
            mkdir -p test_images
            
            # Create a simple test image
            ${pkgs.imagemagick}/bin/convert -size 100x100 xc:red test_images/test.png
            
            # Test the binary with a valid image
            ${dots-wallpaper}/bin/dots-wallpaper output.png 200x200 0 test_images/test.png
            
            # Verify output was created
            test -f output.png
            
            # Test with no images (should create black canvas)  
            ${dots-wallpaper}/bin/dots-wallpaper empty.png 100x100 0
            test -f empty.png
            
            touch $out
          '';
        
          # TODO: VM tests are complex and require proper stylix integration
          # For now, we use simpler integration tests above
          # vm-test can be re-enabled when stylix dependency is properly resolved
          
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
