{ pkgs, ... }: {
  packages = with pkgs; [
    git
    wasm-pack
    python3
    lsof
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src" ];
    targets = [ "wasm32-unknown-unknown" ];
  };
}
