# Crisol (English summary)

[![CI](https://github.com/blackl1ght98/Crisol/actions/workflows/ci.yml/badge.svg)](https://github.com/blackl1ght98/Crisol/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/blackl1ght98/Crisol.svg)](https://github.com/blackl1ght98/Crisol/releases)

**Read the full documentation in [Spanish](README.md).**

Crisol is a Rust (standard library only) orchestration layer on top of
Wine/winetricks. Wine is the engine; Crisol is the mechanic: it detects the
installer type, creates an isolated prefix, runs silent installers, reads the
log, applies recipes, retries, extracts the PE icon and registers the app in
the desktop. When Wine is not enough (kernel anti-cheat, DRM, drivers) it can
hand the installer off to a Windows VM.

> Status: working prototype, version 0.1.5. Releases `v0.1.0` to `v0.1.5` are
> tagged with source tarball, binary and `SHA256SUMS`. Six of the eight community
> recipes are anonymized/synthetic examples, **not real installation logs**; two
> use redacted real logs. The VM path has not been boot-tested here and QEMU is
> not installed. See `community-recipes/REAL-LOGS.md` and `docs/VM.md`.

## Basic commands

```bash
./install.sh                                   # build and install to ~/.local/bin
crisol install ~/Downloads/setup.msi            # silent install (type detected)
crisol install game.exe --engine wine-ge --attempts 7
crisol install app.exe --interactive            # show the installer UI
crisol install portable.exe                     # plain .exe: register only, do not run
crisol install portable.exe --execute           # force execution as a smoke test
crisol install app.exe --vm                     # hand off to the Windows VM

crisol list                                    # installed apps
crisol launch <id>                             # --force overrides the VDS warning
crisol launch <id> --vm                        # stage the installer in the Windows VM
crisol repair <id>                             # retry with matching recipes
crisol remove <id>
crisol doctor

crisol app                                     # start/open the local panel in a browser
crisol panel [--port 8765]                     # local feedback panel (127.0.0.1)
crisol feedback stats
crisol recipes import community-recipes/apps    # recursive
tools/validate-recipes.sh
```

## Layout

* `recipes/core.recipe` — built-in recipes; user recipes in
  `~/.config/crisol/recipes/*.recipe` override them by id.
* `community-recipes/` — shared recipes and fixtures; link to the
  [real-log campaign](community-recipes/REAL-LOGS.md).
* `docs/SECURITY.md`, `docs/VM.md`, `docs/CONTRIBUTING.md` — security model, VM
  guide and contribution rules.
* `CHANGELOG.md`, `ROADMAP.md` — what exists and what is pending.

## Honest limits

Silent flags reduce but do not remove clicks. `bubblewrap` is practical
isolation, not a strong security boundary (`--no-sandbox` disables it). The
panel listens only on 127.0.0.1 and enforces a local token plus CSRF/Origin checks.
Crisol does not ship Microsoft DLLs or runtimes, does not install QEMU or
Windows, and never runs `sudo` for you. Disk utilities such as Rufus cannot run
under Wine (no Virtual Disk Service): `crisol launch` blocks them unless you pass
`--force`, which only confirms the crash; use `install --vm` / `launch --vm`
instead. Crisol stages the installer and prints copy-paste command blocks for
both the host shell and the Windows guest, but you
still have to install/boot Windows and run `INSTALAR.cmd` inside the VM.

Licensed under MIT. See [CONTRIBUTING](docs/CONTRIBUTING.md) and the
[code of conduct](CODE_OF_CONDUCT.md).
