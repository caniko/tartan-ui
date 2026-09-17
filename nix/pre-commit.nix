{
  pkgs,
  treefmtWrapper,
  rustToolchain ? null,
}: {
  treefmt = {
    enable = true;
    name = "treefmt";
    package = treefmtWrapper;
    entry = "${treefmtWrapper}/bin/treefmt --fail-on-change";
    pass_filenames = false;
  };

  cargo-fmt = {
    enable = true;
    name = "cargo fmt";
    entry = "cargo fmt --all -- --check";
    extraPackages = pkgs.lib.optional (rustToolchain != null) rustToolchain;
    pass_filenames = false;
  };

  cargo-clippy = {
    enable = true;
    name = "cargo clippy";
    # The Dioxus crate's renderer features are mutually exclusive. Check the
    # browser surface here; the flake check and release app cover the complete
    # web/server/native/desktop matrix.
    entry = "cargo clippy -p tartan-ui-dioxus --no-default-features --features web --all-targets --locked -- --deny warnings";
    extraPackages = pkgs.lib.optional (rustToolchain != null) rustToolchain;
    pass_filenames = false;
  };

  cargo-msrv = {
    enable = true;
    name = "cargo check MSRV";
    entry = "${pkgs.rust-bin.stable."1.85.0".default}/bin/cargo check -p tartan-ui-core --locked && ${pkgs.rust-bin.stable."1.85.0".default}/bin/cargo check -p tartan-ui-dioxus --no-default-features --features web --locked";
    extraPackages = [pkgs.rust-bin.stable."1.85.0".default];
    pass_filenames = false;
    stages = ["pre-push" "manual"];
  };

  cargo-audit = {
    enable = true;
    name = "cargo audit";
    entry = "cargo audit";
    extraPackages = pkgs.lib.optional (rustToolchain != null) rustToolchain ++ [pkgs.cargo-audit];
    pass_filenames = false;
  };

  nix-flake-check = {
    enable = true;
    name = "nix flake check";
    entry = "nix --extra-experimental-features 'nix-command flakes' flake check --cores 0 --max-jobs auto --no-update-lock-file";
    extraPackages = [pkgs.nix];
    pass_filenames = false;
    stages = ["manual"];
  };
}
