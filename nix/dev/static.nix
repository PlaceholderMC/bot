{
  perSystem = {
    lib,
    pkgs,
    inputs',
    refraction',
    ...
  }: let
    targets = with pkgs.pkgsCross; {
      x86_64 = musl64.pkgsStatic;
      aarch64 = aarch64-multiplatform.pkgsStatic;
    };

    toolchain = inputs'.rust-overlay.packages.rust.minimal.override {
      extensions = ["rust-std"];
      targets = map (pkgs: pkgs.stdenv.hostPlatform.config) (lib.attrValues targets);
    };

    rustPlatforms =
      lib.mapAttrs (
        lib.const (pkgs:
          pkgs.makeRustPlatform (
            lib.genAttrs ["cargo" "rustc"] (lib.const toolchain)
          ))
      )
      targets;

    buildWith = rustPlatform:
      refraction'.packages.refraction.override {
        inherit rustPlatform;
        optimizeSize = true;
      };
  in {
    packages =
      lib.mapAttrs' (
        target: rustPlatform:
          lib.nameValuePair "refraction-static-${target}" (buildWith rustPlatform)
      )
      rustPlatforms;
  };
}