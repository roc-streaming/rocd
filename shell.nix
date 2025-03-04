{ pkgs ? import <nixpkgs> {} }:
let
  unstable = import <nixos-unstable> {};
in
  pkgs.mkShell {
    nativeBuildInputs = [
      (
        let my_py_pkgs = p: with p; [
          mkdocs
          (
            buildPythonPackage rec {
              # https://pypi.org/project/mkdocs-d2-plugin
              pname = "mkdocs_d2_plugin";
              version = "1.6.0";
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
            }
          )

          # FIXME
          #~(
          #~  buildPythonPackage rec {
          #~    # https://pypi.org/project/mkdocs_puml
          #~    pname = "mkdocs_puml";
          #~    version = "2.3.0";
          #~    src = fetchPypi {
          #~      inherit pname version;
          #~      hash = "sha256-Ucf3QutjhOTkjbiApjiGnf+bOafULMQthqryXw/9tao";
          #~    };
          #~    dependencies = [
          #~      mkdocs
          #~      msgpack
          #~      rich
          #~    ];
          #~    pythonRemoveDeps = [ httpx ];
          #~    build-system = [ poetry-core ];
          #~    pyproject = true;
          #~    doCheck = true;
          #~  }
          #~)

          # FIXME
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

        ]; in pkgs.python3.withPackages my_py_pkgs
      )
    ];
  }
