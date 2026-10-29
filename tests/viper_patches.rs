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

/// A size through an aspect ratio is a single-precision product, widened: 70px high at 50 / 35 is 100px wide,
/// as in `f32` (the `f64` product of the `f32` ratio is 100.0000024px).
#[test]
fn a_size_through_an_aspect_ratio_is_a_single_precision_product() {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    taffy.disable_rounding();
    let style = |position: Position| Style {
        position,
        size: Size { width: auto(), height: length(70.0) },
        aspect_ratio: Some(50.0 / 35.0),
        ..Default::default()
    };
    let flowed = taffy.new_leaf(style(Position::Relative)).unwrap();
    let absolute = taffy.new_leaf(style(Position::Absolute)).unwrap();
    let root =
        taffy.new_with_children(Style { display: Display::Block, ..Default::default() }, &[flowed, absolute]).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(taffy.layout(flowed).unwrap().size.width, 100.0);
    assert_eq!(taffy.layout(absolute).unwrap().size.width, 100.0);
}

// VIPER-PATCH(subgrid-placement): a subgridded axis uses the parent's tracks and line names, its items are placed in
// them and clamped into them, and they are laid out in the parent's final track sizes (CSS Grid 2 section 9).
mod subgrid {
    use taffy::prelude::*;
    use taffy::{
        compute_cached_layout, compute_grid_layout, compute_leaf_layout, compute_root_layout, Cache, CacheTree,
        Direction, GridAutoFlow, GridPlacement, GridTemplateArea, GridTemplateAreas, Layout, LayoutFloat, LayoutInput,
        LayoutOutput, Point, SubgridTracks,
    };

    /// A box of the test tree: a grid when its style says so, otherwise a leaf 10px high when its height is not
    /// known
    struct Node {
        style: Style<String>,
        children: Vec<usize>,
        cache: Cache,
        layout: Layout,
        tracks: Option<SubgridTracks<String>>,
    }

    /// A tree that gives subgrids their parent's tracks, which `TaffyTree` does not yet
    #[derive(Default)]
    struct Tree(Vec<Node>);

    impl Tree {
        fn add(&mut self, style: Style<String>, children: Vec<usize>) -> usize {
            let layout = Layout::with_order(0);
            self.0.push(Node { style, children, cache: Cache::new(), layout, tracks: None });
            self.0.len() - 1
        }

        fn leaf(&mut self, style: Style<String>) -> usize {
            self.add(style, vec![])
        }

        fn node(&self, id: NodeId) -> &Node {
            &self.0[usize::from(id)]
        }

        fn node_mut(&mut self, id: NodeId) -> &mut Node {
            &mut self.0[usize::from(id)]
        }

        /// `node`'s location and size, relative to its parent's border box
        fn rect(&self, node: usize) -> (LayoutFloat, LayoutFloat, LayoutFloat, LayoutFloat) {
            let layout = &self.0[node].layout;
            (layout.location.x, layout.location.y, layout.size.width, layout.size.height)
        }

        fn lay_out(&mut self, root: usize) {
            compute_root_layout(self, NodeId::from(root), Size::MAX_CONTENT);
        }
    }

    struct ChildIter(std::vec::IntoIter<usize>);
    impl Iterator for ChildIter {
        type Item = NodeId;
        fn next(&mut self) -> Option<NodeId> {
            self.0.next().map(NodeId::from)
        }
    }

    impl TraversePartialTree for Tree {
        type ChildIter<'a> = ChildIter;
        fn child_ids(&self, id: NodeId) -> ChildIter {
            ChildIter(self.node(id).children.clone().into_iter())
        }
        fn child_count(&self, id: NodeId) -> usize {
            self.node(id).children.len()
        }
        fn get_child_id(&self, id: NodeId, index: usize) -> NodeId {
            NodeId::from(self.node(id).children[index])
        }
    }

    impl LayoutPartialTree for Tree {
        type CoreContainerStyle<'a> = &'a Style<String>;
        type CustomIdent = String;
        fn get_core_container_style(&self, id: NodeId) -> &Style<String> {
            &self.node(id).style
        }
        fn set_unrounded_layout(&mut self, id: NodeId, layout: &Layout) {
            self.node_mut(id).layout = *layout;
        }
        fn resolve_calc_value(&self, _val: *const (), _basis: LayoutFloat) -> LayoutFloat {
            0.0
        }
        fn compute_child_layout(&mut self, id: NodeId, inputs: LayoutInput) -> LayoutOutput {
            compute_cached_layout(self, id, inputs, |tree, id, inputs| {
                if tree.node(id).style.display == Display::Grid {
                    return compute_grid_layout(tree, id, inputs);
                }
                compute_leaf_layout(
                    inputs,
                    &tree.node(id).style,
                    |_, _| 0.0,
                    |known, _| Size { width: known.width.unwrap_or(0.0), height: known.height.unwrap_or(10.0) },
                )
            })
        }
    }

    impl CacheTree for Tree {
        fn cache_get(&mut self, id: NodeId, inputs: &LayoutInput) -> Option<LayoutOutput> {
            self.node_mut(id).cache.get(inputs)
        }
        fn cache_store(&mut self, id: NodeId, inputs: &LayoutInput, output: LayoutOutput) {
            self.node_mut(id).cache.store(inputs, output)
        }
        fn cache_clear(&mut self, id: NodeId) {
            self.node_mut(id).cache.clear();
        }
    }

    impl taffy::LayoutContainingBlock for Tree {
        type OofItemStyle<'a> = &'a Style<String>;
        fn get_oof_item_style(&self, id: NodeId) -> &Style<String> {
            &self.node(id).style
        }
        fn clear_hoisted_children(&mut self, _id: NodeId) {}
        fn add_hoisted_children(&mut self, _id: NodeId, _hoisted: &[NodeId]) {}
    }

    impl taffy::LayoutGridContainer for Tree {
        type GridContainerStyle<'a> = &'a Style<String>;
        type GridItemStyle<'a> = &'a Style<String>;
        fn get_grid_container_style(&self, id: NodeId) -> &Style<String> {
            &self.node(id).style
        }
        fn get_grid_child_style(&self, id: NodeId) -> &Style<String> {
            &self.node(id).style
        }
        fn set_subgrid_tracks(&mut self, id: NodeId, tracks: Option<SubgridTracks<String>>) {
            let node = self.node_mut(id);
            if node.tracks != tracks {
                node.tracks = tracks;
                node.cache.clear();
            }
        }
        fn subgrid_tracks(&self, id: NodeId) -> Option<SubgridTracks<String>> {
            self.node(id).tracks.clone()
        }
    }

    fn columns(sizes: &[f32]) -> Vec<GridTemplateComponent<String>> {
        sizes.iter().map(|size| length(*size)).collect()
    }

    fn names(sets: &[&[&str]]) -> Vec<Vec<String>> {
        sets.iter().map(|set| set.iter().map(|name| name.to_string()).collect()).collect()
    }

    fn columns_at(start: i16, end: i16) -> Line<GridPlacement<String>> {
        Line { start: line(start), end: line(end) }
    }

    /// A grid with columns 50px, 100px and 150px and a 10px column gap (lines at 0, 60, 170 and 320) and rows of
    /// 10px, with one child
    fn parent(tree: &mut Tree, child: usize, style: Style<String>) -> usize {
        let parent = Style {
            display: Display::Grid,
            grid_template_columns: columns(&[50.0, 100.0, 150.0]),
            grid_auto_rows: vec![length(10.0)],
            gap: Size { width: length(10.0), height: zero() },
            ..style
        };
        tree.add(parent, vec![child])
    }

    /// A subgrid of its columns, with the 10px column gap of the parents here (`gap_normal` is false by default), so
    /// that its gutters are theirs
    fn subgrid_columns(style: Style<String>) -> Style<String> {
        Style {
            display: Display::Grid,
            grid_template_subgrid: Point { x: true, y: false },
            gap: Size { width: length(10.0), height: zero() },
            ..style
        }
    }

    /// Section 9: "the subgrid's own grid tracks are the parent's tracks it spans"; its items, auto-placed one per
    /// track, take the parent's widths and positions, gaps included.
    #[test]
    fn a_subgrids_columns_are_its_parents() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..3).map(|_| tree.leaf(Style::default())).collect();
        let subgrid = Style { grid_column: columns_at(1, 4), ..subgrid_columns(Style::default()) };
        let subgrid = tree.add(subgrid, items.clone());
        let root = parent(&mut tree, subgrid, Style::default());
        tree.lay_out(root);
        assert_eq!(tree.rect(subgrid), (0.0, 0.0, 320.0, 10.0));
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 50.0, 10.0));
        assert_eq!(tree.rect(items[1]), (60.0, 0.0, 100.0, 10.0));
        assert_eq!(tree.rect(items[2]), (170.0, 0.0, 150.0, 10.0));
    }

    /// Section 9: the subgrid's margin, border and padding do not move its lines; its edge items keep clear of them.
    /// Spanning lines 2 to 4 (60 to 320) with a 3px left margin, a 2px left border and 5px left padding, its first
    /// track is 100 - 10 = 90px from x 70, and its second is the parent's third, at 170.
    #[test]
    fn a_subgrids_edge_items_keep_clear_of_its_margin_border_and_padding() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
        let subgrid = Style {
            grid_column: columns_at(2, 4),
            margin: Rect { left: length(3.0), right: zero(), top: zero(), bottom: zero() },
            border: Rect { left: length(2.0), right: zero(), top: zero(), bottom: zero() },
            padding: Rect { left: length(5.0), right: zero(), top: zero(), bottom: zero() },
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = parent(&mut tree, subgrid, Style::default());
        tree.lay_out(root);
        assert_eq!(tree.rect(subgrid), (63.0, 0.0, 257.0, 10.0));
        assert_eq!(tree.rect(items[0]), (7.0, 0.0, 90.0, 10.0));
        assert_eq!(tree.rect(items[1]), (107.0, 0.0, 150.0, 10.0));
    }

    /// Section 9: the subgrid's lines carry the parent's names in its span (explicit names and the implicit
    /// `<area>-start` and `<area>-end`) and its own `<line-name-list>`.
    #[test]
    fn a_subgrids_items_are_placed_by_the_parents_and_its_own_line_names() {
        let mut tree = Tree::default();
        let named = |start: &str, end: &str| Line {
            start: GridPlacement::NamedLine(start.to_string(), 1),
            end: GridPlacement::NamedLine(end.to_string(), 1),
        };
        let by_parent = tree.leaf(Style { grid_column: named("c", "d"), ..Style::default() });
        let by_own = tree.leaf(Style { grid_column: named("x", "y"), ..Style::default() });
        let by_area = tree.leaf(Style { grid_column: named("side-start", "side-end"), ..Style::default() });
        let subgrid = Style {
            grid_column: columns_at(2, 4),
            grid_template_column_names: names(&[&["x"], &["y"], &[]]),
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, vec![by_parent, by_own, by_area]);
        let area =
            GridTemplateArea { name: "side".to_string(), row_start: 1, row_end: 2, column_start: 3, column_end: 4 };
        let root = parent(
            &mut tree,
            subgrid,
            Style {
                grid_template_column_names: names(&[&["a"], &["b"], &["c"], &["d"]]),
                grid_template_areas: Some(GridTemplateAreas { areas: vec![area], row_count: 1, column_count: 3 }),
                ..Style::default()
            },
        );
        tree.lay_out(root);
        assert_eq!(tree.rect(by_parent).0, 110.0);
        assert_eq!(tree.rect(by_parent).2, 150.0);
        assert_eq!(tree.rect(by_own).0, 0.0);
        assert_eq!(tree.rect(by_own).2, 100.0);
        assert_eq!(tree.rect(by_area).0, 110.0);
    }

    /// Section 9: a subgridded axis has no implicit tracks; an item placed beyond the span is clamped into it as in
    /// an overlarge grid: one ending beyond it ends at its last line, one entirely beyond it takes its last track.
    #[test]
    fn an_item_beyond_the_subgrids_span_is_clamped_into_it() {
        let mut tree = Tree::default();
        let beyond = tree.leaf(Style { grid_column: columns_at(5, 7), ..Style::default() });
        let across = tree.leaf(Style { grid_column: columns_at(1, 9), ..Style::default() });
        let subgrid = Style { grid_column: columns_at(1, 3), ..subgrid_columns(Style::default()) };
        let subgrid = tree.add(subgrid, vec![beyond, across]);
        let root = parent(&mut tree, subgrid, Style::default());
        tree.lay_out(root);
        assert_eq!(tree.rect(subgrid).2, 160.0);
        assert_eq!((tree.rect(beyond).0, tree.rect(beyond).2), (60.0, 100.0));
        assert_eq!((tree.rect(across).0, tree.rect(across).2), (0.0, 160.0));
    }

    /// Section 9: subgridded rows work as columns do: rows 20px and 30px with a 5px row gap.
    #[test]
    fn a_subgrids_rows_are_its_parents() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
        let subgrid = Style {
            grid_row: columns_at(1, 3),
            grid_auto_flow: GridAutoFlow::Column,
            display: Display::Grid,
            grid_template_subgrid: Point { x: false, y: true },
            gap: Size { width: zero(), height: length(5.0) },
            ..Style::default()
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = Style {
            display: Display::Grid,
            size: Size { width: length(40.0), height: auto() },
            grid_template_rows: columns(&[20.0, 30.0]),
            gap: Size { width: zero(), height: length(5.0) },
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(subgrid), (0.0, 0.0, 40.0, 55.0));
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 40.0, 20.0));
        assert_eq!(tree.rect(items[1]), (0.0, 25.0, 40.0, 30.0));
    }

    /// Section 9: a subgrid whose inline direction runs against its parent's takes the parent's columns in reverse:
    /// a right-to-left subgrid over 50px and 100px columns starts with the 100px one, on the right.
    #[test]
    fn a_right_to_left_subgrid_takes_its_parents_columns_in_reverse() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
        let subgrid = Style {
            direction: Direction::Rtl,
            grid_column: columns_at(1, 3),
            gap: zero(),
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = Style {
            display: Display::Grid,
            grid_template_columns: columns(&[50.0, 100.0]),
            grid_auto_rows: vec![length(10.0)],
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(items[0]), (50.0, 0.0, 100.0, 10.0));
        assert_eq!(tree.rect(items[1]), (0.0, 0.0, 50.0, 10.0));
    }

    /// Section 9: a subgrid nested in a subgrid takes its tracks from it, which has them from its own parent.
    #[test]
    fn a_nested_subgrid_takes_the_outer_grids_tracks() {
        let mut tree = Tree::default();
        let item = tree.leaf(Style { grid_column: columns_at(2, 3), ..Style::default() });
        let inner = Style { grid_column: columns_at(1, 3), ..subgrid_columns(Style::default()) };
        let inner = tree.add(inner, vec![item]);
        let outer = Style { grid_column: columns_at(2, 4), ..subgrid_columns(Style::default()) };
        let outer = tree.add(outer, vec![inner]);
        let root = parent(&mut tree, outer, Style::default());
        tree.lay_out(root);
        assert_eq!(tree.rect(outer).0, 60.0);
        assert_eq!(tree.rect(inner), (0.0, 0.0, 260.0, 10.0));
        assert_eq!(tree.rect(item), (110.0, 0.0, 150.0, 10.0));
    }

    /// Section 9: a grid that is not a grid item is not a subgrid: its subgridded axis has implicit tracks only.
    #[test]
    fn a_subgrid_outside_a_grid_has_implicit_tracks() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
        let root = Style { size: Size { width: length(80.0), height: auto() }, ..subgrid_columns(Style::default()) };
        let root = tree.add(root, items.clone());
        tree.lay_out(root);
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 80.0, 10.0));
        assert_eq!(tree.rect(items[1]), (0.0, 10.0, 80.0, 10.0));
    }

    /// A right-to-left grid with columns 50px and 100px and a 10px gap in 160px: the 50px column on the right
    /// (110 to 160), the 100px one on the left (0 to 100).
    fn right_to_left_parent(tree: &mut Tree, child: usize) -> usize {
        let root = Style {
            display: Display::Grid,
            direction: Direction::Rtl,
            size: Size { width: length(160.0), height: auto() },
            grid_template_columns: columns(&[50.0, 100.0]),
            grid_auto_rows: vec![length(10.0)],
            gap: Size { width: length(10.0), height: zero() },
            ..Style::default()
        };
        tree.add(root, vec![child])
    }

    /// The span's sizes from a right-to-left parent's used positions, gap included: a left-to-right subgrid takes
    /// the columns in reverse, its first track the parent's 100px one on the left.
    #[test]
    fn a_right_to_left_parent_gives_a_left_to_right_subgrid_its_columns_in_reverse() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
        let subgrid =
            Style { direction: Direction::Ltr, grid_column: columns_at(1, 3), ..subgrid_columns(Style::default()) };
        let subgrid = tree.add(subgrid, items.clone());
        let root = right_to_left_parent(&mut tree, subgrid);
        tree.lay_out(root);
        assert_eq!(tree.rect(subgrid), (0.0, 0.0, 160.0, 10.0));
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 100.0, 10.0));
        assert_eq!(tree.rect(items[1]), (110.0, 0.0, 50.0, 10.0));
    }

    /// A right-to-left subgrid of a right-to-left parent keeps the parent's order: its first track is the 50px one
    /// on the right.
    #[test]
    fn a_right_to_left_parent_gives_a_right_to_left_subgrid_its_columns_in_order() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
        let subgrid =
            Style { direction: Direction::Rtl, grid_column: columns_at(1, 3), ..subgrid_columns(Style::default()) };
        let subgrid = tree.add(subgrid, items.clone());
        let root = right_to_left_parent(&mut tree, subgrid);
        tree.lay_out(root);
        assert_eq!(tree.rect(items[0]), (110.0, 0.0, 50.0, 10.0));
        assert_eq!(tree.rect(items[1]), (0.0, 0.0, 100.0, 10.0));
    }

    /// A right-to-left subgrid's inline start is its right edge: over the parent's 100px and 150px columns (60 to
    /// 320) with a 3px right margin, a 2px right border and 5px right padding, its first track is the 150px column
    /// less 10px, ending at its content box's right edge (250), and its second the 100px column at 0.
    #[test]
    fn a_right_to_left_subgrids_start_edge_is_its_right_edge() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
        let subgrid = Style {
            direction: Direction::Rtl,
            grid_column: columns_at(2, 4),
            margin: Rect { left: zero(), right: length(3.0), top: zero(), bottom: zero() },
            border: Rect { left: zero(), right: length(2.0), top: zero(), bottom: zero() },
            padding: Rect { left: zero(), right: length(5.0), top: zero(), bottom: zero() },
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = parent(&mut tree, subgrid, Style::default());
        tree.lay_out(root);
        assert_eq!(tree.rect(subgrid), (60.0, 0.0, 257.0, 10.0));
        assert_eq!(tree.rect(items[0]), (110.0, 0.0, 140.0, 10.0));
        assert_eq!(tree.rect(items[1]), (0.0, 0.0, 100.0, 10.0));
    }

    /// Section 9: a subgrid is stretched in its subgridded axis whatever its aspect ratio: 10px high with
    /// `aspect-ratio: 1`, it is as wide as its three columns (320px), not 10px.
    #[test]
    fn a_subgrid_is_stretched_in_its_subgridded_axis_whatever_its_aspect_ratio() {
        let mut tree = Tree::default();
        let items: Vec<usize> = (0..3).map(|_| tree.leaf(Style::default())).collect();
        let subgrid = Style {
            grid_column: columns_at(1, 4),
            size: Size { width: auto(), height: length(10.0) },
            aspect_ratio: Some(1.0),
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = parent(&mut tree, subgrid, Style::default());
        tree.lay_out(root);
        assert_eq!(tree.rect(subgrid), (0.0, 0.0, 320.0, 10.0));
    }

    /// The span's gutters include the parent's content distribution and are exact: four 50px columns spaced
    /// between in 300px have gutters of 100/3px, which no `f32` holds; the subgrid's items stand exactly where
    /// plain items in the same columns do.
    #[test]
    fn a_subgrids_gutters_take_the_parents_distributed_space_exactly() {
        let mut tree = Tree::default();
        let row = |row: i16| Line { start: line(row), end: line(row + 1) };
        let in_subgrid: Vec<usize> = (0..4).map(|_| tree.leaf(Style::default())).collect();
        let subgrid =
            Style { grid_column: columns_at(1, 5), grid_row: row(1), gap: zero(), ..subgrid_columns(Style::default()) };
        let subgrid = tree.add(subgrid, in_subgrid.clone());
        let plain: Vec<usize> = (0..4i16)
            .map(|at| {
                tree.leaf(Style { grid_column: columns_at(at + 1, at + 2), grid_row: row(2), ..Style::default() })
            })
            .collect();
        let root = Style {
            display: Display::Grid,
            size: Size { width: length(300.0), height: auto() },
            grid_template_columns: columns(&[50.0, 50.0, 50.0, 50.0]),
            grid_auto_rows: vec![length(10.0)],
            justify_content: taffy::JustifyContent::SPACE_BETWEEN,
            ..Style::default()
        };
        let root = tree.add(root, [vec![subgrid], plain.clone()].concat());
        tree.lay_out(root);
        assert_eq!(tree.rect(plain[1]).0, 50.0 + 100.0 / 3.0);
        for (item, plain) in in_subgrid.iter().zip(&plain) {
            assert_eq!((tree.rect(*item).0, tree.rect(*item).2), (tree.rect(*plain).0, tree.rect(*plain).2));
        }
    }

    /// The overlarge-grid clamp on the start side, and an auto-placed item, each alone in a subgrid over two columns
    /// (50px and 100px): an item at lines -9 to -8 lies entirely before the span and takes its first track, and an
    /// auto-placed item spanning three columns runs into a hypothetical implicit one and is cut at the span's last
    /// line.
    #[test]
    fn items_before_or_overflowing_the_subgrids_span_are_clamped_into_it() {
        let alone = |grid_column: Line<GridPlacement<String>>| {
            let mut tree = Tree::default();
            let item = tree.leaf(Style { grid_column, ..Style::default() });
            let subgrid = Style { grid_column: columns_at(1, 3), ..subgrid_columns(Style::default()) };
            let subgrid = tree.add(subgrid, vec![item]);
            let root = parent(&mut tree, subgrid, Style::default());
            tree.lay_out(root);
            (tree.rect(item).0, tree.rect(item).2)
        };
        assert_eq!(alone(columns_at(-9, -8)), (0.0, 50.0));
        assert_eq!(alone(Line { start: span(3), end: auto() }), (0.0, 160.0));
    }

    /// Two auto columns 10px apart, with one child
    fn auto_parent(tree: &mut Tree, child: usize) -> usize {
        let root = Style {
            display: Display::Grid,
            grid_template_columns: vec![auto(), auto()],
            grid_auto_rows: vec![length(10.0)],
            gap: Size { width: length(10.0), height: zero() },
            ..Style::default()
        };
        tree.add(root, vec![child])
    }

    /// A leaf `width` wide
    fn wide(tree: &mut Tree, width: f32) -> usize {
        tree.leaf(Style { size: Size { width: length(width), height: auto() }, ..Style::default() })
    }

    /// Section 9: a subgrid is empty for sizing in its subgridded axes, and its items size the parent's tracks
    /// there in its place: items 40px and 70px wide make the parent's auto columns 40px and 70px.
    #[test]
    fn a_subgrids_items_size_its_parents_tracks() {
        let mut tree = Tree::default();
        let items = vec![wide(&mut tree, 40.0), wide(&mut tree, 70.0)];
        let subgrid = Style { grid_column: columns_at(1, 3), ..subgrid_columns(Style::default()) };
        let subgrid = tree.add(subgrid, items.clone());
        let root = auto_parent(&mut tree, subgrid);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 120.0);
        assert_eq!(tree.rect(subgrid), (0.0, 0.0, 120.0, 10.0));
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 40.0, 10.0));
        assert_eq!(tree.rect(items[1]), (50.0, 0.0, 70.0, 10.0));
    }

    /// Section 9: the subgrid's margin, border and padding at an edge are extra margin on the items at that edge:
    /// a 5px left padding and a 3px right margin make the columns 45px and 73px, and the items keep their widths.
    #[test]
    fn a_subgrids_edges_are_extra_margin_on_its_edge_items() {
        let mut tree = Tree::default();
        let items = vec![wide(&mut tree, 40.0), wide(&mut tree, 70.0)];
        let subgrid = Style {
            grid_column: columns_at(1, 3),
            padding: Rect { left: length(5.0), right: zero(), top: zero(), bottom: zero() },
            margin: Rect { left: zero(), right: length(3.0), top: zero(), bottom: zero() },
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = auto_parent(&mut tree, subgrid);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 128.0);
        assert_eq!(tree.rect(subgrid), (0.0, 0.0, 125.0, 10.0));
        assert_eq!(tree.rect(items[0]), (5.0, 0.0, 40.0, 10.0));
        assert_eq!(tree.rect(items[1]), (55.0, 0.0, 70.0, 10.0));
    }

    /// Section 9's example: in a 300px grid with 50px gaps and columns `100px 1fr`, a subgrid over both columns
    /// with a gap of 0 has columns 125px and 175px, with 25px 112.5px and 162.5px, and with 75px 87.5px and
    /// 137.5px: each gutter keeps its centre and the tracks take half the difference.
    #[test]
    fn a_subgrids_own_gap_keeps_the_parents_gutters_centred() {
        let first_and_second = |gap: f32| {
            let mut tree = Tree::default();
            let items: Vec<usize> = (0..2).map(|_| tree.leaf(Style::default())).collect();
            let subgrid = Style {
                grid_column: columns_at(1, 3),
                gap: Size { width: length(gap), height: zero() },
                ..subgrid_columns(Style::default())
            };
            let subgrid = tree.add(subgrid, items.clone());
            let root = Style {
                display: Display::Grid,
                size: Size { width: length(300.0), height: auto() },
                grid_template_columns: vec![length(100.0), fr(1.0)],
                grid_auto_rows: vec![length(10.0)],
                gap: Size { width: length(50.0), height: zero() },
                ..Style::default()
            };
            let root = tree.add(root, vec![subgrid]);
            tree.lay_out(root);
            ((tree.rect(items[0]).0, tree.rect(items[0]).2), (tree.rect(items[1]).0, tree.rect(items[1]).2))
        };
        assert_eq!(first_and_second(0.0), ((0.0, 125.0), (125.0, 175.0)));
        assert_eq!(first_and_second(25.0), ((0.0, 112.5), (137.5, 162.5)));
        assert_eq!(first_and_second(75.0), ((0.0, 87.5), (162.5, 137.5)));
    }

    /// Section 9: half the difference of the subgrid's gap from the parent's is extra margin on the items it
    /// separates: with a 30px gap over the parent's 10px, items 40px and 70px wide make the columns 50px and 80px.
    #[test]
    fn half_a_subgrids_gap_difference_is_extra_margin_on_its_items() {
        let mut tree = Tree::default();
        let items = vec![wide(&mut tree, 40.0), wide(&mut tree, 70.0)];
        let subgrid = Style {
            grid_column: columns_at(1, 3),
            gap: Size { width: length(30.0), height: zero() },
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = auto_parent(&mut tree, subgrid);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 140.0);
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 40.0, 10.0));
        assert_eq!(tree.rect(items[1]), (70.0, 0.0, 70.0, 10.0));
    }

    /// Section 9: a subgrid's `normal` gap is its parent's, whatever `gap` holds: the gutters stay the parent's 10px
    /// and its items, 40px and 70px wide, get no extra margin.
    #[test]
    fn a_subgrids_normal_gap_is_its_parents() {
        let mut tree = Tree::default();
        let items = vec![wide(&mut tree, 40.0), wide(&mut tree, 70.0)];
        let subgrid = Style {
            grid_column: columns_at(1, 3),
            gap: Size { width: length(30.0), height: zero() },
            gap_normal: Size { width: true, height: false },
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = auto_parent(&mut tree, subgrid);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 120.0);
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 40.0, 10.0));
        assert_eq!(tree.rect(items[1]), (50.0, 0.0, 70.0, 10.0));
    }

    /// Section 9: the extra margins add up through nested subgrids: an item 60px wide in a subgrid with a 4px left
    /// padding, nested in the second column of a subgrid with a 2px right padding, makes that column 66px.
    #[test]
    fn a_nested_subgrids_items_size_the_outer_grids_tracks() {
        let mut tree = Tree::default();
        let item = wide(&mut tree, 60.0);
        let inner = Style {
            grid_column: columns_at(2, 3),
            padding: Rect { left: length(4.0), right: zero(), top: zero(), bottom: zero() },
            ..subgrid_columns(Style::default())
        };
        let inner = tree.add(inner, vec![item]);
        let first = wide(&mut tree, 40.0);
        let outer = Style {
            grid_column: columns_at(1, 3),
            padding: Rect { left: zero(), right: length(2.0), top: zero(), bottom: zero() },
            ..subgrid_columns(Style::default())
        };
        let outer = tree.add(outer, vec![first, inner]);
        let root = auto_parent(&mut tree, outer);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 116.0);
        assert_eq!(tree.rect(first), (0.0, 0.0, 40.0, 10.0));
        assert_eq!(tree.rect(inner), (50.0, 0.0, 64.0, 10.0));
        assert_eq!(tree.rect(item), (4.0, 0.0, 60.0, 10.0));
    }

    /// Section 9: a subgrid's items size the parent's rows as they do its columns: items 20px and 30px high make
    /// the parent's auto rows 20px and 30px.
    #[test]
    fn a_subgrids_items_size_its_parents_rows() {
        let mut tree = Tree::default();
        let high = |tree: &mut Tree, height: f32| {
            tree.leaf(Style { size: Size { width: auto(), height: length(height) }, ..Style::default() })
        };
        let items = vec![high(&mut tree, 20.0), high(&mut tree, 30.0)];
        let subgrid = Style {
            grid_row: columns_at(1, 3),
            grid_auto_flow: GridAutoFlow::Column,
            display: Display::Grid,
            grid_template_subgrid: Point { x: false, y: true },
            gap: Size { width: zero(), height: length(5.0) },
            ..Style::default()
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = Style {
            display: Display::Grid,
            size: Size { width: length(40.0), height: auto() },
            grid_template_rows: vec![auto(), auto()],
            gap: Size { width: zero(), height: length(5.0) },
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).3, 55.0);
        assert_eq!(tree.rect(items[0]), (0.0, 0.0, 40.0, 20.0));
        assert_eq!(tree.rect(items[1]), (0.0, 25.0, 40.0, 30.0));
    }

    /// Section 9: a subgrid's items size the parent's tracks only in its subgridded axes, so the item of a subgrid
    /// of its columns is in the subgrid's own row, not in the parent's row's baseline-sharing group: the parent's
    /// baseline-aligned item 20px high stays at the top beside a baseline-aligned subgrid item 50px high.
    #[test]
    fn a_column_subgrids_item_shares_no_baseline_with_the_parents_row() {
        let mut tree = Tree::default();
        let high = |tree: &mut Tree, height: f32, column: Line<GridPlacement<String>>| {
            let size = Size { width: auto(), height: length(height) };
            tree.leaf(Style { grid_column: column, align_self: Some(AlignSelf::BASELINE), size, ..Style::default() })
        };
        let first = high(&mut tree, 20.0, columns_at(1, 2));
        let inner = high(&mut tree, 50.0, Line { start: auto(), end: auto() });
        let subgrid = Style { grid_column: columns_at(2, 3), gap: zero(), ..subgrid_columns(Style::default()) };
        let subgrid = tree.add(subgrid, vec![inner]);
        let root = Style {
            display: Display::Grid,
            grid_template_columns: columns(&[100.0, 100.0]),
            grid_template_rows: vec![auto()],
            ..Style::default()
        };
        let root = tree.add(root, vec![first, subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(first), (0.0, 0.0, 100.0, 20.0));
        assert_eq!(tree.rect(inner), (0.0, 0.0, 100.0, 50.0));
    }

    /// Section 9 with a right-to-left subgrid: its items size the parent's columns in reverse, and its left padding
    /// (its inline-end edge) is extra margin on the item at its last line: items 40px and 70px wide make the
    /// columns 75px (70 + 5) and 40px.
    #[test]
    fn a_right_to_left_subgrids_items_size_the_parents_columns_in_reverse() {
        let mut tree = Tree::default();
        let items = vec![wide(&mut tree, 40.0), wide(&mut tree, 70.0)];
        let subgrid = Style {
            direction: Direction::Rtl,
            grid_column: columns_at(1, 3),
            padding: Rect { left: length(5.0), right: zero(), top: zero(), bottom: zero() },
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = auto_parent(&mut tree, subgrid);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 125.0);
        assert_eq!(tree.rect(items[0]), (85.0, 0.0, 40.0, 10.0));
        assert_eq!(tree.rect(items[1]), (5.0, 0.0, 70.0, 10.0));
    }

    /// Section 12.1: an item's height is measured against its own column, which in a subgrid of its rows is the
    /// subgrid's own: in a subgrid of its rows with own columns `100px 100px`, an `aspect-ratio: 1` item in its second
    /// column makes the parent's auto row 100px, not the 200px of the subgrid's whole width.
    #[test]
    fn a_row_subgrids_item_is_measured_in_its_own_column() {
        let mut tree = Tree::default();
        let square = tree.leaf(Style { grid_column: columns_at(2, 3), aspect_ratio: Some(1.0), ..Style::default() });
        let subgrid = Style {
            display: Display::Grid,
            grid_row: columns_at(1, 2),
            grid_template_subgrid: Point { x: false, y: true },
            grid_template_columns: columns(&[100.0, 100.0]),
            ..Style::default()
        };
        let subgrid = tree.add(subgrid, vec![square]);
        let root = Style {
            display: Display::Grid,
            grid_template_columns: columns(&[200.0]),
            grid_template_rows: vec![auto()],
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).3, 100.0);
        assert_eq!(tree.rect(square), (100.0, 0.0, 100.0, 100.0));
    }

    /// Section 12.1: a subgrid of its rows with one own column measures its items in that column, which is the area
    /// when it is flexible
    #[test]
    fn a_row_subgrids_item_is_measured_in_its_one_own_column() {
        for (column, size) in [(length(50.0), 50.0), (fr(1.0), 200.0)] {
            let mut tree = Tree::default();
            let square = tree.leaf(Style { aspect_ratio: Some(1.0), ..Style::default() });
            let subgrid = Style {
                display: Display::Grid,
                grid_template_subgrid: Point { x: false, y: true },
                grid_template_columns: vec![column],
                ..Style::default()
            };
            let subgrid = tree.add(subgrid, vec![square]);
            let root = Style {
                display: Display::Grid,
                grid_template_columns: columns(&[200.0]),
                grid_template_rows: vec![auto()],
                ..Style::default()
            };
            let root = tree.add(root, vec![subgrid]);
            tree.lay_out(root);
            assert_eq!(tree.rect(root).3, size);
            assert_eq!(tree.rect(square), (0.0, 0.0, size, size));
        }
    }

    /// Section 12.1: a subgrid of its rows narrower than its area, by its width or its maximum width, sizes its own
    /// column in its own box and measures its items there, not across the area; one with neither fills the area
    #[test]
    fn a_row_subgrids_item_is_measured_in_its_own_box() {
        let cases: [(Dimension, LengthPercentageAuto, LayoutFloat); 3] =
            [(length(50.0), auto(), 50.0), (auto(), length(50.0), 50.0), (auto(), auto(), 200.0)];
        for (width, max_width, size) in cases {
            let mut tree = Tree::default();
            let square = tree.leaf(Style { aspect_ratio: Some(1.0), ..Style::default() });
            let subgrid = Style {
                display: Display::Grid,
                grid_template_subgrid: Point { x: false, y: true },
                size: Size { width, height: auto() },
                max_size: Size { width: max_width, height: auto() },
                ..Style::default()
            };
            let subgrid = tree.add(subgrid, vec![square]);
            let root = Style {
                display: Display::Grid,
                grid_template_columns: columns(&[200.0]),
                grid_template_rows: vec![auto()],
                ..Style::default()
            };
            let root = tree.add(root, vec![subgrid]);
            tree.lay_out(root);
            assert_eq!(tree.rect(root).3, size);
            assert_eq!(tree.rect(square), (0.0, 0.0, size, size));
        }
    }

    /// Section 12.1: the items of a subgrid of its rows size the parent's rows each in its own column: over two auto
    /// rows, an `aspect-ratio: 1` item in the second of own columns `100px 100px` makes the first row 100px and a
    /// 30px-high item in the first column the second row 30px, rather than the subgrid's 130px spread over both.
    #[test]
    fn a_row_subgrids_items_size_the_parents_rows_in_their_own_columns() {
        let mut tree = Tree::default();
        let square = Style { grid_row: columns_at(1, 2), grid_column: columns_at(2, 3), ..Style::default() };
        let square = tree.leaf(Style { aspect_ratio: Some(1.0), ..square });
        let size = Size { width: auto(), height: length(30.0) };
        let low =
            tree.leaf(Style { grid_row: columns_at(2, 3), grid_column: columns_at(1, 2), size, ..Style::default() });
        let subgrid = Style {
            display: Display::Grid,
            grid_row: columns_at(1, 3),
            grid_template_subgrid: Point { x: false, y: true },
            grid_template_columns: columns(&[100.0, 100.0]),
            ..Style::default()
        };
        let subgrid = tree.add(subgrid, vec![square, low]);
        let root = Style {
            display: Display::Grid,
            grid_template_columns: columns(&[200.0]),
            grid_template_rows: vec![auto(), auto()],
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).3, 130.0);
        assert_eq!(tree.rect(square), (100.0, 0.0, 100.0, 100.0));
        assert_eq!(tree.rect(low), (0.0, 100.0, 100.0, 30.0));
    }

    /// Section 12.1, step 3: the items of a subgrid of its columns size the parent's columns again once each is
    /// measured in its own row: over two auto columns, `aspect-ratio: 1` items in own rows `50px 100px` make the
    /// columns 100px (the item in the second row) and 50px (the one in the first), rather than 75px each.
    #[test]
    fn a_column_subgrids_items_size_the_parents_columns_in_their_own_rows() {
        let mut tree = Tree::default();
        let square = |tree: &mut Tree, row: i16, column: i16| {
            let placed = Style {
                grid_row: columns_at(row, row + 1),
                grid_column: columns_at(column, column + 1),
                ..Style::default()
            };
            tree.leaf(Style { aspect_ratio: Some(1.0), ..placed })
        };
        let items = vec![square(&mut tree, 2, 1), square(&mut tree, 1, 2)];
        let subgrid = Style {
            display: Display::Grid,
            grid_column: columns_at(1, 3),
            grid_template_subgrid: Point { x: true, y: false },
            grid_template_rows: columns(&[50.0, 100.0]),
            ..Style::default()
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = Style {
            display: Display::Grid,
            grid_template_columns: vec![auto(), auto()],
            grid_template_rows: vec![auto()],
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 150.0);
        assert_eq!(tree.rect(items[0]), (0.0, 50.0, 100.0, 100.0));
        assert_eq!(tree.rect(items[1]), (100.0, 0.0, 50.0, 50.0));
    }

    /// A subgrid of its columns, with no gap, in a grid of one auto column and one row `row` px high
    fn column_subgrid_in(tree: &mut Tree, subgrid: Style<String>, items: Vec<usize>, row: f32) -> (usize, usize) {
        let subgrid = Style { grid_column: columns_at(1, 2), gap: zero(), ..subgrid_columns(subgrid) };
        let subgrid = tree.add(subgrid, items);
        let root = Style {
            display: Display::Grid,
            grid_template_columns: vec![auto()],
            grid_template_rows: columns(&[row]),
            ..Style::default()
        };
        (tree.add(root, vec![subgrid]), subgrid)
    }

    /// Section 12.1, step 1: a subgrid of its columns in a 100px row measures its items in its own rows, not in the
    /// row: an `aspect-ratio: 1` item in the second of own rows `50px 50px`, or `50% 50%`, makes the auto column 50px.
    #[test]
    fn a_column_subgrids_items_size_the_parents_columns_in_their_own_rows_of_a_fixed_row() {
        for rows in [columns(&[50.0, 50.0]), vec![percent(0.5), percent(0.5)]] {
            let mut tree = Tree::default();
            let square = tree.leaf(Style { aspect_ratio: Some(1.0), grid_row: columns_at(2, 3), ..Style::default() });
            let subgrid = Style { grid_template_rows: rows, ..Style::default() };
            let (root, _) = column_subgrid_in(&mut tree, subgrid, vec![square], 100.0);
            tree.lay_out(root);
            assert_eq!(tree.rect(root).2, 50.0);
            assert_eq!(tree.rect(square), (0.0, 50.0, 50.0, 50.0));
        }
    }

    /// Section 12.1, step 1: a subgrid of its columns aligned to the start of a 100px row is as high as its content,
    /// so an `aspect-ratio: 1` item with `min-height: 30px` is measured in a 30px own row and makes the column 30px.
    #[test]
    fn a_column_subgrid_aligned_to_the_start_of_its_row_measures_its_items_in_its_own_height() {
        let mut tree = Tree::default();
        let min_size = Size { width: auto(), height: length(30.0) };
        let square = tree.leaf(Style { aspect_ratio: Some(1.0), min_size, ..Style::default() });
        let subgrid = Style { align_self: Some(AlignSelf::START), ..Style::default() };
        let (root, subgrid) = column_subgrid_in(&mut tree, subgrid, vec![square], 100.0);
        tree.lay_out(root);
        assert_eq!(tree.rect(root), (0.0, 0.0, 30.0, 100.0));
        assert_eq!(tree.rect(subgrid), (0.0, 0.0, 30.0, 30.0));
        assert_eq!(tree.rect(square), (0.0, 0.0, 30.0, 30.0));
    }

    /// Section 12.1: the items of a right-to-left subgrid of its columns size the parent's columns in their own rows,
    /// the subgrid's first column being the parent's second: `aspect-ratio: 1` items in own rows `50px 100px` make
    /// the parent's first column 50px (the item in the subgrid's second column, first row) and its second 100px.
    #[test]
    fn a_right_to_left_column_subgrids_items_size_the_parents_columns_in_their_own_rows() {
        let mut tree = Tree::default();
        let square = |tree: &mut Tree, row: i16, column: i16| {
            let placed = Style {
                grid_row: columns_at(row, row + 1),
                grid_column: columns_at(column, column + 1),
                ..Style::default()
            };
            tree.leaf(Style { aspect_ratio: Some(1.0), ..placed })
        };
        let items = vec![square(&mut tree, 2, 1), square(&mut tree, 1, 2)];
        let subgrid = Style {
            direction: Direction::Rtl,
            grid_column: columns_at(1, 3),
            gap: zero(),
            grid_template_rows: columns(&[50.0, 100.0]),
            ..subgrid_columns(Style::default())
        };
        let subgrid = tree.add(subgrid, items.clone());
        let root = Style {
            display: Display::Grid,
            grid_template_columns: vec![auto(), auto()],
            grid_template_rows: vec![auto()],
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 150.0);
        assert_eq!(tree.rect(items[0]), (50.0, 50.0, 100.0, 100.0));
        assert_eq!(tree.rect(items[1]), (0.0, 0.0, 50.0, 50.0));
    }

    /// Section 9: a subgrid's size constraints do not apply in its subgridded axis, and its aspect ratio sizes nothing
    /// in the other, where it fills its 200px column: its `aspect-ratio: 1` item makes the auto row 200px.
    #[test]
    fn a_row_subgrids_aspect_ratio_does_not_narrow_its_items() {
        let mut tree = Tree::default();
        let square = tree.leaf(Style { aspect_ratio: Some(1.0), ..Style::default() });
        let subgrid = Style {
            display: Display::Grid,
            grid_template_subgrid: Point { x: false, y: true },
            aspect_ratio: Some(4.0),
            ..Style::default()
        };
        let subgrid = tree.add(subgrid, vec![square]);
        let root = Style {
            display: Display::Grid,
            grid_template_columns: columns(&[200.0]),
            grid_template_rows: vec![auto()],
            ..Style::default()
        };
        let root = tree.add(root, vec![subgrid]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).3, 200.0);
        assert_eq!(tree.rect(square), (0.0, 0.0, 200.0, 200.0));
    }

    /// A subgrid of its rows, inside a subgrid of both axes, in a grid of one 200px column and an auto row
    fn row_subgrid_in_both(tree: &mut Tree, inner: Style<String>, items: Vec<usize>) -> usize {
        let inner = Style { display: Display::Grid, grid_template_subgrid: Point { x: false, y: true }, ..inner };
        let inner = tree.add(inner, items);
        let outer =
            Style { display: Display::Grid, grid_template_subgrid: Point { x: true, y: true }, ..Style::default() };
        let outer = tree.add(outer, vec![inner]);
        let root = Style {
            display: Display::Grid,
            grid_template_columns: columns(&[200.0]),
            grid_template_rows: vec![auto()],
            ..Style::default()
        };
        tree.add(root, vec![outer])
    }

    /// Section 12.1: a subgrid of its rows inside a subgrid of both axes measures its items in its own column, `100px`
    /// of `100px 100px`, or in its own 50px width: its `aspect-ratio: 1` item makes the row 100px, or 50px.
    #[test]
    fn a_row_subgrid_inside_a_subgrid_of_both_axes_measures_its_items_in_its_own_columns() {
        let mut tree = Tree::default();
        let square = tree.leaf(Style { aspect_ratio: Some(1.0), grid_column: columns_at(2, 3), ..Style::default() });
        let inner = Style { grid_template_columns: columns(&[100.0, 100.0]), ..Style::default() };
        let root = row_subgrid_in_both(&mut tree, inner, vec![square]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).3, 100.0);
        assert_eq!(tree.rect(square), (100.0, 0.0, 100.0, 100.0));

        let mut tree = Tree::default();
        let square = tree.leaf(Style { aspect_ratio: Some(1.0), ..Style::default() });
        let inner = Style { size: Size { width: length(50.0), height: auto() }, ..Style::default() };
        let root = row_subgrid_in_both(&mut tree, inner, vec![square]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).3, 50.0);
        assert_eq!(tree.rect(square), (0.0, 0.0, 50.0, 50.0));
    }

    /// Section 12.1: a subgrid of its columns inside a subgrid of its rows measures its items in its own rows: an
    /// `aspect-ratio: 1` item in the second of own rows `50px 50px` makes the outer grid's auto column 50px.
    #[test]
    fn a_column_subgrid_inside_a_row_subgrid_measures_its_items_in_its_own_rows() {
        let mut tree = Tree::default();
        let square = tree.leaf(Style { aspect_ratio: Some(1.0), grid_row: columns_at(2, 3), ..Style::default() });
        let inner =
            Style { gap: zero(), grid_template_rows: columns(&[50.0, 50.0]), ..subgrid_columns(Style::default()) };
        let inner = tree.add(inner, vec![square]);
        let outer =
            Style { display: Display::Grid, grid_template_subgrid: Point { x: false, y: true }, ..Style::default() };
        let outer = tree.add(outer, vec![inner]);
        let root = Style {
            display: Display::Grid,
            grid_template_columns: vec![auto()],
            grid_template_rows: vec![auto()],
            ..Style::default()
        };
        let root = tree.add(root, vec![outer]);
        tree.lay_out(root);
        assert_eq!(tree.rect(root).2, 50.0);
        assert_eq!(tree.rect(outer).2, 50.0);
        assert_eq!(tree.rect(square), (0.0, 50.0, 50.0, 50.0));
    }
}
