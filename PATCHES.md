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
| `grid-fr-carry` | fr tracks sized in layout units with Blink's carried remainder (`ExpandFlexibleTracks`) | `adr:layout-grid-fr-remainder` | fork: `tests/viper_patches.rs` `fr_tracks_carry_their_remainder`, `fr_tracks_never_sum_to_more_than_the_container`; Viper: `viper-layout::grid fr_tracks_carry_their_remainder_as_blink_does` | MIT (Viper's own change; the rule follows Chromium's BSD-licensed `grid_track_sizing_algorithm.cc`, no code copied) | candidate; never offered without the owner |
| `grid-baseline-margin` | a baseline-aligned grid item's used margin is its own, without the baseline shim | `adr:layout-grid-baseline-margin` | fork: `a_baseline_aligned_grid_item_keeps_its_own_margin`; Viper: `viper-layout::alignment a_baseline_aligned_grid_items_margin_is_its_own` | MIT (Viper's own change) | candidate; never offered without the owner |
| `flex-aspect-ratio` | a flex item's automatic sizes follow its aspect ratio from its used main size and from a definite cross size, in the `box-sizing` box | `adr:layout-flex-aspect-ratio` | fork: `a_flexed_items_cross_size_follows_its_ratio`, `a_stretched_cross_size_goes_through_the_ratio_in_the_content_box`, `a_columns_stretched_width_goes_through_the_ratio_in_the_content_box`, `an_aspect_ratio_item_grows_to_its_content_unless_its_minimum_is_set`; Viper: `viper-layout::flex_grid flex_items_size_through_their_aspect_ratio_in_their_box_sizing_box` | MIT; its first commit is the `src/compute/flexbox.rs` change of upstream commit `eb2c9e1a69e9bf79870758535c2920ccbcee4961` by Nico Burns, taken unchanged apart from the markers; its second is Viper's | candidate; never offered without the owner (the upstream part is upstream's own work in progress) |

### `grid-fr-carry`

`src/compute/grid/track_sizing.rs`, the final loop of `expand_flexible_tracks`.
Blink's `GridTrackSizingAlgorithm::ExpandFlexibleTracks`
(`third_party/blink/renderer/core/layout/grid/grid_track_sizing_algorithm.cc`)
keeps the fr size as a float in raw layout units (1/64 px). For each flexible
set in order, `fr_share = fr_size * flex_factor + leftover`; the expanded size
is `LayoutUnit::FromRawValue(fr_share + FLT_EPSILON)`, a saturating
truncation; when it is at least the set's base size the set takes it and
`leftover = max(fr_share - expanded, 0)`. Taffy's fr size in px times 64 is
exactly Blink's raw fr size (scaling by a power of two commutes with
rounding), so only this loop changes: three 1fr tracks of 1232px are
26,282, 26,283 and 26,282 units (410.65625px, 410.671875px, 410.65625px), as
Chrome sizes them, instead of 410.6667px each. Like Blink's, the carry can
lose the last remainder (seventeen 1fr tracks of 10,879 units are 10,878
units: `FLT_EPSILON` is below the float step at 640 units), and the tracks
never sum to more than the free space.

Limit: Blink sizes adjacent tracks that no item edge separates (such as a
`repeat()` of empty tracks) as one set, divided evenly; Taffy has no sets, so
the carry runs per track.

### `grid-baseline-margin`

`src/compute/grid/alignment.rs`, `align_item_within_area`. Upstream returns a
baseline-aligned item's start margin with the shim that moves the item down
to its row's baseline, so the used margin-top it reports holds the shim (an
item without a margin moved 20px reports 20px). The item still moves by the
shim; the reported margin is its own, as a flex item's is and as Chrome
reports it.

### `flex-aspect-ratio`

`src/compute/flexbox.rs`, two commits.

1. **Upstream commit `eb2c9e1a69e9bf79870758535c2920ccbcee4961`** (Nico
   Burns, "Flexbox: fix automatic minimum size and used cross size for
   aspect-ratio items", read on 2026-10-17 on an upstream work-in-progress
   branch):
   its `src/compute/flexbox.rs` change, unchanged apart from the markers. An
   item whose cross size is transferred from its main size gets its used
   cross size from its flexed main size, and the automatic minimum size is
   clamped by the transferred minimum. Its fixtures and `CHANGELOG.md` are
   not taken: every file this branch changes carries a marker.
2. **Viper's change.** A definite cross size (a stretched one included)
   becomes the flex base size through the ratio in the box that `box-sizing`
   names (CSS Box Sizing 4, `aspect-ratio`; Flexbox 1 §9.2 step 3B): an
   80px-tall `content-box` item with a 1px border and `aspect-ratio: 2` is
   158px wide, not 160px. The hypothetical cross size of an item whose cross
   size is transferred comes from its used main size too (or its content, if
   larger where its automatic minimum size is content-based), so its flex
   line is as tall as the item (Flexbox 1 §9.4 step 7): `flex: 0 0 90px;
   aspect-ratio: 1` with a 1px border was 2px tall in its line's cross size.
   The upstream transfer moves to `transferred_cross_size`, shared by both
   steps. The automatic minimum size in the ratio-dependent axis is the
   item's min-content size when `min-<cross>` is `auto` and the item is not
   a scroll container (CSS Box Sizing 4, `aspect-ratio`): content taller
   than the ratio size then makes both the hypothetical and the used cross
   size larger (upstream's used cross size alone cut such an item to its
   ratio size), and an explicit minimum or a scroll container keeps the
   ratio size.

## Upstream work evaluated

Upstream work in progress read on 2026-10-17 for these patches (upstream
`main` had no commit on them):

- `eb2c9e1a69e9bf79870758535c2920ccbcee4961`: taken, as the first commit of
  `flex-aspect-ratio`.
- `4fe98fa9` (`grid-aspect-ratio-height`): a grid container's own aspect
  ratio; no Viper register entry names it. Not taken.
- `a9d478ca` (floats shrink-to-fit): left for Viper's float work. Not taken.

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

## Pin log

Each Viper revision that moves its pin to this branch.

| Viper task | Pinned commit | Change |
| --- | --- | --- |
| V13b-2c1 | `3a41d2dc0942002d7f3ff3c876a0269cd4b885e8` | this ledger only |
| V13b-2c2 | the commit that adds this row | `grid-fr-carry`, `grid-baseline-margin`, `flex-aspect-ratio` |
