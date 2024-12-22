set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

_default:
  just --list -u

fix:
  cargo fix --allow-dirty --allow-staged
  cargo fmt --all
