{
  description = "Basic Rust development environment";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    let
      overlays = [
        (import rust-overlay)
        (self: super: { rustToolchain = super.rust-bin.stable.latest.default; })
      ];
    in flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit overlays system; };
      in {
        devShells.default = pkgs.mkShell {
          packages = (with pkgs; [ rustToolchain rust-analyzer just taplo ])
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [
              libclang
              libiconv
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreServices
              darwin.apple_sdk.frameworks.Carbon
            ]);
        };
      });
}
