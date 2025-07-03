# 🎨 Dots Wallpaper

A robust NixOS wallpaper generator that creates beautiful composite wallpapers by combining multiple images with angled strips, designed for seamless integration with [stylix](https://github.com/danth/stylix).

## ✨ Features

- **Multi-image composition**: Combine multiple wallpapers into stunning angled strip layouts
- **Robust image handling**: Gracefully handles invalid, corrupt, or missing image files
- **Comprehensive format support**: Supports all major image formats (PNG, JPEG, BMP, GIF, TIFF, WebP, PNM)
- **NixOS integration**: Seamless integration with NixOS and stylix theming
- **Automatic user discovery**: Collects wallpapers from all user home directories
- **Flexible configuration**: Customizable resolution, angles, and fallback options

## 🛡️ Robustness & Error Handling

This application is designed to be extremely robust against various edge cases and invalid inputs:

### Image Format Support
- **PNG**: Full support including transparency (alpha channel)
- **JPEG**: Full support with compression handling
- **BMP**: Standard bitmap format support
- **GIF**: Animated GIFs handled as static images
- **TIFF**: Tagged Image File Format support
- **WebP**: Modern web image format support
- **PNM**: Portable Network Graphics family (PBM, PGM, PPM)

### Error Handling
- **Invalid image paths**: Non-existent files are logged and skipped
- **Corrupt image files**: Malformed or partially corrupted images are gracefully skipped
- **Non-image files**: Files with image extensions but invalid content are detected and skipped
- **Empty files**: Zero-byte files are handled gracefully
- **Permission errors**: Insufficient file permissions are logged and handled
- **Memory constraints**: Large images are processed efficiently with memory management

### Edge Cases
- **Empty input**: Creates a solid black wallpaper when no valid images are provided
- **Single image**: Directly resizes and saves the single image (no composition needed)
- **Tiny images**: 1x1 pixel images are supported and scaled appropriately
- **Large images**: High-resolution images are handled with efficient memory usage
- **Non-square images**: Aspect ratios are preserved during resizing
- **Transparency**: Alpha channels are properly converted to RGB with appropriate backgrounds
- **Duplicate images**: Same image used multiple times is handled correctly
- **Mixed valid/invalid**: Processes all valid images while logging warnings for invalid ones

All error conditions result in informative warning messages logged to stderr, allowing the application to continue processing other images successfully.

## 🚀 Installation

### As a NixOS Module

1. Add this flake to your NixOS configuration:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    dots-wallpaper.url = "github:shift/dots-wallpaper";
    stylix.url = "github:danth/stylix";
  };

  outputs = { nixpkgs, dots-wallpaper, stylix, ... }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
      modules = [
        stylix.nixosModules.stylix
        dots-wallpaper.nixosModules.default
        {
          # Enable stylix (required)
          stylix.enable = true;
          stylix.base16Scheme = "${pkgs.base16-schemes}/share/themes/dracula.yaml";
          
          # Configure dots-wallpaper
          dots.wallpaper = {
            enable = true;
            width = 1920;
            height = 1080;
          };
        }
      ];
    };
  };
}
```

### As a Standalone Package

```bash
# Using nix run
nix run github:shift/dots-wallpaper -- output.png 1920x1080 20 image1.jpg image2.png

# Install globally
nix profile install github:shift/dots-wallpaper

# In a development environment
nix develop github:shift/dots-wallpaper
```

## ⚙️ Configuration

### NixOS Module Options

```nix
dots.wallpaper = {
  enable = true;           # Enable the wallpaper generator
  width = 1920;           # Output width in pixels
  height = 1080;          # Output height in pixels
  defaultWallpaper = ./path/to/fallback.jpg;  # Fallback wallpaper
  flakeRootDefault = ./assets/default.jpg;    # Default from flake root
};
```

### Command Line Usage

```bash
dots-wallpaper <output_path> <width>x<height> <angle_degrees> [image_paths...]
```

**Parameters:**
- `output_path`: Where to save the generated wallpaper
- `width>x<height`: Output resolution (e.g., "1920x1080")
- `angle_degrees`: Angle of the strips in degrees (0 = vertical, 45 = diagonal)
- `image_paths`: Paths to input images (optional)

**Examples:**
```bash
# Create a vertical strip wallpaper
dots-wallpaper output.png 1920x1080 0 img1.jpg img2.png img3.bmp

# Create angled strips at 30 degrees
dots-wallpaper wallpaper.png 2560x1440 30 photo1.jpg photo2.png

# Handle mixed valid and invalid files (will log warnings but continue)
dots-wallpaper output.png 1920x1080 45 valid.jpg corrupt.png nonexistent.jpg valid2.bmp
```

## 🏗️ Building & Development

### Prerequisites

- Nix with flakes enabled
- Rust toolchain (provided via Nix)

### Development

```bash
# Enter development environment
nix develop

# Run tests
cargo test

# Run all tests (including VM tests)
nix build .#packages.x86_64-linux.test-runner
./result/bin/run-all-tests

# Build the package
nix build

# Run VM tests
nix build .#checks.x86_64-linux.vm-test
```

### Testing

The project includes comprehensive test coverage:

- **Unit tests**: Test core functionality and edge cases
- **Robustness tests**: Test handling of invalid/corrupt files and various image formats
- **Integration tests**: Test complete workflows
- **VM tests**: Test NixOS module integration and system-level behavior

```bash
# Run Rust unit tests
cargo test

# Run VM tests via Nix
nix build .#checks.x86_64-linux.vm-test
nix build .#checks.x86_64-linux.vm-test-module-validation
nix build .#checks.x86_64-linux.vm-test-stylix-requirement
```

## 🔄 Automatic Releases & Changelog

This project uses automated release management:

- **Release automation**: Powered by [release-plz](https://release-plz.ieni.dev/)
- **Changelog generation**: Powered by [git-cliff](https://git-cliff.org/)
- **Conventional commits**: Uses [Conventional Commits](https://www.conventionalcommits.org/) for automated versioning
- **GitHub Actions**: Automated workflows for releases and changelog generation on PRs

### Release Process

1. **Automatic releases**: Triggered weekly or when changes are pushed to main
2. **Semantic versioning**: Version numbers follow [SemVer](https://semver.org/) based on commit messages
3. **Changelog generation**: Automatically generated from conventional commit messages
4. **PR previews**: Changelog previews are automatically posted on pull requests

### Contributing

When contributing, please use conventional commit messages:

```bash
feat: add support for WebP image format
fix: handle corrupted JPEG files gracefully
docs: update installation instructions
test: add tests for edge case image sizes
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines and make sure to:

1. Follow conventional commit messages
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure all tests pass (`cargo test` and VM tests)

## 🙏 Acknowledgments

- [stylix](https://github.com/danth/stylix) - For the excellent NixOS theming framework
- [image-rs](https://github.com/image-rs/image) - For robust image processing capabilities
- [NixOS](https://nixos.org/) - For the amazing declarative system configuration