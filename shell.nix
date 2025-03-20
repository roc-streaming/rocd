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

            (buildPythonPackage rec {
              # docs: d2 plugin for mkdocs
              pname = "mkdocs_d2_plugin"; # https://pypi.org/project/mkdocs-d2-plugin
              version = "1.6.0"; # XXX: https://github.com/landmaj/mkdocs-d2-plugin/releases
              src = fetchPypi {
                inherit pname version;
                hash = "sha256-QsHy3C7p+omMYg4BHyJt4NkKtM0mQFYBmdf5GViaWRs=";
              };
              dependencies = [
                mkdocs
                pydantic
                pymdown-extensions
              ];
              doCheck = true;
            })

            (buildPythonPackage rec {
              # docs: plantuml plugin for mkdocs
              pname = "mkdocs_puml"; # https://pypi.org/project/mkdocs_puml
              version = "2.3.0"; # XXX: https://github.com/MikhailKravets/mkdocs_puml/releases
              src = fetchPypi {
                inherit pname version;
                hash = "sha256-Ucf3QutjhOTkjbiApjiGnf+bOafULMQthqryXw/9tao";
              };
              dependencies = [
                httpx
                mkdocs
                msgpack
                rich
              ];
              pythonRelaxDeps = [ "httpx" ];
              build-system = [ poetry-core ];
              pyproject = true;
              doCheck = true;
            })

            # FIXME: alternative plugin, but building should be fixed
            #        >   File "setup.py", line 10, in <module>
            #        >     with open('VERSION', 'r') as f:
            #        >          ^^^^^^^^^^^^^^^^^^^^
            #        > FileNotFoundError: [Errno 2] No such file or directory: 'VERSION'
            #(
            #  buildPythonPackage rec {
            #    pname = "mkdocs-build-plantuml-plugin";
            #    version = "1.11.0";
            #    src = fetchPypi {
            #      inherit pname version;
            #      hash = "sha256-PolKtx5183+MUjbb5OY9k4MSv6el/CSJz5NHplnbQfg";
            #    };
            #    dependencies = [
            #    ];
            #    doCheck = true;
            #  }
            #)

            # mermaid has Github support, but doesn't work out of the box;
            # moreover it renders a diagram on client side => doesn't fit
            #mkdocs-mermaid2-plugin
          ];
      in
      unstable.python3.withPackages my_py_pkgs
    )
  ];

  buildInputs = [
    pkgs.entr # needed for `make docs-serve`
  ];
}
