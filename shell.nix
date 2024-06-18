{ pkgs ? import <nixpkgs> {} }:

  pkgs.mkShell {
    buildInputs = [
      pkgs.go-swag
    ];
  }
