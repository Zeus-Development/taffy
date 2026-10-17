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
