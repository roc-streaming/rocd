{
  pkgs ? import <nixpkgs> { },
}:
let
  unstable = import <nixos-unstable> { };
in
pkgs.mkShell {
  nativeBuildInputs = [
    unstable.cargo
    unstable.rustc
    unstable.rustfmt

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
