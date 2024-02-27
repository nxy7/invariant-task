# Invariant Recruitment Task 1. Programing, math, problem solving

## Prerequisites
`rust tool chain` installed or `Nix` installed

## How to run tests with Nix
This repository is using Nix to provide development shell with exact rustc/cargo/nextest versions pinned
in flake lockfile. Because of this all tests are fully reproducible and can be ran using the following command

```bash
  nix develop . -c bash -c "cargo nextest r"
```
