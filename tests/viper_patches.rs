//! Tests of the Viper fork's patches (see `PATCHES.md`), through `TaffyTree`. Each test sits under its patch's
//! marker comment and fails without that patch. Sizes are worked out by hand from Chrome's rules; a
//! layout unit is 1/64 px, Chrome's `LayoutUnit`. Taffy's rounding to whole pixels is disabled: Viper lays out without it.

use taffy::prelude::*;
use taffy::{LayoutFloat, Overflow, Point};

/// A content-box leaf with a 1px border on every side.
fn bordered(style: Style) -> Style {
    let border = Rect { left: length(1.0), right: length(1.0), top: length(1.0), bottom: length(1.0) };
    Style { box_sizing: BoxSizing::ContentBox, border, ..style }
}

// VIPER-PATCH(grid-fr-carry): fr tracks are sized in layout units as Blink's `ExpandFlexibleTracks` sizes them,
// each truncated with its remainder carried to the next flexible track.

/// The used widths, in layout units, of `count` 1fr columns of a grid `width` px wide with one item per column.
// VIPER-PATCH(layout-f64): layout numbers of the alias type.
fn fr_columns_in_units(width: f32, count: usize) -> Vec<LayoutFloat> {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    taffy.disable_rounding();
    let items: Vec<NodeId> = (0..count)
        .map(|_| taffy.new_leaf(Style { size: Size { width: auto(), height: length(5.0) }, ..Default::default() }))
        .collect::<Result<_, _>>()
        .unwrap();
    let grid = Style {
        display: Display::Grid,
        size: Size { width: length(width), height: auto() },
        grid_template_columns: (0..count).map(|_| fr(1.0)).collect(),
        ..Default::default()
    };
    let root = taffy.new_with_children(grid, &items).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    items.iter().map(|item| taffy.layout(*item).unwrap().size.width * 64.0).collect()
}

/// Three 1fr tracks of 1232px (78,848 units, 26,282.67 a track) and of 752px (48,128 units, 16,042.67 a track),
/// Chrome's sizes in the development corpus case grid-17-item-margins.
#[test]
fn fr_tracks_carry_their_remainder() {
    assert_eq!(fr_columns_in_units(1232.0, 3), [26_282.0, 26_283.0, 26_282.0]);
    assert_eq!(fr_columns_in_units(752.0, 3), [16_042.0, 16_043.0, 16_043.0]);
}

/// Seventeen 1fr tracks of 10,879 units (639.94 a track) are 639, fifteen of 640 and 639 again: 10,878 units,
/// never more than the container. Blink loses the last remainder too: the `f32` share is 639.99998, and
/// `FLT_EPSILON` is below the float step at 640.
#[test]
fn fr_tracks_never_sum_to_more_than_the_container() {
    let columns = fr_columns_in_units(10_879.0 / 64.0, 17);
    let mut expected = vec![639.0];
    expected.extend([640.0; 15]);
    expected.push(639.0);
    assert_eq!(columns, expected);
    // VIPER-PATCH(layout-f64): layout numbers of the alias type.
    assert_eq!(columns.iter().sum::<LayoutFloat>(), 10_878.0);
}

// VIPER-PATCH(grid-baseline-margin): a baseline-aligned grid item's used margin is its own; the baseline shim
// only moves it.

/// Two items without a baseline, synthesized from their bottom edges: the 20px-high one moves down 20px to the
/// 40px-high one's baseline, and its margin-top stays 0.
#[test]
fn a_baseline_aligned_grid_item_keeps_its_own_margin() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    taffy.disable_rounding();
    let high =
        taffy.new_leaf(Style { size: Size { width: auto(), height: length(40.0) }, ..Default::default() }).unwrap();
    let low =
        taffy.new_leaf(Style { size: Size { width: auto(), height: length(20.0) }, ..Default::default() }).unwrap();
    let grid = Style {
        display: Display::Grid,
        align_items: AlignItems::BASELINE,
        grid_template_columns: vec![length(100.0), length(100.0)],
        ..Default::default()
    };
    let root = taffy.new_with_children(grid, &[high, low]).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let low = taffy.layout(low).unwrap();
    assert_eq!(low.location.y, 20.0);
    assert_eq!(low.margin.top, 0.0);
    assert_eq!(taffy.layout(high).unwrap().margin.top, 0.0);
}

// VIPER-PATCH(flex-aspect-ratio): a flex item's automatic cross size follows its aspect ratio from its used main
// size, also in its line's cross size, and a definite cross size goes through the ratio in the box that
// `box-sizing` names (the development corpus case flex-20-aspect-ratio-items).

/// Lays out `children` in a flex container with `container`'s style and returns their border boxes as
/// `(x, y, width, height)`, then the container's.
// VIPER-PATCH(layout-f64): layout numbers of the alias type.
fn flex_boxes(container: Style, children: &[Style]) -> Vec<(LayoutFloat, LayoutFloat, LayoutFloat, LayoutFloat)> {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    taffy.disable_rounding();
    let items: Vec<NodeId> =
        children.iter().map(|style| taffy.new_leaf(style.clone())).collect::<Result<_, _>>().unwrap();
    let root = taffy.new_with_children(Style { display: Display::Flex, ..container }, &items).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    items
        .iter()
        .chain([&root])
        .map(|node| {
            let layout = taffy.layout(*node).unwrap();
            (layout.location.x, layout.location.y, layout.size.width, layout.size.height)
        })
        .collect()
}

/// `flex: 0 0 90px; aspect-ratio: 1` with a 1px border is 92px high, and `flex: 1 1 0; aspect-ratio: 3` takes
/// 400 - 92 - 2 = 306px of content width, 102px high; the line, and so the container, is 104px high.
#[test]
fn a_flexed_items_cross_size_follows_its_ratio() {
    let start = Style {
        align_items: AlignItems::FLEX_START,
        size: Size { width: length(400.0), height: auto() },
        ..Default::default()
    };
    let fixed = bordered(Style {
        flex_grow: 0.0,
        flex_shrink: 0.0,
        flex_basis: length(90.0),
        aspect_ratio: Some(1.0),
        ..Default::default()
    });
    let grown = bordered(Style {
        flex_grow: 1.0,
        flex_shrink: 1.0,
        flex_basis: length(0.0),
        aspect_ratio: Some(3.0),
        ..Default::default()
    });
    let boxes = flex_boxes(start, &[fixed, grown]);
    assert_eq!(boxes, [(0.0, 0.0, 92.0, 92.0), (92.0, 0.0, 308.0, 104.0), (0.0, 0.0, 400.0, 104.0)]);
}

/// Stretched to 80px, a content-box item with a 1px border has a 78px content box: `aspect-ratio: 2` makes it 156px
/// wide (158px with the border), `1 / 2` 39px (41px).
#[test]
fn a_stretched_cross_size_goes_through_the_ratio_in_the_content_box() {
    let stretch = Style { size: Size { width: auto(), height: length(80.0) }, ..Default::default() };
    let wide = bordered(Style { aspect_ratio: Some(2.0), ..Default::default() });
    let narrow = bordered(Style { aspect_ratio: Some(0.5), ..Default::default() });
    let boxes = flex_boxes(stretch, &[wide, narrow]);
    assert_eq!(boxes[..2], [(0.0, 0.0, 158.0, 80.0), (158.0, 0.0, 41.0, 80.0)]);
}

/// A column's stretched 300px is 298px of content: `aspect-ratio: 4` makes it 74.5px high (76.5px with the border).
#[test]
fn a_columns_stretched_width_goes_through_the_ratio_in_the_content_box() {
    let column = Style {
        flex_direction: FlexDirection::Column,
        size: Size { width: length(300.0), height: auto() },
        ..Default::default()
    };
    let item = bordered(Style { aspect_ratio: Some(4.0), ..Default::default() });
    let boxes = flex_boxes(column, &[item]);
    assert_eq!(boxes, [(0.0, 0.0, 300.0, 76.5), (0.0, 0.0, 300.0, 76.5)]);
}

/// The height of `flex: 0 0 50px; aspect-ratio: 1` holding content 100px high, in a 400px row aligned as `align`,
/// with `item`'s extra style; the container's height second.
// VIPER-PATCH(layout-f64): layout numbers of the alias type.
fn tall_content_item(align: AlignItems, item: Style) -> (LayoutFloat, LayoutFloat) {
    let mut taffy: TaffyTree<LayoutFloat> = TaffyTree::new();
    taffy.disable_rounding();
    let style = Style { flex_grow: 0.0, flex_shrink: 0.0, flex_basis: length(50.0), aspect_ratio: Some(1.0), ..item };
    let leaf = taffy.new_leaf_with_context(style, 100.0).unwrap();
    let row = Style {
        display: Display::Flex,
        align_items: align,
        size: Size { width: length(400.0), height: auto() },
        ..Default::default()
    };
    let root = taffy.new_with_children(row, &[leaf]).unwrap();
    taffy
        .compute_layout_with_measure(root, Size::MAX_CONTENT, |input, _, height, _| {
            let content = height.map_or(0.0, |height| *height);
            let size = Size {
                width: input.known_dimensions.width.unwrap_or(10.0),
                height: input.known_dimensions.height.unwrap_or(content),
            };
            taffy::LayoutOutput::from_outer_size(size)
        })
        .unwrap();
    (taffy.layout(leaf).unwrap().size.height, taffy.layout(root).unwrap().size.height)
}

/// An item whose content is taller than its ratio size grows to it: its automatic minimum size in the
/// ratio-dependent axis is its min-content size (CSS Box Sizing 4, `aspect-ratio`), in its line and in its used
/// size. An explicit `min-height` or a scroll container keeps the ratio size.
#[test]
fn an_aspect_ratio_item_grows_to_its_content_unless_its_minimum_is_set() {
    for align in [AlignItems::FLEX_START, AlignItems::CENTER] {
        assert_eq!(tall_content_item(align, Style::default()), (100.0, 100.0));
        let zero = Style { min_size: Size { width: auto(), height: length(0.0) }, ..Default::default() };
        assert_eq!(tall_content_item(align, zero), (50.0, 50.0));
        let hidden = Style { overflow: Point { x: Overflow::Hidden, y: Overflow::Hidden }, ..Default::default() };
        assert_eq!(tall_content_item(align, hidden), (50.0, 50.0));
    }
}

// VIPER-PATCH(last-baseline): `align-items` and `align-self: last baseline` in flex and grid containers; block, flex
// and grid containers report their last baselines (CSS Box Alignment 3 §9, and the Flexbox 1 and Grid 2 Editor's
// Drafts' container baselines, read 2026-10-23).

/// A line of text, 20px high with its baseline 16px down: a flex row holding a 10 × 16 leaf, whose baseline is
/// synthesized from the leaf's bottom edge.
fn text_line(taffy: &mut TaffyTree<()>) -> NodeId {
    let glyphs =
        taffy.new_leaf(Style { size: Size { width: length(10.0), height: length(16.0) }, ..Default::default() });
    let line =
        Style { display: Display::Flex, size: Size { width: auto(), height: length(20.0) }, ..Default::default() };
    taffy.new_with_children(line, &[glyphs.unwrap()]).unwrap()
}

/// A block of `lines` text lines with `padding_bottom`: its last baseline is 20 × (lines - 1) + 16px down.
fn paragraph(taffy: &mut TaffyTree<()>, lines: usize, padding_bottom: f32, align_self: Option<AlignSelf>) -> NodeId {
    let children: Vec<NodeId> = (0..lines).map(|_| text_line(taffy)).collect();
    let padding = Rect { left: zero(), right: zero(), top: zero(), bottom: length(padding_bottom) };
    let style = Style { display: Display::Block, padding, align_self, ..Default::default() };
    taffy.new_with_children(style, &children).unwrap()
}

/// Lays out `root` and returns the `(y, height)` of each of `nodes`.
// VIPER-PATCH(layout-f64): layout numbers of the alias type.
fn heights(taffy: &mut TaffyTree<()>, root: NodeId, nodes: &[NodeId]) -> Vec<(LayoutFloat, LayoutFloat)> {
    taffy.disable_rounding();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    nodes
        .iter()
        .map(|node| taffy.layout(*node).map(|layout| (layout.location.y, layout.size.height)).unwrap())
        .collect()
}

/// A container of `display` whose items align on their last baselines.
fn last_baseline_container(display: Display) -> Style {
    let columns = if display == Display::Grid { vec![auto(), auto(), auto()] } else { Vec::new() };
    Style { display, align_items: AlignItems::LAST_BASELINE, grid_template_columns: columns, ..Default::default() }
}

/// One line with 10px of padding below (last baseline 16, 14px below it) and three lines (last baseline 56, 4px
/// below it): the group is 56 + 14 = 70px high, and each item's last baseline is 14px above its end, so the
/// one-line item is 70 - 14 - 16 = 40px down. Aligned at the end instead, the row would be 60px and the item 30px
/// down.
#[test]
fn flex_items_align_on_their_last_baselines() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let one = paragraph(&mut taffy, 1, 10.0, None);
    let three = paragraph(&mut taffy, 3, 0.0, None);
    let root = taffy.new_with_children(last_baseline_container(Display::Flex), &[one, three]).unwrap();
    assert_eq!(heights(&mut taffy, root, &[one, three, root]), [(40.0, 30.0), (0.0, 60.0), (0.0, 70.0)]);
}

/// The same two items in one grid row: the row is 70px high and the one-line item 40px down.
#[test]
fn grid_items_align_on_their_last_baselines() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let one = paragraph(&mut taffy, 1, 10.0, None);
    let three = paragraph(&mut taffy, 3, 0.0, None);
    let root = taffy.new_with_children(last_baseline_container(Display::Grid), &[one, three]).unwrap();
    assert_eq!(heights(&mut taffy, root, &[one, three, root]), [(40.0, 30.0), (0.0, 60.0), (0.0, 70.0)]);
}

/// A wrapping flex container 10px wide holding a one-line and a two-line block, on two lines, with 5px of padding
/// below: its last baseline is its last line's last item's, 20 + 36 = 56px down, 9px above its end. Beside it, a
/// one-line block with 20px of padding below (24px below its baseline): the group is 56 + 24 = 80px high.
#[test]
fn a_flex_container_reports_its_last_lines_baseline() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let first = paragraph(&mut taffy, 1, 0.0, None);
    let second = paragraph(&mut taffy, 2, 0.0, None);
    let padding = Rect { left: zero(), right: zero(), top: zero(), bottom: length(5.0) };
    let nested = Style {
        display: Display::Flex,
        flex_wrap: FlexWrap::Wrap,
        padding,
        size: Size { width: length(10.0), height: auto() },
        ..Default::default()
    };
    let nested = taffy.new_with_children(nested, &[first, second]).unwrap();
    let beside = paragraph(&mut taffy, 1, 20.0, None);
    let root = taffy.new_with_children(last_baseline_container(Display::Flex), &[nested, beside]).unwrap();
    assert_eq!(heights(&mut taffy, root, &[nested, beside, root]), [(0.0, 65.0), (40.0, 40.0), (0.0, 80.0)]);
}

/// A grid container's last baseline is its last row's: a grid of two rows, a one-line and a two-line block, has
/// its last baseline 20 + 36 = 56px down, 4px above its end; beside it a one-line block with 20px of padding below.
#[test]
fn a_grid_container_reports_its_last_rows_baseline() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let first = paragraph(&mut taffy, 1, 0.0, None);
    let second = paragraph(&mut taffy, 2, 0.0, None);
    let nested = Style { display: Display::Grid, ..Default::default() };
    let nested = taffy.new_with_children(nested, &[first, second]).unwrap();
    let beside = paragraph(&mut taffy, 1, 20.0, None);
    let root = taffy.new_with_children(last_baseline_container(Display::Flex), &[nested, beside]).unwrap();
    assert_eq!(heights(&mut taffy, root, &[nested, beside, root]), [(0.0, 60.0), (40.0, 40.0), (0.0, 80.0)]);
}

/// An item without a baseline synthesizes its last baseline from its border box's bottom edge: a 30px leaf (0 below
/// its baseline) beside a one-line block with 10px of padding below (14px below): 30 + 14 = 44px, the block 14px
/// down. Aligned at the end, the row would be 30px high.
#[test]
fn an_item_without_a_baseline_synthesizes_its_last_baseline() {
    for display in [Display::Flex, Display::Grid] {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let leaf = Style { size: Size { width: length(10.0), height: length(30.0) }, ..Default::default() };
        let leaf = taffy.new_leaf(leaf).unwrap();
        let one = paragraph(&mut taffy, 1, 10.0, None);
        let root = taffy.new_with_children(last_baseline_container(display), &[leaf, one]).unwrap();
        assert_eq!(heights(&mut taffy, root, &[leaf, one, root]), [(0.0, 30.0), (14.0, 30.0), (0.0, 44.0)]);
    }
}

/// `first baseline` and `last baseline` items on one line form two groups: the three-line `baseline` item stays at
/// the start, and the two `last baseline` items share a group 56 + 14 = 70px high.
#[test]
fn first_and_last_baseline_items_form_separate_groups() {
    for display in [Display::Flex, Display::Grid] {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let first = paragraph(&mut taffy, 3, 0.0, Some(AlignSelf::BASELINE));
        let one = paragraph(&mut taffy, 1, 10.0, None);
        let three = paragraph(&mut taffy, 3, 0.0, None);
        let root = taffy.new_with_children(last_baseline_container(display), &[first, one, three]).unwrap();
        let expected = [(0.0, 60.0), (40.0, 30.0), (0.0, 60.0), (0.0, 70.0)];
        assert_eq!(heights(&mut taffy, root, &[first, one, three, root]), expected);
    }
}

/// A container of `display` holding a one-line block and, after it, a 10 × 40 leaf without a baseline. Its last
/// baseline is the block's, 16px down: the last item of its last line or row that has a baseline set (CSS Flexbox 1 and
/// CSS Grid 2 Editor's Drafts, "Flex Container Baselines" and "Grid Container Baselines", read 2026-10-23, step 3 with
/// "first" and "last" inverted), not one synthesized from the leaf, which comes last. Beside it, a one-line block with
/// 30px of padding below (34px below its baseline): the group is 16 + 34 = 50px high and both items sit at the top.
/// From the leaf's bottom edge, the group would be 40 + 34 = 74px high; aligned at the end, the container would be 10px
/// down.
#[test]
fn a_containers_last_baseline_comes_from_its_last_item_that_has_one() {
    for display in [Display::Flex, Display::Grid] {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let block = paragraph(&mut taffy, 1, 0.0, None);
        let leaf = Style { size: Size { width: length(10.0), height: length(40.0) }, ..Default::default() };
        let leaf = taffy.new_leaf(leaf).unwrap();
        let columns = if display == Display::Grid { vec![auto(), auto()] } else { Vec::new() };
        let nested = Style { display, grid_template_columns: columns, ..Default::default() };
        let nested = taffy.new_with_children(nested, &[block, leaf]).unwrap();
        let beside = paragraph(&mut taffy, 1, 30.0, None);
        let root = taffy.new_with_children(last_baseline_container(Display::Flex), &[nested, beside]).unwrap();
        let expected = [(0.0, 40.0), (0.0, 50.0), (0.0, 50.0)];
        assert_eq!(heights(&mut taffy, root, &[nested, beside, root]), expected, "{display:?}");
    }
}

/// Lays out `root` and returns the `(x, width)` of each of `nodes`.
// VIPER-PATCH(layout-f64): layout numbers of the alias type.
fn widths(taffy: &mut TaffyTree<()>, root: NodeId, nodes: &[NodeId]) -> Vec<(LayoutFloat, LayoutFloat)> {
    taffy.disable_rounding();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    nodes.iter().map(|node| taffy.layout(*node).map(|layout| (layout.location.x, layout.size.width)).unwrap()).collect()
}

/// A leaf `width` px wide and 10px high with `margin_right`.
fn bar(taffy: &mut TaffyTree<()>, width: f32, margin_right: f32) -> NodeId {
    let margin = Rect { left: zero(), right: length(margin_right), top: zero(), bottom: zero() };
    let style = Style { size: Size { width: length(width), height: length(10.0) }, margin, ..Default::default() };
    taffy.new_leaf(style).unwrap()
}

/// `justify-self: last baseline` in a grid of two 100px columns: items with horizontal text have no baseline across
/// a column, so one is synthesized at the line-under edge of an axis-compatible writing mode, the border box's left
/// edge (CSS Box Alignment 3 §9.1), and the items whose areas end in one column share that group (§6.4, §9.2). The
/// group then takes its fallback alignment, `safe self-end` (§9.3): a 30px item with 10px of margin on the right and
/// a 50px item in the first column put their left edges 50px into it, the widest outer part flush with its end.
/// Alone in the second column, a 30px item is end-aligned. Aligned at the end, the 30px item would be at 60.
#[test]
fn grid_items_justify_on_their_last_baselines_by_column() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let narrow = bar(&mut taffy, 30.0, 10.0);
    let wide = bar(&mut taffy, 50.0, 0.0);
    let alone = bar(&mut taffy, 30.0, 0.0);
    let place = |row: i16, column: i16| Style {
        grid_row: Line { start: line(row), end: auto() },
        grid_column: Line { start: line(column), end: auto() },
        ..Default::default()
    };
    for (node, (row, column)) in [(narrow, (1, 1)), (wide, (2, 1)), (alone, (1, 2))] {
        let size = taffy.style(node).unwrap().size;
        let margin = taffy.style(node).unwrap().margin;
        taffy.set_style(node, Style { size, margin, ..place(row, column) }).unwrap();
    }
    let grid = Style {
        display: Display::Grid,
        justify_items: AlignItems::LAST_BASELINE,
        grid_template_columns: vec![length(100.0), length(100.0)],
        ..Default::default()
    };
    let root = taffy.new_with_children(grid, &[narrow, wide, alone]).unwrap();
    assert_eq!(widths(&mut taffy, root, &[narrow, wide, alone]), [(50.0, 30.0), (50.0, 50.0), (170.0, 30.0)]);
}

/// `align-items: last baseline` in a flex column 100px wide: as in a grid column, the items' baselines are
/// synthesized at their border boxes' left edges (CSS Box Alignment 3 §9.1), so a 30px and a 50px item both start
/// 50px in. Aligned at the end, the 30px item would be at 70.
#[test]
fn flex_column_items_align_on_synthesized_last_baselines() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let narrow = bar(&mut taffy, 30.0, 0.0);
    let wide = bar(&mut taffy, 50.0, 0.0);
    let column = Style {
        flex_direction: FlexDirection::Column,
        size: Size { width: length(100.0), height: auto() },
        ..last_baseline_container(Display::Flex)
    };
    let root = taffy.new_with_children(column, &[narrow, wide]).unwrap();
    assert_eq!(widths(&mut taffy, root, &[narrow, wide]), [(50.0, 30.0), (50.0, 50.0)]);
}

/// A group taller than its line takes the `safe` half of its fallback alignment, `safe self-end` (CSS Box Alignment
/// 3 §4.2, §4.4): a three-line item, 60px high, in a 20px flex line or grid row is start-aligned, where end alignment
/// would put it 40px above the line.
#[test]
fn an_overflowing_last_baseline_group_is_start_aligned() {
    for display in [Display::Flex, Display::Grid] {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let three = paragraph(&mut taffy, 3, 0.0, None);
        let rows = if display == Display::Grid { vec![length(20.0)] } else { Vec::new() };
        let container = Style {
            size: Size { width: auto(), height: length(20.0) },
            grid_template_rows: rows,
            ..last_baseline_container(display)
        };
        let root = taffy.new_with_children(container, &[three]).unwrap();
        assert_eq!(heights(&mut taffy, root, &[three, root]), [(0.0, 60.0), (0.0, 20.0)], "{display:?}");
    }
}

/// `wrap-reverse` swaps a flex line's cross-start and cross-end, but a last-baseline group takes its items' fallback
/// alignment resolved to a physical side, `self-end`: the bottom (CSS Box Alignment 3 §9.3). In a 100px line the
/// shared last baseline is the group's largest descent, 14px, above the bottom: the three-line item sits 86 - 56 =
/// 30px down and the one-line item 86 - 16 = 70px down. At the top, they would sit at 0 and 40.
#[test]
fn a_wrap_reverse_rows_last_baseline_group_stays_at_the_bottom() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let one = paragraph(&mut taffy, 1, 10.0, None);
    let three = paragraph(&mut taffy, 3, 0.0, None);
    let container = Style {
        flex_wrap: FlexWrap::WrapReverse,
        size: Size { width: auto(), height: length(100.0) },
        ..last_baseline_container(Display::Flex)
    };
    let root = taffy.new_with_children(container, &[one, three]).unwrap();
    assert_eq!(heights(&mut taffy, root, &[one, three]), [(70.0, 30.0), (30.0, 60.0)]);
}

/// A container whose first line or row holds only a last-baseline group takes its first baseline from that group
/// (CSS Flexbox 1 and CSS Grid 2 Editor's Drafts, "Flex Container Baselines" and "Grid Container Baselines", step
/// 2): a three-line item and, after it, a one-line item with 10px of padding below share a last baseline 56px down,
/// so a one-line block beside the container in a `baseline` row sits 56 - 16 = 40px down. From the first item's
/// first baseline, 16px down, it would sit at the top.
#[test]
fn a_containers_first_baseline_can_come_from_its_last_baseline_group() {
    for display in [Display::Flex, Display::Grid] {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let three = paragraph(&mut taffy, 3, 0.0, None);
        let one = paragraph(&mut taffy, 1, 10.0, None);
        let nested = taffy.new_with_children(last_baseline_container(display), &[three, one]).unwrap();
        let beside = paragraph(&mut taffy, 1, 0.0, None);
        let row = Style { display: Display::Flex, align_items: AlignItems::BASELINE, ..Default::default() };
        let root = taffy.new_with_children(row, &[nested, beside]).unwrap();
        let expected = [(0.0, 70.0), (40.0, 20.0), (0.0, 70.0)];
        assert_eq!(heights(&mut taffy, root, &[nested, beside, root]), expected, "{display:?}");
    }
}

// VIPER-PATCH(size-containment): size and inline-size containment. The box is sized as if it had no content, then
// its content is laid out in that size (CSS Containment 2 §3.1, "sizing as if empty" and "laying out in-place");
// inline-size containment does it in the inline axis only (§3.2).

use taffy::{compute_leaf_layout, BaselineSource, Contain};

/// A block of `width` × `height` px.
fn block(taffy: &mut TaffyTree<()>, width: f32, height: f32) -> NodeId {
    taffy.new_leaf(Style { size: Size { width: length(width), height: length(height) }, ..Default::default() }).unwrap()
}

/// A block container with `contain`, 5px of padding on every side, and `children`.
fn contained(taffy: &mut TaffyTree<()>, display: Display, contain: Contain, children: &[NodeId]) -> NodeId {
    let padding = Rect { left: length(5.0), right: length(5.0), top: length(5.0), bottom: length(5.0) };
    let style = Style { display, contain, padding, ..Default::default() };
    taffy.new_with_children(style, children).unwrap()
}

/// Lays out `root` and returns the `(width, height)` of each of `nodes`.
// VIPER-PATCH(layout-f64): layout numbers of the alias type.
fn sizes(taffy: &mut TaffyTree<()>, root: NodeId, nodes: &[NodeId]) -> Vec<(LayoutFloat, LayoutFloat)> {
    taffy.disable_rounding();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    nodes
        .iter()
        .map(|node| taffy.layout(*node).map(|layout| (layout.size.width, layout.size.height)).unwrap())
        .collect()
}

/// A flex row whose items keep their own heights (`align-items: flex-start`).
fn row() -> Style {
    Style { display: Display::Flex, align_items: AlignItems::FLEX_START, ..Default::default() }
}

/// In a 300px-wide block, a block with `contain: size` and 5px of padding, holding a 100 × 50 block, takes the
/// full width (stretching does not depend on content) and the height of its padding alone: 10px, where it would be
/// 60px. Its child is still laid out, at (5, 5), overflowing it.
#[test]
fn a_size_contained_block_is_as_high_as_if_it_were_empty() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let child = block(&mut taffy, 100.0, 50.0);
    let boxed = contained(&mut taffy, Display::Block, Contain::SIZE, &[child]);
    let width =
        Style { display: Display::Block, size: Size { width: length(300.0), height: auto() }, ..Default::default() };
    let root = taffy.new_with_children(width, &[boxed]).unwrap();
    assert_eq!(sizes(&mut taffy, root, &[boxed, child]), [(300.0, 10.0), (100.0, 50.0)]);
    assert_eq!(taffy.layout(child).unwrap().location, Point { x: 5.0, y: 5.0 });
}

/// As a flex item sized by its content (`flex-basis: auto`, `width: auto`), the same box is 10 × 10: its content
/// contributes nothing to its intrinsic sizes in either axis, where it would make it 110 × 60. Its explicit
/// `min-height: 20px` is still honoured (all its own properties are, §3.1).
#[test]
fn a_size_contained_flex_item_has_no_content_size() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let child = block(&mut taffy, 100.0, 50.0);
    let boxed = contained(&mut taffy, Display::Block, Contain::SIZE, &[child]);
    let child = block(&mut taffy, 100.0, 50.0);
    let tall = contained(&mut taffy, Display::Flex, Contain::SIZE, &[child]);
    taffy
        .set_style(
            tall,
            Style { min_size: Size { width: auto(), height: length(20.0) }, ..taffy.style(tall).unwrap().clone() },
        )
        .unwrap();
    let root = taffy.new_with_children(row(), &[boxed, tall]).unwrap();
    assert_eq!(sizes(&mut taffy, root, &[boxed, tall]), [(10.0, 10.0), (10.0, 20.0)]);
}

/// With `contain: inline-size` only its width is as if empty, 10px; its height then comes from its content laid out
/// in that width: a wrapping flex row of two 30 × 20 items in a 0px-wide content box puts them on two lines, so
/// the box is 40 + 10 = 50px high. Without containment it would be 70 × 30.
#[test]
fn an_inline_size_contained_box_lays_its_content_out_in_its_empty_width() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let items = [block(&mut taffy, 30.0, 20.0), block(&mut taffy, 30.0, 20.0)];
    let wrap = Style { display: Display::Flex, flex_wrap: FlexWrap::Wrap, ..Default::default() };
    let lines = taffy.new_with_children(wrap, &items).unwrap();
    let boxed = contained(&mut taffy, Display::Block, Contain::INLINE_SIZE, &[lines]);
    let root = taffy.new_with_children(row(), &[boxed]).unwrap();
    assert_eq!(sizes(&mut taffy, root, &[boxed, lines]), [(10.0, 50.0), (0.0, 40.0)]);
}

/// The size fixed by sizing as if empty is definite: content is laid out "into the now fixed-size containment box"
/// (§3.1). In a 300px-wide block, a `contain: size` block with `min-height: 40px` is 40px high, and its child's
/// `height: 50%` resolves against that, 20px, though the child holds a 30px block, which an indefinite height would
/// leave it at.
#[test]
fn a_size_contained_boxs_fixed_height_is_a_percentage_basis() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let content = block(&mut taffy, 10.0, 30.0);
    let half =
        Style { display: Display::Block, size: Size { width: auto(), height: percent(0.5) }, ..Default::default() };
    let child = taffy.new_with_children(half, &[content]).unwrap();
    let style = Style {
        display: Display::Block,
        contain: Contain::SIZE,
        min_size: Size { width: auto(), height: length(40.0) },
        ..Default::default()
    };
    let boxed = taffy.new_with_children(style, &[child]).unwrap();
    let width =
        Style { display: Display::Block, size: Size { width: length(300.0), height: auto() }, ..Default::default() };
    let root = taffy.new_with_children(width, &[boxed]).unwrap();
    assert_eq!(sizes(&mut taffy, root, &[boxed, child]), [(300.0, 40.0), (300.0, 20.0)]);
}

/// A grid with `contain: size` is sized as if it had no items, and an empty grid still has its explicit tracks: one
/// 100px column and an `auto` row, which is 0px without items. So the grid is 110 × 10, where its 200 × 50 item
/// would make it 110 × 60. In an `auto` column of an outer grid the same contained grid makes the column 110px.
#[test]
fn a_size_contained_grid_keeps_its_explicit_tracks() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let item = block(&mut taffy, 200.0, 50.0);
    let grid = contained(&mut taffy, Display::Grid, Contain::SIZE, &[item]);
    let columns = Style { grid_template_columns: vec![length(100.0)], ..taffy.style(grid).unwrap().clone() };
    taffy.set_style(grid, columns).unwrap();
    let outer = Style { display: Display::Grid, grid_template_columns: vec![auto()], ..row() };
    let root = taffy.new_with_children(outer, &[grid]).unwrap();
    assert_eq!(sizes(&mut taffy, root, &[grid, root]), [(110.0, 10.0), (110.0, 10.0)]);
}

/// A leaf with `contain: size` is sized without its measure function: it is only ever asked to lay its content out
/// in a definite available width (its 0px content box), never for its intrinsic size. With
/// `contain: inline-size` it is measured once its width, its padding's 10px, is known, and the measure function
/// answers 100px for a 0px-wide content box (text wrapped at every word, say). The leaf alone would measure
/// 80 × 40.
#[test]
fn a_size_contained_leaf_is_measured_only_in_its_contained_size() {
    use std::cell::Cell;
    for (contain, expected) in [(Contain::SIZE, (10.0, 10.0)), (Contain::INLINE_SIZE, (10.0, 110.0))] {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        taffy.disable_rounding();
        let padding = Rect { left: length(5.0), right: length(5.0), top: length(5.0), bottom: length(5.0) };
        let leaf = taffy.new_leaf(Style { contain, padding, ..Default::default() }).unwrap();
        let root = taffy.new_with_children(row(), &[leaf]).unwrap();
        let intrinsic_measures = Cell::new(0);
        // Like Viper's, the measure function reads the content box's available width.
        // VIPER-PATCH(layout-f64): layout numbers of the alias type.
        let measure = |_: Size<Option<LayoutFloat>>, available: Size<AvailableSpace>| {
            let width = match available.width {
                AvailableSpace::Definite(width) => Some(width),
                _ => None,
            };
            match width {
                Some(width) => Size { width, height: if width == 0.0 { 100.0 } else { 40.0 } },
                None => {
                    intrinsic_measures.set(intrinsic_measures.get() + 1);
                    Size { width: 80.0, height: 40.0 }
                }
            }
        };
        taffy
            .compute_layout_with_measure(root, Size::MAX_CONTENT, |inputs, _, _, style| {
                compute_leaf_layout(inputs, style, |_, _| 0.0, measure)
            })
            .unwrap();
        let size = taffy.layout(leaf).unwrap().size;
        assert_eq!((size.width, size.height), expected, "{contain:?}");
        assert_eq!(intrinsic_measures.get(), 0, "{contain:?}");
    }
}

// VIPER-PATCH(scroll-last-baseline): a block container that is a block-axis scroll container with
// `baseline-source: auto` has its last baseline at its block-end margin edge (CSS Box Alignment 3 §9.1).

/// A 40px-high block container that clips (`overflow: hidden`, a scroll container) with a 5px bottom margin, holding
/// one line of text, and `baseline_source`; `table` marks it as a table.
fn clipping(taffy: &mut TaffyTree<()>, display: Display, baseline_source: BaselineSource, table: bool) -> NodeId {
    let line = text_line(taffy);
    let margin = Rect { left: zero(), right: zero(), top: zero(), bottom: length(5.0) };
    let style = Style {
        display,
        item_is_table: table,
        overflow: Point { x: Overflow::Hidden, y: Overflow::Hidden },
        size: Size { width: auto(), height: length(40.0) },
        margin,
        baseline_source,
        ..Default::default()
    };
    taffy.new_with_children(style, &[line]).unwrap()
}

/// Beside a one-line paragraph with 10px of padding below (last baseline 16px down, 14px above its end), the
/// clipping block's last baseline is its margin edge, 45px down: the group's baselines meet 45px down, so the
/// paragraph sits 45 - 16 = 29px down, in flex and in grid. With `baseline-source: first` or `last`, or as a flex
/// container or a table, which are no block containers, the clipping box's last baseline is its line's, 16px down
/// and 29px above its margin edge, and the paragraph stays at the top.
#[test]
fn a_clipping_blocks_last_baseline_is_its_bottom_margin_edge() {
    let cases = [
        (Display::Block, BaselineSource::Auto, false, 29.0),
        (Display::Block, BaselineSource::First, false, 0.0),
        (Display::Block, BaselineSource::Last, false, 0.0),
        (Display::Flex, BaselineSource::Auto, false, 0.0),
        (Display::Block, BaselineSource::Auto, true, 0.0),
    ];
    for container in [Display::Flex, Display::Grid] {
        for (display, source, table, expected) in cases {
            let mut taffy: TaffyTree<()> = TaffyTree::new();
            let clip = clipping(&mut taffy, display, source, table);
            let paragraph = paragraph(&mut taffy, 1, 10.0, None);
            let root = taffy.new_with_children(last_baseline_container(container), &[clip, paragraph]).unwrap();
            let y = heights(&mut taffy, root, &[clip, paragraph]);
            assert_eq!(y, [(0.0, 40.0), (expected, 30.0)], "{container:?} {display:?} {source:?} {table}");
        }
    }
}

/// A block container's last baseline is its last in-flow child's (CSS 2.1 §10.8.1), so one ending with the clipping
/// block has its last baseline 45px down too: beside it in a last-baseline row the paragraph sits 29px down.
#[test]
fn a_containers_last_baseline_can_be_its_clipping_childs_margin_edge() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let clip = clipping(&mut taffy, Display::Block, BaselineSource::Auto, false);
    let outer = taffy.new_with_children(Style { display: Display::Block, ..Default::default() }, &[clip]).unwrap();
    let paragraph = paragraph(&mut taffy, 1, 10.0, None);
    let root = taffy.new_with_children(last_baseline_container(Display::Flex), &[outer, paragraph]).unwrap();
    assert_eq!(heights(&mut taffy, root, &[outer, paragraph]), [(0.0, 45.0), (29.0, 30.0)]);
}

// VIPER-PATCH(layout-float-alias): every layout number has the type `taffy::LayoutFloat`.
// VIPER-PATCH(layout-f64): the alias is `f64`; style values stay `f32` and widen exactly where they are resolved.

// VIPER-PATCH(layout-f64): this comment names the alias's widths.
/// The alias is `f64`, layout results have its type, and style values stay `f32`.
#[test]
fn layout_numbers_are_the_alias() {
    use std::any::TypeId;
    // VIPER-PATCH(layout-f64): layout numbers of the alias type.
    assert_eq!(TypeId::of::<LayoutFloat>(), TypeId::of::<f64>());
    let layout = Layout::new();
    let _: (Point<LayoutFloat>, Size<LayoutFloat>, Rect<LayoutFloat>) = (layout.location, layout.size, layout.padding);
    let _: LayoutFloat = layout.scrollbar_size.width;
    let available: Option<LayoutFloat> = AvailableSpace::Definite(10.5).into_option();
    assert_eq!(available, Some(10.5));
    let style_value: f32 = CompactLength::length(10.5).value();
    assert_eq!(style_value, 10.5);
    // VIPER-PATCH(layout-f64): layout numbers of the alias type.
    assert_eq!(CompactLength::length(10.5).layout_value(), 10.5);
}

// VIPER-PATCH(layout-f64): this comment names the alias's widths.
/// Two blocks 2^24px and 1px high stack in exact sums: the second sits at 2^24px and the container is
/// 2^24 + 1px high, which `f32` cannot hold (its next value after 2^24 is 2^24 + 2).
#[test]
// VIPER-PATCH(layout-f64): layout numbers of the alias type.
fn layout_sums_are_exact_beyond_f32() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    taffy.disable_rounding();
    let block =
        |height: f32| Style { size: Size { width: length(10.0), height: length(height) }, ..Default::default() };
    let tall = taffy.new_leaf(block(16_777_216.0)).unwrap();
    let short = taffy.new_leaf(block(1.0)).unwrap();
    let root =
        taffy.new_with_children(Style { display: Display::Block, ..Default::default() }, &[tall, short]).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(taffy.layout(short).unwrap().location.y, 16_777_216.0);
    // VIPER-PATCH(layout-f64): the 1px above 2^24px is kept (an `f32` sum rounds it away, so the
    // difference would be 0).
    assert_eq!(taffy.layout(root).unwrap().size.height - 16_777_216.0, 1.0);
}

/// Three hundred blocks 100,000.015625px high (each exact in `f32`) stack exactly, to 30,000,004.6875px, a sum
/// that `f32` cannot hold.
#[test]
fn many_children_sum_exactly_in_layout_units() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    taffy.disable_rounding();
    let block =
        |height: f32| Style { size: Size { width: length(10.0), height: length(height) }, ..Default::default() };
    let children: Vec<NodeId> = (0..300).map(|_| taffy.new_leaf(block(100_000.016)).unwrap()).collect();
    let root = taffy.new_with_children(Style { display: Display::Block, ..Default::default() }, &children).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    // 100,000.016 is the `f32` 100,000.015625.
    let last = taffy.layout(children[299]).unwrap().location.y;
    assert_eq!(last, 299.0 * 100_000.015_625);
    assert_eq!(taffy.layout(root).unwrap().size.height, 300.0 * 100_000.015_625);
}
