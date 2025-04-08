{
  pkgs ? import <nixpkgs> { },
}:
let
  unstable = import <nixos-unstable> { };
in
pkgs.mkShell {
  nativeBuildInputs = [
    unstable.cargo
    unstable.clippy
    unstable.rustc
    unstable.rustfmt

    pkgs.cargo-expand # Subcommand to show result of macro expansion
    #pkgs.cargo-shear # Detect and remove unused dependencies from Cargo.toml
    #pkgs.cargo-ndk # Cargo extension for building Android NDK projects
    #pkgs.cargo-bloat # Tool and Cargo subcommand that helps you find out what takes most of the space in your executable
    #cargo-audit # Audit Cargo.lock files for crates with security vulnerabilities

    # add some Python packages
    (
      let
        my_py_pkgs =
          p: with p; [
            mkdocs
            mkdocs-material
            pymdown-extensions
          ];
      in
      unstable.python3.withPackages my_py_pkgs
    )
  ];

  buildInputs = [
    pkgs.entr # optional; used by `make docs-serve`
  ];
}
