{
  self,
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.dots.wallpaper;

  pieImageCombinerApp = self.packages.${pkgs.system}.dots-wallpaper;

  wallpaperGeneratorScript = pkgs.writeShellScript "generate-wallpaper" ''

    # This script collects user wallpapers at activation time and generates the composite wallpaper

    set -euo pipefail

    # Set up variables
    OUTPUT_PATH="./system-wallpaper.png"
    DEFAULT_WALLPAPER="${toString cfg.defaultWallpaper}"
    FLAKE_ROOT_DEFAULT="${toString cfg.flakeRootDefault}"

    # Function to find wallpapers in user HOME directories
    collect_user_wallpapers() {
      local -a wallpapers=()
      
      # Check each user's home directory
      while IFS=: read -r username _ uid _ _ home_dir _; do
        # Skip system users and users with UID < 1000
        if [[ "$uid" -lt 1000 ]]; then
          continue
        fi
        
        # Look for stylix config in user's home
        if [[ -f "$home_dir/.config/stylix/image" ]]; then
          # This might be a path or a symlink to the actual wallpaper
          user_wallpaper=$(readlink -f "$home_dir/.config/stylix/image")
          if [[ -f "$user_wallpaper" ]]; then
            wallpapers+=("$user_wallpaper")
          fi
        fi
      done < /etc/passwd
      
      # If no wallpapers found, use default
      if [[ "''${#wallpapers[@]}" -eq 0 ]]; then
        # Try flake root default first
        if [[ -f "$FLAKE_ROOT_DEFAULT" ]]; then
          echo "No user wallpapers found. Using flake root default: $FLAKE_ROOT_DEFAULT"
          wallpapers+=("$FLAKE_ROOT_DEFAULT")
        else
          # Fall back to configured default
          echo "No user wallpapers found. Using configured default: $DEFAULT_WALLPAPER"
          wallpapers+=("$DEFAULT_WALLPAPER")
        fi
      fi
      
      # Fix: Print array elements properly quoted
      for wp in "''${wallpapers[@]}"; do
        printf "%s\n" "$wp"
      done
    }

    # Collect wallpapers - Fix: Use read to handle paths with spaces
    mapfile -t wallpapers < <(collect_user_wallpapers)

    echo "Generating combined wallpaper with ${toString cfg.width}x${toString cfg.height} dimensions"

    ${pieImageCombinerApp}/bin/dots-wallpaper \
      "$OUTPUT_PATH" \
      20 \
      "${toString cfg.width}x${toString cfg.height}" \
      "''${wallpapers[@]}"
      
    # Update stylix to use the new wallpaper
    # We need to create a symlink that matches the expected stylix path pattern
    echo "Setting system wallpaper to: $OUTPUT_PATH"
    ln -sf "$OUTPUT_PATH" "/etc/nixos/wallpaper.png"

    # Force stylix to reload the wallpaper
    if command -v "dconf" &> /dev/null; then
      echo "Refreshing desktop wallpaper..."
      
      # If using GNOME
      dconf reset /org/gnome/desktop/background/picture-uri || true
      sleep 1
      dconf write /org/gnome/desktop/background/picture-uri "'file:///etc/nixos/wallpaper.png'" || true
      
      # If using KDE Plasma
      if command -v "plasma-apply-wallpaperimage" &> /dev/null; then
        plasma-apply-wallpaperimage "/etc/nixos/wallpaper.png" || true
      fi
    fi

    echo "Wallpaper generation complete!"


  '';
in
{
  # Define module options under the 'dots.wallpaper' path
  options.dots.wallpaper = {
    enable = mkEnableOption "NixOS Pie Chart Wallpaper Generator";

    width = mkOption {
      type = types.int;
      default = 1920;
      description = "Width of the generated wallpaper in pixels.";
      apply =
        value: if value <= 0 then throw "dots.wallpaper: Width must be a positive integer" else value;
    };

    height = mkOption {
      type = types.int;
      default = 1080;
      description = "Height of the generated wallpaper in pixels.";
      apply =
        value: if value <= 0 then throw "dots.wallpaper: Height must be a positive integer" else value;
    };

    defaultWallpaper = mkOption {
      type = types.path;
      default = /etc/nixos/wallpaper/default.jpg;
      description = "Default wallpaper to use if no input wallpapers are found.";
    };

    flakeRootDefault = mkOption {
      type = types.path;
      default = /etc/nixos/wallpaper/default.jpg;
      description = "Path to the default wallpaper in your flake root.";
      example = literalExpression "../../../assets/wallpaper.jpg";
    };
  };

  # Configure the module's behavior
  config = mkIf cfg.enable {
    # Ensure stylix is enabled
    assertions = [
      {
        assertion = hasAttrByPath [ "stylix" "enable" ] config && config.stylix.enable;
        message = "dots.wallpaper requires stylix to be enabled";
      }
    ];

    # Create a system-level directory for storing the wallpaper
    system.activationScripts.setupWallpaperDir = ''
      mkdir -p /etc/nixos/wallpaper
    '';

    # Use our script to generate the wallpaper during activation
    system.activationScripts.generatePieChartWallpaper = ''
      # Run the wallpaper generator script
      echo "Generating pie chart wallpaper from user wallpapers..."
      ${wallpaperGeneratorScript} 
    '';

    # Tell stylix to use our image path
    # This won't create a circular dependency because the path is fixed ahead of time

    # Make sure the tools we need are installed
    environment.systemPackages = [
      pieImageCombinerApp 
    ];
  };
}
