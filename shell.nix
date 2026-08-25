{
  pkgs ? import <nixpkgs> { },
}:

let
  rustfmt-nightly = pkgs.rustfmt.override { asNightly = true; };
  libdisplay-info =
    if pkgs ? libdisplay-info_0_3 then pkgs.libdisplay-info_0_3 else pkgs.libdisplay-info;
in
pkgs.mkShell {
  packages = builtins.attrValues {
    inherit (pkgs)
      cargo
      clippy
      cargo-insta
      rust-analyzer
      rustc
      ;
    inherit rustfmt-nightly;
  };

  nativeBuildInputs = with pkgs; [
    clang
    pkg-config
    rustPlatform.bindgenHook
    wrapGAppsHook4
  ];

  buildInputs = with pkgs; [
    cairo
    dbus
    libadwaita
    libGL
    libdisplay-info
    libgbm
    libinput
    libxkbcommon
    pango
    pipewire
    seatd
    systemd
    wayland
  ];

  env = {
    RUSTFLAGS = toString (
      map (arg: "-C link-arg=" + arg) [
        "-Wl,--push-state,--no-as-needed"
        "-lEGL"
        "-lwayland-client"
        "-Wl,--pop-state"
      ]
    );
  };
}
