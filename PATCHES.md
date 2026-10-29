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
| `size-containment` | size and inline-size containment: `Contain::SIZE`, `Contain::INLINE_SIZE` and `contain: strict`; a box sized as if it had no content in its contained axes, then its content laid out in that size, in the block, flexbox, grid and leaf algorithms (CSS Containment 2 §3.1, §3.2) | none (no register entry existed) | fork: `a_size_contained_block_is_as_high_as_if_it_were_empty`, `a_size_contained_flex_item_has_no_content_size`, `an_inline_size_contained_box_lays_its_content_out_in_its_empty_width`, `a_size_contained_grid_keeps_its_explicit_tracks`, `a_size_contained_leaf_is_measured_only_in_its_contained_size`, `a_size_contained_boxs_fixed_height_is_a_percentage_basis`, and `parse_contain` in `src/style/mod.rs`; Viper: `viper-layout::containment size_and_inline_size_containment_size_boxes_as_if_empty` | MIT (Viper's own change) | candidate; never offered without the owner |
| `scroll-last-baseline` | `CoreStyle::baseline_source` and `CoreStyle::is_block_container`; a block container that is a block-axis scroll container with `baseline-source: auto` has its last baseline at its block-end margin edge (CSS Box Alignment 3 §9.1) | `adr:layout-scroll-last-baseline` | fork: `a_clipping_blocks_last_baseline_is_its_bottom_margin_edge`, `a_containers_last_baseline_can_be_its_clipping_childs_margin_edge`; Viper: `viper-layout::containment a_scroll_containers_last_baseline_is_its_bottom_margin_edge` | MIT (Viper's own change) | candidate; never offered without the owner |
| `layout-float-alias` | every layout number (computed sizes, positions, offsets, baselines, the available space, known and intrinsic sizes, track sizes and offsets, float segments, scrollable overflow, cache entries) has the type of one alias, `LayoutFloat = f32`, exported at the crate root and in the prelude; style values stay `f32`. Mechanical; behaviour-free; prepares `layout-f64` | `adr:layout-length-cap` (prepares it; later tasks close it) | fork: `layout_numbers_are_the_alias`, `layout_sums_are_still_f32_sums` (since `layout-f64`: `layout_sums_are_exact_beyond_f32`), and the whole suite unchanged; Viper: `viper-layout::api taffy_layout_numbers_are_the_alias` | MIT (Viper's own change) | candidate; never offered without the owner |
| `layout-f64` | the alias becomes `LayoutFloat = f64`: layout arithmetic (sums, positions, offsets, the available space, track sizes, cache keys) is exact for every multiple of 1/64 px up to 2^46; style values stay `f32` and widen exactly where they are resolved; a percentage and a size through an aspect ratio stay upstream's single-precision products, widened; the `grid-fr-carry` loop stays single-precision | `adr:layout-length-cap` (Viper closes it with its own clamp removal) | fork: `layout_numbers_are_the_alias`, `layout_sums_are_exact_beyond_f32`, `many_children_sum_exactly_in_layout_units`, `a_size_through_an_aspect_ratio_is_a_single_precision_product`, and the whole suite unchanged; Viper: `viper-layout::api taffy_layout_numbers_are_the_alias` | MIT (Viper's own change) | candidate; never offered without the owner |
| `subgrid-placement` | subgrids (CSS Grid 2 section 9): `GridContainerStyle::grid_template_subgrid` (a `Style` field) says whether an axis is subgridded; a parent grid gives each subgrid item its spanned tracks' used sizes and their line names through `LayoutGridContainer::set_subgrid_tracks` around the item's final layout; in a subgridded axis the subgrid's explicit tracks are exactly those, its lines carry the parent's names beside its own, its items are clamped into the span, and it is stretched with its sizes ignored. Its items size the parent's tracks through `subgrid-sizing` | `adr:layout-subgrid` (its placement part; V13b-2f3b closes it) | fork: `subgrid::a_subgrids_columns_are_its_parents`, `a_subgrids_edge_items_keep_clear_of_its_margin_border_and_padding`, `a_subgrids_items_are_placed_by_the_parents_and_its_own_line_names`, `an_item_beyond_the_subgrids_span_is_clamped_into_it`, `a_subgrids_rows_are_its_parents`, `a_right_to_left_subgrid_takes_its_parents_columns_in_reverse`, `a_nested_subgrid_takes_the_outer_grids_tracks`, `a_subgrid_outside_a_grid_has_implicit_tracks`, `a_right_to_left_parent_gives_a_left_to_right_subgrid_its_columns_in_reverse`, `a_right_to_left_parent_gives_a_right_to_left_subgrid_its_columns_in_order`, `a_right_to_left_subgrids_start_edge_is_its_right_edge`, `a_subgrid_is_stretched_in_its_subgridded_axis_whatever_its_aspect_ratio`, `a_subgrids_gutters_take_the_parents_distributed_space_exactly`, `items_before_or_overflowing_the_subgrids_span_are_clamped_into_it`; Viper: `viper-layout::subgrid a_subgrids_columns_and_line_names_are_its_parents` | MIT (Viper's own change, from the specification; no browser source read) | candidate; never offered without the owner |
| `subgrid-sizing` | a subgrid's items size its parent's tracks (CSS Grid 2 section 9): the subgrid is empty for sizing in its subgridded axes, and its items, each with extra margins (the subgrid's margin, border and padding at its edge lines and half its gap's difference from the parent's at the others, adding up through nested subgrids), take part in the parent's track sizing in its place; `GridContainerStyle::gap_normal` says whether a subgrid's gap is `normal`, the parent's; a subgrid's own gap narrows or widens the parent's gutters about their centres | `adr:layout-subgrid` (V13b-2f3b closes it) | fork: `subgrid::a_subgrids_items_size_its_parents_tracks`, `a_subgrids_edges_are_extra_margin_on_its_edge_items`, `a_subgrids_own_gap_keeps_the_parents_gutters_centred`, `half_a_subgrids_gap_difference_is_extra_margin_on_its_items`, `a_nested_subgrids_items_size_the_outer_grids_tracks`, `a_subgrids_items_size_its_parents_rows`, `a_subgrids_normal_gap_is_its_parents`, `a_column_subgrids_item_shares_no_baseline_with_the_parents_row`, `a_right_to_left_subgrids_items_size_the_parents_columns_in_reverse`, `a_row_subgrids_item_is_measured_in_its_own_column`, `a_row_subgrids_item_is_measured_in_its_one_own_column`, `a_row_subgrids_item_is_measured_in_its_own_box`; Viper: `viper-layout::subgrid` | MIT (Viper's own change, from the specification; no browser source read) | candidate; never offered without the owner |

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
  (the Flexbox 1 Editor's Draft, "Flex Container Baselines"); a grid container's from its last row holding an
  item: the row's last-baseline group (items whose areas end in it), else
  the first-baseline group starting in it, else the last item in grid order
  that has a last baseline, grid order traversed from the end, else one
  synthesized from the last item (the Grid 2 Editor's Draft, "Grid Container
  Baselines"; both drafts read 2026-10-23: the published CR texts lack step
  3, "the item that has a baseline set"). Step 2 of the first
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
  (§9.3): the bottom of a row, `wrap-reverse` or not; a group taller than
  its line, or an item taller than its area, is start-aligned (the `safe`
  half, §4.4). Items with auto margins
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
  each one's shim is its distance from the group's edge. These inline shims
  are not part of the columns' track sizing (CSS Grid 2 §12.5 shims both
  axes): a group wider than an `auto` column overflows it; Viper's register
  records it.
- **No group**: absolutely positioned boxes, static positions and
  block-level `justify-self` belong to no shared alignment context and take
  the fallback alignment, `safe self-end` (§4.2); a lone item is end-aligned.

### `size-containment`

Two commits, shared with `scroll-last-baseline`: the first adds the flags,
the keywords and the fork tests, which fail there; the second lays them out.

- **The flags.** `Contain::INLINE_SIZE`, and `Contain::SIZE`, which includes
  it; `contain: strict` is `size layout paint`; `size` and `inline-size`
  exclude each other when parsing (CSS Containment 2 §2).
  `Contain::size_contained_axes` gives the contained axes: both under size
  containment, the inline axis under inline-size containment, which is the
  width in Taffy's horizontal axes.
- **Two phases** (§3.1, "sizing as if empty" and "laying out in-place";
  `src/compute/common/containment.rs`). `compute_block_layout`,
  `compute_flexbox_layout`, `compute_grid_layout` and `compute_leaf_layout`
  first run their own algorithm as a size computation without the box's
  children (a leaf without calling its measure function), in a formatting
  context of the box's own, when a contained axis has no known size. Each
  such axis then takes that size as a known, definite dimension, and the
  algorithm runs again with the children, so they are laid out in the fixed
  size and may overflow it. The box's own properties keep their effect (its
  sizes, minimums, padding, border, aspect ratio and a grid's explicit
  tracks), and so does stretching, which does not depend on content. In
  every intrinsic size the box contributes only that size, so a contained
  item adds nothing of its content to a flex base size, an automatic
  minimum size or a grid track's size (the note of §3.1). A box contained in
  the inline axis only is laid out in its empty width, and its height comes
  from its content as usual (§3.2).
- **Not in the fork:** which boxes containment applies to (§3.1's list:
  boxes without a principal box, tables, internal table and ruby boxes,
  non-atomic inline boxes) is the caller's style; replaced elements' natural
  sizes are the caller's measure function's; `contain-intrinsic-size` is not
  read.

### `scroll-last-baseline`

`CoreStyle::baseline_source` (the CSS `baseline-source` property, a
`Style` field, `auto` by default) and `CoreStyle::is_block_container`
(`Style`: `display: block` or `flow-root`, not a replaced element or a table). CSS Box
Alignment 3 §9.1 gives a block container that is a block-axis scroll
container and whose `baseline-source` is `auto` a last baseline set at its
block-end margin edge, "for legacy reasons". The parent algorithms, which
already clamp a scroll container's baselines to its border box, use that
edge for its last baseline instead (`last_baseline_at_margin_edge`, read
once per item): a block container's last baseline from its last in-flow
child, flex and grid last-baseline sharing groups (the item's last baseline has no
descent below it), and flex and grid containers' last baselines
from their items. First baselines and other boxes are unchanged.
`tests/xml.rs` reads the new field from a `baseline-source` attribute.

### `layout-float-alias`

`pub type LayoutFloat = f32;` in `src/lib.rs`, also in the prelude. Every
`f32` that carries a computed layout number has that type instead: the
`Size`, `Point`, `Rect` and `Line` values of layout inputs, outputs and
results (`Layout`, `LayoutInput`, `LayoutOutput`), `AvailableSpace::Definite`,
known dimensions, intrinsic and content sizes, baselines, track sizes and
offsets, float segments, scrollable overflow, the cache's entries, the
`MaybeMath` and `MaybeResolve` helpers over layout numbers, the context and
results of resolving a length against a basis (`resolve_to_option`,
`resolve_calc_value`), and the `no_std` helpers in `src/util/sys.rs`.

Style values stay `f32`: `CompactLength`'s stored value, the constructors of
`Dimension`, `LengthPercentage`, `LengthPercentageAuto` and the track sizing
functions, `style_helpers`, flex grow and shrink factors and their sums, fr
factors, aspect ratios and `scrollbar_width`. Comments, the bit packing of
the cache keys and the unit tests inside `src/` are unchanged.

The change is mechanical and changes no value: the alias is `f32`. The
whole fork suite passes unchanged, and so do Viper's suites with the new
pin. It is far above the 100-line target of the rules above (about 660
changed lines, each with a marker on its hunk), because it is one type
substitution across the layout code; it makes the later change of the
alias's type one line plus the conversions where style values meet layout
numbers. It is applied with Taffy's own formatting (`cargo fmt`).

### `layout-f64`

`pub type LayoutFloat = f64;` in `src/lib.rs`. With `layout-float-alias`
in place, this is the alias's type plus the places where an `f32` style
value meets a layout number:

- `CompactLength::layout_value()` widens a stored length exactly (every
  `f32` is an `f64`); every length resolve uses it.
- `CompactLength::percent_of(basis)` resolves a percentage as upstream's
  single-precision product of the fraction and the basis, widened
  afterwards. An `f64` product changes rounded results (1% of 50px is
  0.5 in `f32` and 0.49999999 in `f64`, since `0.01f32` is 0.0099999998),
  and 20 of the XML fixture tests fail with it; the single-precision
  product keeps every resolved percentage as it was.
- A size transferred through an aspect ratio (`maybe_apply_aspect_ratio`,
  the leaf's height, a flex item's ratio-derived flex basis and main size)
  is upstream's single-precision product or quotient, widened
  (`geometry::ratio_mul`, `ratio_div`): 70px through 50 / 35 is 100px in
  `f32`, and the `f64` product of the `f32` ratio is 100.0000024px, which
  a box's offset then truncates one unit short (a Viper corpus case shows
  it).
- Flex grow and shrink factors, fr factors and `scrollbar_width` widen
  where they meet layout numbers; a `FlexItem`'s factors are layout
  numbers.
- `grid-fr-carry`'s loop keeps its fr size, shares and remainders in `f32`
  with `f32::EPSILON`, as that patch defines them; only the expanded size, a
  whole number of layout units, widens. So percentages, fr shares and
  sizes through an aspect ratio are exact only as far as `f32` reaches: above 2^18 px a result may be off by
  half an `f32` step (1/64 px just above 2^18 px, up to 1 px near 2^25 px;
  `width: 100%` of a 300,000.015625px container is 300,000px), as upstream's
  are. Sums, positions and every other layout number are exact.
- `AvailableSpace::is_roughly_equal` compares within `LayoutFloat::EPSILON`
  (`layout-float-alias`'s line), now `f64`'s; nothing in Taffy or Viper
  calls it.
- The cache keys pack two `f64`s in a `u128` (`DimBits = u64`,
  `KeyBits = u128`), as upstream packs two `f32`s in a `u64`; any narrower
  packing would let two sizes share a key.
- Casts that became same-type casts are removed (`float.rs`, the line
  balancing in `flexbox.rs`, whose `to_size` clamp stays at `f32::MAX`);
  `Size<Dimension>::from_lengths` and `from_percent` take `f32` again, as
  the base's style constructors do.

Tests, examples and the `cfg(test)` modules take the alias's type. The
whole fork suite passes unchanged apart from the two alias proofs of
`layout-float-alias`, which now assert `f64` and an exact sum. About 280
added and 150 removed lines under `src/` (markers and comments included),
above the 100-line target, because each meeting point of a style value
and a layout number is its own hunk with its own marker.

### `subgrid-placement`

CSS Grid 2 section 9, its first part (V13b-2f3a); `subgrid-sizing`
(V13b-2f3b) adds the subgrid's items to the parent's track sizing.

- **The style.** `GridContainerStyle::grid_template_subgrid(axis)`, `false`
  by default, forwarded by the `&T` implementation; `Style` answers from its
  field `grid_template_subgrid` (`x` the columns, `y` the rows). In a
  subgridded axis the track template is ignored and the existing line-name
  methods give the subgrid's own `<line-name-list>`, one set per line with
  repetitions expanded.
- **The channel** (`src/tree/traits.rs`). Taffy has no input from a parent's
  layout to a child's but the tree: `LayoutInput` is a cache key, and a child
  is laid out through the host's `compute_child_layout`. So
  `LayoutGridContainer` gains `set_subgrid_tracks` and `subgrid_tracks`,
  optional like `set_detailed_grid_info` (a no-op and `None`). A parent grid
  stores a `SubgridTracks` (`src/compute/grid/subgrid.rs`, exported) for a
  subgrid item just before the item's final layout and takes it back just
  after; a host that caches layouts drops the item's cache when the value
  changes. So the take-back drops the entry of the final layout just
  computed, and each final layout of a subgrid reruns its own grid pass; its
  children stay cached, so the cost is bounded. In an axis it does not
  subgrid, the item's intrinsic sizes during the parent's track sizing are
  measured without it, as an ordinary grid's.
- **What the parent gives** (per subgridded axis): the used sizes of the
  spanned tracks and of the gutters between them, alternating; a gutter is the
  space between its tracks' used positions, so content distribution's space
  is in it. And the names of the `n + 1` lines: the parent's named lines
  (its own subgrid names included, so nesting works level by level) and the
  `<area>-start` and `<area>-end` names of its areas' edges. A subgrid whose
  inline direction runs against its parent's takes the columns and their
  names in reverse.
- **The subgrid's grid** (`compute_grid_layout`, when the tree holds tracks
  for the node). In a subgridded axis: the explicit track count is the span,
  with no repetition and whatever its own areas; its lines take the parent's
  names beside its own; its items are placed as usual (hypothetical implicit
  lines resolve their placement) and then clamped into the span as an
  overlarge grid clamps them, since the axis has no implicit tracks; its
  tracks are fixed at exactly the given sizes (`GridTrack::exact`, which track
  sizing keeps), its first track less its margin, border and padding at its
  start and its last less those at its end, never below zero (and with a
  gap of its own, `subgrid-sizing`), so its lines
  fall on the parent's and its edge items keep clear of those edges. The
  parent lays the subgrid out stretched in that axis, its `align-self` or
  `justify-self` and its sizes there ignored, and no aspect ratio transfers a
  size into it.
- **Not yet:** baseline sharing across the subgrid; `repeat(auto-fill, ...)`
  in a `<line-name-list>` is the caller's to expand; an orthogonal subgrid is
  the caller's to give transposed tracks; `TaffyTree` does not store subgrid
  tracks yet.
- **Size.** About 250 changed code lines (the ledger's target is 100):
  the task's plan allows the fork 500.

### `subgrid-sizing`

CSS Grid 2 section 9, its second part (V13b-2f3b): a subgrid's items size
its parent's tracks.

- **The subgrid is empty for sizing** in its subgridded axes: the parent's
  track sizing skips it there (`GridItem::sizes_tracks`), and sizes it as
  any item in an axis it does not subgrid.
- **Its items take its place.** After placement and before track sizing,
  the parent runs each subgrid item's grid pass in a collecting mode
  (`grid_layout`'s `collect`): the subgrid places its items in stand-in
  tracks of the span (the parent's line names and gap, zero sizes), collects
  its own subgrid items' items the same way, and hands them all back without
  laying anything out. The parent moves them into its own lines (in reverse
  when the subgrid's columns run against its own; in an axis the subgrid
  does not subgrid, onto its area there, where they size nothing) and sizes
  its tracks with them as with its own items, until track alignment, where
  it drops them. Only items that size some axis are kept.
- **Extra margins** (`GridItem::subgrid_margin`, added to the item's margins
  wherever track sizing reads them): at the subgrid's edge lines its margin,
  border and padding there (in its own order of the axis), at its other
  lines half its gap's difference from the parent's; in an axis it does not
  subgrid, all of them. Its scrollbar gutter counts with its padding.
  Through nested subgrids they add up level by level.
- **Baselines.** Only items that size the parent's rows share its rows'
  baselines (`resolve_baseline_group_shims`): the items of a subgrid of its
  columns keep theirs to its own rows.
- **Gaps.** `GridContainerStyle::gap_normal`, `false` in both axes by
  default, says whether a container's gap is `normal`; a `normal` gap in a
  subgridded axis is the parent's (its gap value, `SubgridAxis::gap`, without
  content distribution's space). With a gap of its own a subgrid's gutters
  are the parent's less the difference, and its tracks beside each gutter
  take half of it, so its gutters stay centred on the parent's. `Style`
  answers from its `gap_normal` field, `false` in both axes by default.
- **Not yet:** a subgrid's item measured in the parent's sizing resolves its
  percentages against the parent's content box, not the subgrid's, and so do
  the subgrid's own percentage margin, border and padding read in the
  collecting mode for the extra margins. The items of a subgrid of its rows
  join the parent's row baseline groups in sizing, but the subgrid's own
  layout does not align them with the parent's items. A subgrid of one
  axis hands over its items only when, in the other axis, it is itself
  stretched there with no size of its own (its size, minimum and maximum
  `auto`, no aspect ratio, self-alignment `normal` or `stretch`, no `auto`
  margin) and it has at most one track of its own that fills its area
  (its maximum is `fr`, or `auto` with the subgrid's content alignment
  there `normal` or `stretch`): its items are measured across the
  subgrid's whole area less its insets. Otherwise it sizes the parent's tracks as an ordinary item,
  as before this patch, and hands over no items: they would have to be
  measured against its own tracks, which are sized only after the
  parent's (section 12.1). V13b-2f3c sizes them in their own tracks. The
  rest of section 9's "not yet" list under `subgrid-placement`.
- **Size.** About 300 changed code lines; the task's plan allows the fork
  500.

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
| V13c-1-c3b | `66999e06cb9d957facb28782db6ea1e5977a4a62` | `last-baseline` |
| V10b-10s | `ec90f57687866cc4e54054147d7629f29a941bdc` | `size-containment`, `scroll-last-baseline` |
| V13b-2i1a | the last commit of the range that adds this row | `layout-float-alias` |
| V13b-2i1b | the last commit of the range that adds this row | `layout-f64` |
| V13b-2f3a | the last commit of the range that adds this row | `subgrid-placement` |
| V13b-2f3b | the last commit of the range that adds this row | `subgrid-sizing` |
