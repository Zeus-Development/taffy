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
| `last-baseline` | `last baseline` in `align-items`, `align-self`, `justify-items` and `justify-self` of flex and grid items; block, flex and grid containers report their last baselines | `adr:layout-last-baseline` | fork: `flex_items_align_on_their_last_baselines`, `grid_items_align_on_their_last_baselines`, `a_flex_container_reports_its_last_lines_baseline`, `a_grid_container_reports_its_last_rows_baseline`, `an_item_without_a_baseline_synthesizes_its_last_baseline`, `first_and_last_baseline_items_form_separate_groups`, `a_containers_last_baseline_comes_from_its_last_item_that_has_one`, `grid_items_justify_on_their_last_baselines_by_column`, `flex_column_items_align_on_synthesized_last_baselines`, and the parser tests in `src/style/alignment.rs`; Viper: `viper-layout::alignment flex_and_grid_items_align_on_their_last_baselines` | MIT; adapted from upstream work in progress by Nico Burns (Taffy's maintainer), commits `2d469f25069288b023717e112dc7f4423bf6d0ad` ("Output last baselines from all built-in layout algorithms") and `bbea180faea4baa11d66b64e6fbd1b837d619829` to `7846613e2c27538ff5b586dafc0e5673d0986c85` (last-baseline alignment), attribution kept; re-applied on 0.15.0 with Viper's changes | candidate; never offered without the owner |

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

### `last-baseline`

Two commits. The first adds the keyword (`AlignItemsKeyword::LastBaseline`,
`AlignItems::LAST_BASELINE`, parsed from `last baseline`, `first baseline`
parsed as `baseline`, a serde tag; CSS Box Alignment 3 §4.2) laid out as its
fallback alignment, `safe self-end`, everywhere, with the fork tests, which
fail there. The second lays it out:

- **Containers' last baselines** (`Baselines.last`, which 0.15.0 leaves
  empty for its own algorithms), each rule its specification's first-baseline
  rule with "first" and "last" inverted: a block container's is its last
  in-flow child's that has one (CSS 2.1 §10.8.1, CSS Box Alignment 3 §9.1;
  `src/compute/block.rs`); a flex row's comes from its endmost line, the
  visually last as 0.15.0 takes the visually first for the first baseline:
  the line's last-baseline group's shared baseline, else its first-baseline
  group's, else the last item that has a last baseline, else one synthesized
  from the endmost item's border box; a flex column's from its endmost item
  (CSS Flexbox 1 §8.5); a grid container's from its last row holding an
  item: the row's last-baseline group (items whose areas end in it), else
  the first-baseline group starting in it, else the last item in grid order
  that has a last baseline, grid order traversed from the end, else one
  synthesized from the last item (CSS Grid 2 §11.6). Step 2 of the first
  baselines, a first line's or row's last-baseline group when it has no
  first-baseline group, is added too; without `last baseline` items every
  first baseline is unchanged.
- **Last-baseline sharing groups** (CSS Box Alignment 3 §6.4, §9.2, §9.3):
  in flex, per line (`src/compute/flexbox.rs`), as tall as the largest parts
  above and below the shared baseline; in grid, per row the item's area ends
  in (`src/compute/grid/`), with an end shim beside the first-baseline shim,
  resolved in the same pass, so that the items' distances from their last
  baselines to their margin boxes' bottom edges coincide. The group takes
  its fallback alignment, `safe self-end`, resolved to a physical side
  (§9.3): the bottom of a row, `wrap-reverse` or not. Items with auto margins
  in the axis, or with a block size that depends on their row, do not
  participate, on the terms of first-baseline items.
- **Groups across a column**: `justify-self: last baseline` in grid (per
  column the item's area ends in) and `align-self: last baseline` in a flex
  column. Horizontal text has no baseline in that axis, so one is
  synthesized from the border box at the line-under edge of the
  axis-compatible writing mode, `vertical-lr` or `vertical-rl`: the left edge
  (§9.1). The group lines those edges up and sits at its self-end: the right
  of a left-to-right container, the left of a right-to-left one. In grid an
  inline shim does it: the group's items are laid out end-aligned once, and
  each one's shim is its distance from the group's edge.
- **No group**: absolutely positioned boxes, static positions and
  block-level `justify-self` belong to no shared alignment context and take
  the fallback alignment (§4.2); a lone item is end-aligned.

## Upstream work evaluated

Upstream work in progress read on 2026-10-17 for these patches (upstream
`main` had no commit on them):

- `eb2c9e1a69e9bf79870758535c2920ccbcee4961`: taken, as the first commit of
  `flex-aspect-ratio`.
- `4fe98fa9` (`grid-aspect-ratio-height`): a grid container's own aspect
  ratio; no Viper register entry names it. Not taken.
- `a9d478ca` (floats shrink-to-fit): left for Viper's float work. Not taken.

Read on 2026-10-19 for `last-baseline`:

- `2d469f25069288b023717e112dc7f4423bf6d0ad` (containers' last baselines,
  1 commit) and `bbea180faea4baa11d66b64e6fbd1b837d619829` to
  `7846613e2c27538ff5b586dafc0e5673d0986c85` (the alignment, 4 commits),
  on a base 59 commits older than 0.15.0: adapted, not cherry-picked (they conflict in
  `src/compute/block.rs`, and 0.15.0 still resolves the grid's
  first-baseline shims while the columns are sized). Their test fixtures,
  `CHANGELOG.md` and yoga benchmark helpers are not taken.
- The same work's later fixes, up to
  `c568fe05c3ebf9f3025e829639d7ec62d23ad032` (9 commits): read for cases;
  its rules for reversed directions (the main-end item) and for the grid's
  end-most row are part of the patch. Not taken as commits.

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
| V13b-2c2 | `4241e09fc1543e4aecefb15a8038d56b7625ba77` | `grid-fr-carry`, `grid-baseline-margin`, `flex-aspect-ratio` |
| V13c-1-c3b | the commit that adds this row | `last-baseline` |
