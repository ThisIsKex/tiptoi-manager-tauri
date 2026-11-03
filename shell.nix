{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "tiptoi-manager-tauri";

  nativeBuildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    nodejs_24
  ];

  buildInputs = with pkgs; [
    at-spi2-atk
    atkmm
    cairo
    gdk-pixbuf
    glib
    glib-networking
    gtk3
    harfbuzz
    librsvg
    libsoup_3
    openssl
    pango
    webkitgtk_4_1
  ];

  # Tauri-Runtime-Bibliotheken zur Laufzeit auffindbar machen.
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.webkitgtk_4_1
    pkgs.gtk3
    pkgs.glib
    pkgs.cairo
    pkgs.pango
    pkgs.gdk-pixbuf
    pkgs.at-spi2-atk
    pkgs.librsvg
  ];

  # WebKitGTK/libsoup brauchen glib-networking als GIO-Modul, um TLS/HTTPS
  # nutzen zu können (sonst: "TLS support is not available").
  GIO_EXTRA_MODULES = "${pkgs.glib-networking}/lib/gio/modules";
}
