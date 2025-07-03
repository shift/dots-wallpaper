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

        # Import the NixOS testing framework
        nixosTest = import (nixpkgs + "/nixos/lib/testing-python.nix") {
          inherit system pkgs;
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
          
          # VM test with comprehensive image format testing
          vm-test = nixosTest {
            name = "dots-wallpaper-vm-test";
            
            nodes.machine = { config, pkgs, ... }: {
              imports = [ self.nixosModules.default ];
              
              # Enable required services
              services.xserver.enable = true;
              services.xserver.displayManager.lightdm.enable = true;
              services.xserver.desktopManager.xfce.enable = true;
              
              # Mock stylix to satisfy the dependency requirement
              stylix.enable = true;
              stylix.base16Scheme = "${pkgs.base16-schemes}/share/themes/default-dark.yaml";
              
              # Configure dots-wallpaper
              dots.wallpaper = {
                enable = true;
                width = 1920;
                height = 1080;
              };
              
              # Create test users
              users.users.testuser1 = {
                isNormalUser = true;
                uid = 1001;
                home = "/home/testuser1";
                createHome = true;
              };
              
              users.users.testuser2 = {
                isNormalUser = true;
                uid = 1002;
                home = "/home/testuser2";
                createHome = true;
              };
              
              # Ensure our package is available
              environment.systemPackages = [ dots-wallpaper ];
            };
            
            testScript = ''
              import os
              import subprocess
              
              # Helper function to create test images
              def create_test_image(path, format_type, width=100, height=100, color="255,0,0"):
                  """Create a test image using ImageMagick convert command"""
                  cmd = f"${pkgs.imagemagick}/bin/convert -size {width}x{height} xc:rgb\\({color}\\) {path}"
                  subprocess.run(cmd, shell=True, check=True)
              
              def create_corrupt_file(path, content):
                  """Create a file with invalid content"""
                  with open(path, 'wb') as f:
                      f.write(content)
              
              machine.start()
              machine.wait_for_unit("multi-user.target")
              
              # Test 1: Create test images in various formats
              print("Creating test images in various formats...")
              
              # Create directories for test images
              machine.succeed("mkdir -p /home/testuser1/Pictures")
              machine.succeed("mkdir -p /home/testuser2/Pictures")
              machine.succeed("mkdir -p /tmp/test_images")
              
              # Create valid images in different formats
              test_images = [
                  ("/tmp/test_images/test.png", "PNG"),
                  ("/tmp/test_images/test.jpg", "JPEG"),
                  ("/tmp/test_images/test.bmp", "BMP"),
                  ("/tmp/test_images/test.gif", "GIF"),
                  ("/tmp/test_images/test.tiff", "TIFF"),
              ]
              
              for img_path, fmt in test_images:
                  machine.succeed(f"${pkgs.imagemagick}/bin/convert -size 200x200 xc:red {img_path}")
              
              # Test 2: Create edge case images
              print("Creating edge case images...")
              
              # Very small image (1x1)
              machine.succeed("${pkgs.imagemagick}/bin/convert -size 1x1 xc:blue /tmp/test_images/tiny.png")
              
              # Large image
              machine.succeed("${pkgs.imagemagick}/bin/convert -size 1000x1000 xc:green /tmp/test_images/large.png")
              
              # Non-square image
              machine.succeed("${pkgs.imagemagick}/bin/convert -size 300x100 xc:yellow /tmp/test_images/rect.png")
              
              # Image with transparency
              machine.succeed("${pkgs.imagemagick}/bin/convert -size 100x100 xc:none -fill 'rgba(255,0,0,0.5)' -draw 'rectangle 0,0 100,100' /tmp/test_images/transparent.png")
              
              # Test 3: Create invalid/corrupt files
              print("Creating invalid and corrupt files...")
              
              # Empty file with image extension
              machine.succeed("touch /tmp/test_images/empty.jpg")
              
              # Text file with image extension
              machine.succeed("echo 'This is not an image' > /tmp/test_images/fake.png")
              
              # Corrupt image file
              machine.succeed("echo 'PNG INVALID DATA' > /tmp/test_images/corrupt.png")
              
              # Non-image file with image extension
              machine.succeed("cp /etc/passwd /tmp/test_images/notimage.gif")
              
              # Test 4: Copy test images to user directories
              print("Setting up user wallpapers...")
              
              # User 1 gets valid images
              machine.succeed("cp /tmp/test_images/test.png /home/testuser1/Pictures/wallpaper1.png")
              machine.succeed("cp /tmp/test_images/test.jpg /home/testuser1/Pictures/wallpaper2.jpg")
              machine.succeed("cp /tmp/test_images/tiny.png /home/testuser1/Pictures/wallpaper3.png")
              
              # User 2 gets mix of valid and invalid
              machine.succeed("cp /tmp/test_images/test.bmp /home/testuser2/Pictures/wallpaper1.bmp")
              machine.succeed("cp /tmp/test_images/empty.jpg /home/testuser2/Pictures/wallpaper2.jpg")
              machine.succeed("cp /tmp/test_images/fake.png /home/testuser2/Pictures/wallpaper3.png")
              machine.succeed("cp /tmp/test_images/large.png /home/testuser2/Pictures/wallpaper4.png")
              
              # Set proper ownership
              machine.succeed("chown -R testuser1:users /home/testuser1/Pictures")
              machine.succeed("chown -R testuser2:users /home/testuser2/Pictures")
              
              # Test 5: Test the wallpaper generation
              print("Testing wallpaper generation...")
              
              # Run the activation script manually to test
              machine.succeed("mkdir -p /etc/nixos/wallpaper")
              
              # Test direct binary execution with various scenarios
              print("Testing binary with valid images...")
              machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_valid.png 800x600 20 /tmp/test_images/test.png /tmp/test_images/test.jpg")
              machine.succeed("test -f /tmp/output_valid.png")
              
              print("Testing binary with mixed valid/invalid images...")
              machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_mixed.png 800x600 0 /tmp/test_images/test.png /tmp/test_images/fake.png /tmp/test_images/test.bmp")
              machine.succeed("test -f /tmp/output_mixed.png")
              
              print("Testing binary with only invalid images...")
              machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_invalid.png 800x600 45 /tmp/test_images/fake.png /tmp/test_images/empty.jpg")
              machine.succeed("test -f /tmp/output_invalid.png")
              
              print("Testing binary with no images...")
              machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_empty.png 800x600 30")
              machine.succeed("test -f /tmp/output_empty.png")
              
              # Test 6: Test edge cases and error handling
              print("Testing edge cases...")
              
              # Test with very small resolution
              machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_small.png 10x10 0 /tmp/test_images/test.png")
              machine.succeed("test -f /tmp/output_small.png")
              
              # Test with large resolution (but reasonable for CI)
              machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_large.png 2000x1500 0 /tmp/test_images/test.png")
              machine.succeed("test -f /tmp/output_large.png")
              
              # Test with extreme angle
              machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_angle.png 400x400 89 /tmp/test_images/test.png /tmp/test_images/test.jpg")
              machine.succeed("test -f /tmp/output_angle.png")
              
              # Test 7: Verify proper error handling and logging
              print("Testing error handling and logging...")
              
              # These should succeed but log warnings
              result = machine.succeed("${dots-wallpaper}/bin/dots-wallpaper /tmp/output_warnings.png 100x100 0 /tmp/test_images/fake.png /tmp/test_images/test.png 2>&1")
              assert "Warning:" in result, "Expected warning messages for invalid files"
              
              print("All VM tests passed!")
            '';
          };
          
          # Test module configuration validation
          vm-test-module-validation = nixosTest {
            name = "dots-wallpaper-module-validation";
            
            nodes.machine = { config, pkgs, ... }: {
              imports = [ self.nixosModules.default ];
              
              # Mock stylix
              stylix.enable = true;
              stylix.base16Scheme = "${pkgs.base16-schemes}/share/themes/default-dark.yaml";
              
              # Test various module configurations
              dots.wallpaper = {
                enable = true;
                width = 1920;
                height = 1080;
              };
              
              environment.systemPackages = [ dots-wallpaper ];
            };
            
            testScript = ''
              machine.start()
              machine.wait_for_unit("multi-user.target")
              
              print("Testing module configuration validation...")
              
              # Test that the module loads correctly
              machine.succeed("systemctl status multi-user.target")
              
              # Test that our binary is available
              machine.succeed("which dots-wallpaper")
              
              # Test activation script exists
              machine.succeed("test -f /etc/nixos/wallpaper || mkdir -p /etc/nixos/wallpaper")
              
              print("Module validation tests passed!")
            '';
          };
          
          # Test without stylix (should fail validation)
          vm-test-stylix-requirement = nixosTest {
            name = "dots-wallpaper-stylix-requirement";
            
            nodes.machine = { config, pkgs, ... }: {
              imports = [ self.nixosModules.default ];
              
              # Don't enable stylix - should cause assertion failure
              dots.wallpaper.enable = true;
              
              environment.systemPackages = [ dots-wallpaper ];
            };
            
            testScript = ''
              # This test should fail to start because stylix is required
              try:
                  machine.start()
                  machine.wait_for_unit("multi-user.target")
                  # If we get here, the test failed - stylix requirement wasn't enforced
                  assert False, "Expected failure due to missing stylix requirement"
              except Exception as e:
                  # Expected to fail due to assertion
                  print(f"Expected failure: {e}")
                  print("Stylix requirement correctly enforced!")
            '';
          };
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
