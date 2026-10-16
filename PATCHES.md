# Viper's Taffy fork: patch ledger

This repository is a fork of [Taffy](https://github.com/DioxusLabs/taffy) kept
by the Viper project. Its `viper/*` branches carry a small set of patches on
top of exact Taffy release commits; `main` and every other branch are not
Viper's and follow their own history. Viper builds `taffy` from one exact
commit of a `viper/*` branch, through a Cargo `[patch.crates-io]` entry.

This file is the canonical ledger of those patches. It holds the verified
base, one row and one section per patch, the fork-side rebase procedure and
the rebase log. Viper's own record of the fork (how the pin moves, the gate
step that checks it, the cargo-deny source and the licence obligations) is
its ADR 0028, "The Taffy fork".

Base: eb6450694f33e2b7413e40677f230ff63f0fc194
Branch: viper/taffy-0.15.0

## Rules

- **Branches.** One long-lived branch per Taffy release,
  `viper/taffy-<version>`, created at the commit the published crate names.
  A `viper/*` branch is never force-pushed, and never deleted while any Viper
  revision pins a commit on it (a repository ruleset blocks both).
- **Markers.** Every changed hunk carries `VIPER-PATCH(<id>)` in a comment
  (`//` in Rust, `#` in TOML), with `<id>` matching `[a-z0-9-]+` and naming a
  row of the ledger below. The comment states the Chrome behaviour matched,
  or the problem fixed, and why upstream differs, with a specification or
  Chromium link where one exists. Every file that differs from the base,
  apart from this one, carries a marker. Viper's gate step `taffy-fork`
  fails when the marker ids in the pinned commit differ from the ledger ids,
  or when a changed file carries no marker.
- **Size.** Each patch is small (target at most 100 changed code lines) and
  self-contained, one commit per patch where possible.
- **Tests.** Every patch has tests in the fork (listed in its row) and a
  proof in Viper's gate: named Viper tests that must pass in the gate's run.
- **Upstream.** A generic patch may be offered upstream later; nothing goes
  to any DioxusLabs repository without the Viper owner's go-ahead for that
  specific draft.
- **Licences.** Taffy is MIT-licensed (`LICENSE`, copyright Visly Inc. and the
  Taffy Authors); patched files keep their notices, the licence
  file is unchanged, and each row names the licence of its change and the
  upstream code it is derived from.
- **Commits** use the Viper owner's GitHub noreply identity.

## Base

`viper/taffy-0.15.0` starts at Taffy commit
`eb6450694f33e2b7413e40677f230ff63f0fc194` ("Prepare for v0.15 release
(#1302)"). The published crate was verified against it on 2026-10-16:

| Crate | Published `.crate` SHA-256 (equals the crates.io index `cksum`) | `.cargo_vcs_info.json` | Sources compared with the commit |
| --- | --- | --- | --- |
| `taffy` 0.15.0 | `3c8480887e8803986d96a7173729565520d4bbb721ad6b7ad6941a22897207aa` | `sha1` `eb6450694f33e2b7413e40677f230ff63f0fc194`, `path_in_vcs` empty (the repository root) | identical: all 63 packaged files (`src/`, `examples/`, `README.md`); `Cargo.toml.orig` = the root `Cargo.toml`. Generated at packaging time and not compared: the normalized `Cargo.toml`, `Cargo.lock`, `.cargo_vcs_info.json`. The `include` list leaves out the rest of the repository, and `examples/parley`, a package of its own |

## Ledger

| Id | Reason | Register entry | Tests | Licence and upstream attribution | Upstream status |
| --- | --- | --- | --- | --- | --- |

No patch yet: the branch's only change from the base is this file.

## Rebase procedure (fork side)

Viper's ADR 0028 holds the whole procedure; these are its fork-side steps.

1. **Trigger.** A new Taffy release that Viper adopts; a soundness fix may
   trigger one earlier.
2. **Verify the base**: the published `.crate` SHA-256 against the crates.io
   index, its `.cargo_vcs_info.json` naming the release commit, and its
   unpacked sources identical to that commit, apart from the files generated
   at packaging time. Record the result in the "Base" section.
3. **Create the branch** `viper/taffy-<version>` at that commit and
   cherry-pick the patch commits in ledger order, keeping the markers. A
   patch that upstream made unnecessary is retired: its row moves to a
   "Retired" table with the upstream commit, and its Viper proof must still
   pass without it. A patch that no longer applies is reworked as a new
   commit with the same id.
4. **Test.** `cargo test` in the fork and each patch's fork tests. Update
   this file: `Base:`, `Branch:`, the base, the ledger and the rebase log,
   with conflicts and effort in hours.
5. **Review.** An independent review of
   `git range-diff <old base>..<old pin> <new base>..<new pin>`.
6. **Push the new branch.** Never force-push; never delete a branch that
   holds a commit any Viper revision pins.

## Rebase log

| Date | From → to | Patches carried / retired | Conflicts | Effort (hours) | Reviewer | Viper PR |
| --- | --- | --- | --- | --- | --- | --- |
