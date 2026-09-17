{
  description = "Rust project";

  inputs = {
    harbor-rs.url = "git+https://github.com/caniko/harbor-rs.git?ref=trunk&rev=fac8049316846e0ef1c1e6acd92aed7a337b333a";
    nixpkgs.follows = "harbor-rs/nixpkgs";
    rust-overlay.follows = "harbor-rs/rust-overlay";
    crane.follows = "harbor-rs/crane";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    git-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = {
    self,
    harbor-rs,
    nixpkgs,
    rust-overlay,
    crane,
    flake-utils,
    treefmt-nix,
    git-hooks,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      toolchain = harbor-rs.lib.mkToolchain {
        inherit pkgs;
        toolchainProfile = "stable";
        crossTargets = [
          "x86_64-unknown-linux-gnu"
          "aarch64-unknown-linux-gnu"
          "x86_64-pc-windows-gnu"
          "x86_64-apple-darwin"
          "aarch64-apple-darwin"
          "wasm32-unknown-unknown"
        ];
      };
      inherit (toolchain) craneLib rawCraneLib rustToolchain;
      buildCache = harbor-rs.lib.mkBuildCachePolicy {
        inherit pkgs;
        buildPackageSet = pkgs.buildPackages;
        sccachePackage = harbor-rs.packages.${system}.sccache;
        cacheRoot = null;
        namespaceScope = "canix-rust";
        namespaceGeneration = 5;
      };
      # Dioxus' `asset!` macro resolves assets while Cargo compiles. Crane's
      # default Cargo source filter excludes CSS, so keep the shared stylesheet
      # alongside the ordinary Rust sources in every package and check.
      src = pkgs.lib.fileset.toSource {
        root = ./.;
        fileset = pkgs.lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
          ./crates/tartan-ui-dioxus/assets
        ];
      };
      commonArgs = {
        inherit src;
        pname = "tartan-ui";
        version = "0.1.0";
        strictDeps = true;
        # Dioxus native and desktop dependency graphs include openssl-sys.
        # Keep the native dependency visible to both Crane checks and package
        # builds so the renderer matrix is reproducible outside a workstation
        # with system OpenSSL installed.
        nativeBuildInputs = with pkgs; [pkg-config openssl.dev];
        buildInputs = with pkgs; [openssl];
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      # Renderer checks must also run on builders that do not expose atlas'
      # managed compiler-cache transport. The release package remains on the
      # fail-closed harbor-rs cache policy above.
      checkCargoArtifacts = rawCraneLib.buildDepsOnly commonArgs;
      dioxusWebArgs = commonArgs // {
        cargoArtifacts = checkCargoArtifacts;
      };
      dioxusServerArgs = commonArgs // {
        cargoArtifacts = checkCargoArtifacts;
      };
      dioxusNativeArgs = commonArgs // {
        cargoArtifacts = checkCargoArtifacts;
        # Blitz' Stylo dependency generates properties during compilation.
        nativeBuildInputs = commonArgs.nativeBuildInputs ++ [pkgs.python3];
      };
      dioxusDesktopArgs = commonArgs // {
        cargoArtifacts = checkCargoArtifacts;
        # dioxus-desktop uses Wry/WebKitGTK on Linux. Declaring these here
        # keeps browser, server and Blitz-native checks free of desktop-only
        # native dependencies.
        buildInputs = commonArgs.buildInputs ++ (with pkgs; [
          gtk3
          webkitgtk_4_1
        ]);
      };
      mkDioxusCheck = args: command:
        rawCraneLib.mkCargoDerivation (args // {
          pnameSuffix = "-check";
          buildPhaseCargoCommand = command;
          installPhaseCommand = "mkdir -p $out";
        });
      package = buildCache.withRustCache {
        package = craneLib.buildPackage (commonArgs // {inherit cargoArtifacts;});
      };
      treefmtEval = treefmt-nix.lib.evalModule pkgs (import ./nix/treefmt.nix);
      pre-commit-check = git-hooks.lib.${system}.run {
        src = ./.;
        hooks = import ./nix/pre-commit.nix {
          inherit pkgs;
          treefmtWrapper = treefmtEval.config.build.wrapper;
          inherit rustToolchain;
        };
      };
    in {
      packages.default = package;
      formatter = treefmtEval.config.build.wrapper;
      checks = {
        default = package;
        formatting = treefmtEval.config.build.check self;
        # Renderer features are mutually exclusive in this crate (for
        # example, `web` and `native-embedded` intentionally cannot coexist),
        # so validation is an explicit matrix rather than `--all-features`.
        core-tests = rawCraneLib.cargoTest (commonArgs // {
          cargoArtifacts = checkCargoArtifacts;
          cargoTestExtraArgs = "-p tartan-ui-core";
        });
        dioxus-web = mkDioxusCheck dioxusWebArgs "cargoWithProfile check --locked -p tartan-ui-dioxus --no-default-features --features web";
        dioxus-web-devtools = mkDioxusCheck dioxusWebArgs "cargoWithProfile check --locked -p tartan-ui-dioxus --no-default-features --features web,devtools";
        dioxus-web-wasm-split = mkDioxusCheck dioxusWebArgs "cargoWithProfile check --locked -p tartan-ui-dioxus --no-default-features --features web,wasm-split";
        dioxus-server = mkDioxusCheck dioxusServerArgs "cargoWithProfile check --locked -p tartan-ui-dioxus --no-default-features --features server";
        dioxus-native = mkDioxusCheck dioxusNativeArgs "cargoWithProfile check --locked -p tartan-ui-dioxus --no-default-features --features native";
        dioxus-desktop = mkDioxusCheck dioxusDesktopArgs "cargoWithProfile check --locked -p tartan-ui-dioxus --no-default-features --features desktop";
        clippy = rawCraneLib.cargoClippy (dioxusWebArgs // {
          # Crane's cargoClippy helper supplies --locked for the derivation;
          # repeating it here makes Cargo reject the command line.
          cargoClippyExtraArgs = "-p tartan-ui-dioxus --no-default-features --features web --all-targets -- --deny warnings";
        });
        fmt = rawCraneLib.cargoFmt {inherit src;};
      };
      devShells.default = craneLib.devShell {
        checks = self.checks.${system};
         packages = [harbor-rs.packages.${system}.harbor-ci] ++ (with pkgs; [
          cargo-about
          cargo-audit
          cargo-cyclonedx
          cargo-deny
          cargo-llvm-cov
          cargo-sbom
          cargo-nextest
          cosign
          binaryen
          dioxus-cli
          file
          gnutar
          gzip
          jq
          minisign
          nodejs
          openssl
          pkg-config
          python3
          pre-commit
          rpm
          util-linux
          unzip
          zip
          reprepro
          rust-analyzer
          taplo
         ] ++ pre-commit-check.enabledPackages);
        shellHook = pre-commit-check.shellHook;
      };
      apps.local-check-fast = {
        type = "app";
        program = let
          script = pkgs.writeShellApplication {
            name = "local-check-fast";
            runtimeInputs = with pkgs; [
              cargo-deny
              git
              gtk3
              jq
              openssl
              openssl.dev
              pkg-config
              python3
              rustToolchain
              stdenv.cc
              webkitgtk_4_1
            ];
            text = ''
              set -euo pipefail
              # `nix run` does not inherit the devShell's compiler or
              # pkg-config environment. Keep the standalone gate equivalent
              # to the native/desktop Nix checks instead of failing at the
              # first build-script invocation.
              export PKG_CONFIG_PATH="${pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" (with pkgs; [
                openssl
                gtk3
                webkitgtk_4_1
                glib
                atk
                gdk-pixbuf
                pango
                cairo
                libsoup_3
                libepoxy
                harfbuzz
                fribidi
                zlib
                libpng
                expat
                at-spi2-core
                libffi
                pcre2
                fontconfig
                wayland
                cups
                libdrm
                mesa
                libX11
                libXext
                libXi
                libXrender
                libXrandr
                libXcursor
                libXdamage
                libXcomposite
                libXfixes
                libXinerama
                libICE
                libSM
              ])}:${pkgs.lib.makeSearchPathOutput "dev" "share/pkgconfig" (with pkgs; [zlib])}''${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
              export OPENSSL_NO_VENDOR=1
              # Keep this in lockstep with the renderer-specific Nix checks;
              # `--all-features` is invalid because web and native embedding
              # are deliberately mutually exclusive.
              cargo test -p tartan-ui-core --locked
              cargo test -p tartan-ui-dioxus --no-default-features --features web --locked
              cargo check -p tartan-ui-dioxus --no-default-features --features web,devtools --locked
              cargo check -p tartan-ui-dioxus --no-default-features --features web,wasm-split --locked
              cargo check -p tartan-ui-dioxus --no-default-features --features server --locked
              cargo check -p tartan-ui-dioxus --no-default-features --features native --locked
              cargo check -p tartan-ui-dioxus --no-default-features --features desktop --locked
              cargo clippy -p tartan-ui-dioxus --no-default-features --features web --all-targets --locked -- --deny warnings
              cargo deny check bans licenses sources
              cargo package --workspace --allow-dirty --list >/dev/null
            '';
          };
        in "${script}/bin/local-check-fast";
        meta.description = "Run fast local validation checks";
      };
      apps.local-check-release = {
        type = "app";
        program = let
          script = pkgs.writeShellApplication {
            name = "local-check-release";
            runtimeInputs = with pkgs; [
              cargo-about
              cargo-cyclonedx
              cargo-deny
              cargo-sbom
              cosign
              jq
              minisign
              rustToolchain
            ];
            text = ''
              set -euo pipefail
              version="''${1:-}"
              if [ -z "$version" ]; then
                echo "usage: local-check-release <version>" >&2
                exit 2
              fi
              repo="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
              cd "$repo"
              ${self.apps.${system}.local-check-fast.program}
              manifest="release/artifacts.json"
              mkdir -p release
              jq -n --arg version "$version" \
                '{version: $version, artifacts: [], skipped: [], generated_by: "simit local-check-release"}' \
                > "$manifest.tmp"
              mv "$manifest.tmp" "$manifest"
              manifest_add_file() {
                path="$1"
                producer="$2"
                [ -f "$path" ] || return 0
                sha256="$(sha256sum "$path" | awk '{print $1}')"
                jq --arg path "$path" --arg sha256 "$sha256" --arg producer "$producer" \
                  '.artifacts += [{path: $path, sha256: $sha256, producer: $producer}]' \
                  "$manifest" > "$manifest.tmp"
                mv "$manifest.tmp" "$manifest"
              }
              manifest_skip() {
                name="$1"
                reason="$2"
                jq --arg name "$name" --arg reason "$reason" \
                  '.skipped += [{name: $name, reason: $reason}]' \
                  "$manifest" > "$manifest.tmp"
                mv "$manifest.tmp" "$manifest"
              }
              if [ -f about-template.hbs ]; then
                cargo about generate --output-file release/THIRD_PARTY_LICENSES.html about-template.hbs
                manifest_add_file release/THIRD_PARTY_LICENSES.html cargo-about
              else
                echo "warning: about-template.hbs not found; skipping cargo-about report" >&2
                manifest_skip cargo-about "about-template.hbs not found"
              fi
              cargo sbom --output-format cyclone_dx_json_1_5 > "release/''${version}.cdx.json"
              cargo sbom --output-format spdx_json_2_3 > "release/''${version}.spdx.json"
              manifest_add_file "release/''${version}.cdx.json" cargo-sbom-cyclonedx
              manifest_add_file "release/''${version}.spdx.json" cargo-sbom-spdx
              if [ -n "''${COSIGN_PRIVATE_KEY:-}" ]; then
                echo "COSIGN_PRIVATE_KEY present; local release parity will not sign or upload" >&2
              else
                echo "warning: keyless Sigstore and COSIGN_PRIVATE_KEY unavailable locally; skipping local cosign signing" >&2
                manifest_skip cosign "keyless Sigstore and COSIGN_PRIVATE_KEY unavailable locally"
              fi
              if [ -x scripts/release-local-check.sh ]; then
                bash scripts/release-local-check.sh "$version"
              fi
              echo "local release parity dry-run passed for ''${version}; no external publish was attempted"
            '';
          };
        in "${script}/bin/local-check-release";
        meta.description = "Run local release parity checks without publishing";
      };
      apps.local-release-deploy = {
        type = "app";
        program = let
          script = pkgs.writeShellApplication {
            name = "local-release-deploy";
            runtimeInputs = with pkgs; [
              git
              jq
            ];
            text = ''
              set -euo pipefail
              version="''${1:-}"
              publish_flag="''${2:-}"
              publish_version="''${3:-}"
              if [ -z "$version" ] || [ "$publish_flag" != "--publish" ] || [ "$publish_version" != "$version" ]; then
                echo "usage: local-release-deploy <version> --publish <version>" >&2
                echo "refusing to publish without an explicit matching confirmation" >&2
                exit 2
              fi
              repo="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
              cd "$repo"
              ${self.apps.${system}.local-check-release.program} "$version"
              if ! jq -e --arg version "$version" '.version == $version' release/artifacts.json >/dev/null; then
                echo "release/artifacts.json is missing or does not match version $version" >&2
                exit 1
              fi
              if [ -x scripts/local-release-deploy.sh ]; then
                SIMIT_LOCAL_RELEASE_CHECK_DONE=1 exec bash scripts/local-release-deploy.sh "$version" --publish "$version"
              fi
              echo "local-release-deploy has no project publisher hook at scripts/local-release-deploy.sh" >&2
              echo "Homebrew-capable hooks must build Darwin tarballs and gate tap pushes on HOMEBREW_TAP_TOKEN" >&2
              echo "local-check-release must remain non-publishing: no brew bump, git push, upload, or cargo publish" >&2
              exit 2
            '';
          };
        in "${script}/bin/local-release-deploy";
        meta.description = "Run the guarded local release deployment hook";
      };
    });
}
