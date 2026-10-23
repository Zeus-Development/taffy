//! Tests of the Viper fork's patches (see `PATCHES.md`), through `TaffyTree`. Each test sits under its patch's
//! marker comment and fails without that patch. Sizes are worked out by hand from Chrome's rules; a
//! layout unit is 1/64 px, Chrome's `LayoutUnit`. Taffy's rounding to whole pixels is disabled: Viper lays out without it.

use taffy::prelude::*;
use taffy::{Overflow, Point};

/// A content-box leaf with a 1px border on every side.
fn bordered(style: Style) -> Style {
    let border = Rect { left: length(1.0), right: length(1.0), top: length(1.0), bottom: length(1.0) };
    Style { box_sizing: BoxSizing::ContentBox, border, ..style }
}

// VIPER-PATCH(grid-fr-carry): fr tracks are sized in layout units as Blink's `ExpandFlexibleTracks` sizes them,
// each truncated with its remainder carried to the next flexible track.

/// The used widths, in layout units, of `count` 1fr columns of a grid `width` px wide with one item per column.
fn fr_columns_in_units(width: f32, count: usize) -> Vec<f32> {
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
    assert_eq!(columns.iter().sum::<f32>(), 10_878.0);
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
fn flex_boxes(container: Style, children: &[Style]) -> Vec<(f32, f32, f32, f32)> {
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
fn tall_content_item(align: AlignItems, item: Style) -> (f32, f32) {
    let mut taffy: TaffyTree<f32> = TaffyTree::new();
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
fn heights(taffy: &mut TaffyTree<()>, root: NodeId, nodes: &[NodeId]) -> Vec<(f32, f32)> {
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
fn widths(taffy: &mut TaffyTree<()>, root: NodeId, nodes: &[NodeId]) -> Vec<(f32, f32)> {
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
