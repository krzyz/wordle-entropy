{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
        overlays = [ rust-overlay.overlays.default ];
      };
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          toolchain
        ] ++ (with pkgs; [
          rust-analyzer-unwrapped
          wasm-pack
        ]);

        nativeBuildInputs = with pkgs; [cmake pkg-config freetype expat fontconfig];

        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
      };
    };
}
