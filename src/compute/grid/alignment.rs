//! Alignment of tracks and final positioning of items
use super::types::GridTrack;
use crate::compute::common::alignment::{
    apply_alignment_fallback, compute_alignment_offset, resolve_self_alignment_safety,
};
use crate::geometry::{InBothAbsAxis, Line, Point, Rect, Size};
// VIPER-PATCH(subgrid-placement): `Dimension` and `LengthPercentageAuto` for a subgrid's ignored sizes.
use crate::style::{
    AlignContent, AlignItems, AlignItemsKeyword, AlignSelf, AvailableSpace, CoreStyle, Dimension, GridItemStyle,
    LengthPercentageAuto, Overflow, Position,
};
// VIPER-PATCH(last-baseline): `Baselines` for the item's baselines.
use crate::tree::{Baselines, Layout, LayoutPartialTreeExt, NodeId, OofCandidates, SizingMode};
use crate::util::sys::f32_max;
use crate::util::{MaybeMath, MaybeResolve, ResolveOrZero};

#[cfg(feature = "content_size")]
use crate::compute::common::scrollable_overflow::compute_scrollable_overflow_contribution;
// VIPER-PATCH(scroll-last-baseline): the shared baseline code.
use crate::compute::common::containment::last_baseline_at_margin_edge;
use crate::compute::common::sizing_keyword::{resolve_sizing_keyword, SizingKeywordResolution};
use crate::{AbsoluteAxis, BoxSizing, Direction, LayoutGridContainer};
// VIPER-PATCH(layout-float-alias): the alias.
use crate::LayoutFloat;

/// Align the grid tracks within the grid according to the align-content (rows) or
/// justify-content (columns) property. This only does anything if the size of the
/// grid is not equal to the size of the grid container in the axis being aligned.
pub(super) fn align_tracks(
    // VIPER-PATCH(layout-float-alias): the alias.
    grid_container_content_box_size: LayoutFloat,
    padding: Line<LayoutFloat>,
    border: Line<LayoutFloat>,
    tracks: &mut [GridTrack],
    track_alignment_style: AlignContent,
    axis_is_reversed: bool,
) {
    // VIPER-PATCH(layout-float-alias): the alias.
    let used_size: LayoutFloat = tracks.iter().map(|track| track.base_size).sum();
    let free_space = grid_container_content_box_size - used_size;
    let origin = padding.start + border.start;

    // Count the number of non-collapsed tracks (not counting gutters)
    let num_tracks = tracks.iter().skip(1).step_by(2).filter(|track| !track.is_collapsed).count();

    // Grid layout treats gaps as full tracks rather than applying them at alignment so we
    // simply pass zero here. Grid layout is never reversed.
    let gap = 0.0;
    let layout_is_reversed = false;
    let track_alignment = apply_alignment_fallback(free_space, num_tracks, track_alignment_style);
    let track_alignment = if axis_is_reversed { track_alignment.reversed() } else { track_alignment };

    // If every track is collapsed then no track receives the alignment offset below, but the
    // grid's lines should still be aligned within the container (e.g. at the inline-start edge
    // for RTL), so apply the offset to the origin instead.
    let empty_grid_offset = if num_tracks == 0 {
        compute_alignment_offset(free_space, num_tracks, gap, track_alignment, layout_is_reversed, true)
    } else {
        0.0
    };

    // Compute offsets. Tracks are stored in logical order; when the axis is reversed (RTL)
    // physical offsets are assigned right-to-left by iterating the tracks in reverse.
    let mut total_offset = origin + empty_grid_offset;
    let mut seen_non_collapsed_track = false;
    let mut position_track = |i: usize, track: &mut GridTrack| {
        // Odd tracks are gutters (but slices are zero-indexed, so odd tracks have even indices)
        let is_gutter = i % 2 == 0;
        let is_non_collapsed_track = !is_gutter && !track.is_collapsed;

        // Alignment offsets should be applied only to non-collapsed tracks.
        let is_first = is_non_collapsed_track && !seen_non_collapsed_track;

        let offset = if is_non_collapsed_track {
            compute_alignment_offset(free_space, num_tracks, gap, track_alignment, layout_is_reversed, is_first)
        } else {
            0.0
        };

        track.offset = total_offset + offset;
        total_offset = total_offset + offset + track.base_size;
        if is_non_collapsed_track {
            seen_non_collapsed_track = true;
        }
    };
    if axis_is_reversed {
        tracks.iter_mut().rev().enumerate().for_each(|(i, track)| position_track(i, track));
    } else {
        tracks.iter_mut().enumerate().for_each(|(i, track)| position_track(i, track));
    }
}

/// Align and size a grid item into it's final position
#[allow(clippy::too_many_arguments)]
pub(super) fn align_and_position_item(
    tree: &mut impl LayoutGridContainer,
    node: NodeId,
    order: u32,
    // VIPER-PATCH(subgrid-placement): the axes in which the item is a subgrid.
    subgridded: InBothAbsAxis<bool>,
    // VIPER-PATCH(layout-float-alias): the alias.
    grid_area: Rect<LayoutFloat>,
    // Percentage basis for the item's styles. Grid passes the grid area size; grid lanes (css-grid-3) items
    // have the container's content box as their containing block in the stacking axis.
    // VIPER-PATCH(layout-float-alias): the alias.
    containing_block_size: Size<LayoutFloat>,
    container_alignment_styles: InBothAbsAxis<AlignItems>,
    // VIPER-PATCH(last-baseline): the baseline shims of each axis: in the block axis the first-baseline shim at the top
    // and the last-baseline shim at the bottom, in the inline axis the shim of a `justify-self: last baseline` group.
    // VIPER-PATCH(layout-float-alias): the alias.
    baseline_shims: Size<Line<LayoutFloat>>,
    direction: Direction,
    // VIPER-PATCH(layout-float-alias): the alias.
    container_border_box_width: LayoutFloat,
    container_border: Rect<LayoutFloat>,
    #[cfg(feature = "content_size")] container_is_scroll_container: bool,
    bubbled_candidates: &mut OofCandidates,
    // VIPER-PATCH(last-baseline): and the item's location, size and baselines.
    // VIPER-PATCH(layout-float-alias): the alias.
) -> (Rect<LayoutFloat>, Point<LayoutFloat>, Size<LayoutFloat>, Baselines) {
    let grid_area_size = Size { width: grid_area.right - grid_area.left, height: grid_area.bottom - grid_area.top };

    let style = tree.get_grid_child_style(node);

    let overflow = style.overflow();
    // VIPER-PATCH(scroll-last-baseline): CSS Box Alignment 3 §9.1's legacy rule.
    let last_baseline_at_margin_edge = last_baseline_at_margin_edge(&style);
    #[cfg(feature = "content_size")]
    let contain = style.contain();
    // VIPER-PATCH(layout-f64): the scrollbar width, a style value, widened.
    let scrollbar_width = LayoutFloat::from(style.scrollbar_width());
    let aspect_ratio = style.aspect_ratio();
    let is_replaced = style.is_replaced();
    // Fall back to the container's justify-items/align-items if unset (`None`), and then resolve
    // writing-mode-relative self-start/self-end keywords against the item's own direction.
    // The horizontal axis is the inline axis (Taffy only supports horizontal-tb); the vertical
    // (block) axis resolves them to plain start/end.
    let item_direction = style.direction();
    let justify_self = style.justify_self().unwrap_or(container_alignment_styles.horizontal).resolve_self_relative(
        item_direction,
        direction,
        true,
    );
    let align_self = style.align_self().unwrap_or(container_alignment_styles.vertical).resolve_self_relative(
        item_direction,
        direction,
        false,
    );

    let position = style.position();
    let inset_horizontal = style
        .inset()
        .horizontal_components()
        .map(|size| size.resolve_to_option(containing_block_size.width, |val, basis| tree.calc(val, basis)));
    let inset_vertical = style
        .inset()
        .vertical_components()
        .map(|size| size.resolve_to_option(containing_block_size.height, |val, basis| tree.calc(val, basis)));
    let padding = style
        .padding()
        .map(|p| p.resolve_or_zero(Some(containing_block_size.width), |val, basis| tree.calc(val, basis)));
    let border = style
        .border()
        .map(|p| p.resolve_or_zero(Some(containing_block_size.width), |val, basis| tree.calc(val, basis)));
    let padding_border_size = (padding + border).sum_axes();

    let box_sizing_adjustment =
        if style.box_sizing() == BoxSizing::ContentBox { padding_border_size } else { Size::ZERO };

    // VIPER-PATCH(subgrid-placement): a subgrid is stretched in its subgridded axes, its sizes there ignored (CSS
    // Grid 2 section 9).
    let size_style = subgrid_unsized(style.size(), subgridded, Dimension::auto());
    let inherent_size = size_style
        .maybe_resolve(containing_block_size, |val, basis| tree.calc(val, basis))
        .maybe_apply_aspect_ratio(aspect_ratio)
        .maybe_add(box_sizing_adjustment);
    // VIPER-PATCH(subgrid-placement): ignored in a subgridded axis.
    let min_size = subgrid_unsized(style.min_size(), subgridded, LengthPercentageAuto::auto())
        .maybe_resolve(containing_block_size, |val, basis| tree.calc(val, basis))
        .maybe_add(box_sizing_adjustment)
        .or(padding_border_size.map(Some))
        .maybe_max(padding_border_size)
        .maybe_apply_aspect_ratio(aspect_ratio);
    // VIPER-PATCH(subgrid-placement): ignored in a subgridded axis.
    let max_size = subgrid_unsized(style.max_size(), subgridded, LengthPercentageAuto::auto())
        .maybe_resolve(containing_block_size, |val, basis| tree.calc(val, basis))
        .maybe_apply_aspect_ratio(aspect_ratio)
        .maybe_add(box_sizing_adjustment);

    // Resolve `normal` alignment (the default if alignment is set on neither the parent or the node itself)
    // Note: if the child has a preferred aspect ratio but neither width or height are set, then the width is stretched
    // and the then height is calculated from the width according the aspect ratio
    // Replaced elements are not stretched: they are sized as for `start`.
    // See: https://www.w3.org/TR/css-grid-1/#grid-item-sizing
    // VIPER-PATCH(subgrid-placement): stretched in a subgridded axis.
    let justify_self = if subgridded.horizontal { AlignSelf::STRETCH } else { justify_self };
    let align_self = if subgridded.vertical { AlignSelf::STRETCH } else { align_self };
    let alignment_styles = InBothAbsAxis {
        horizontal: match justify_self.keyword {
            AlignItemsKeyword::Normal => {
                if inherent_size.width.is_some() || size_style.width.is_sizing_keyword() || is_replaced {
                    AlignSelf::START
                } else {
                    AlignSelf::STRETCH
                }
            }
            _ => justify_self,
        },
        vertical: match align_self.keyword {
            AlignItemsKeyword::Normal => {
                if inherent_size.height.is_some()
                    || size_style.height.is_sizing_keyword()
                    || aspect_ratio.is_some()
                    || is_replaced
                {
                    AlignSelf::START
                } else {
                    AlignSelf::STRETCH
                }
            }
            _ => align_self,
        },
    };

    // Note: This is not a bug. It is part of the CSS spec that both horizontal and vertical margins
    // resolve against the WIDTH of the grid area.
    let margin = style
        .margin()
        .map(|margin| margin.resolve_to_option(containing_block_size.width, |val, basis| tree.calc(val, basis)));

    drop(style);

    let grid_area_minus_item_margins_size = Size {
        width: grid_area_size.width.maybe_sub(margin.left).maybe_sub(margin.right).max(0.0),
        // VIPER-PATCH(last-baseline): less both shims.
        height: (grid_area_size.height.maybe_sub(margin.top).maybe_sub(margin.bottom) - baseline_shims.height.sum())
            .max(0.0),
    };

    // A size that is a sizing keyword (min-content, max-content, fit-content,
    // fit-content(...), stretch) either resolves to an exact size or is resolved
    // by measuring the item under the corresponding available space constraint
    let keyword_width = inherent_size.width.is_none().then(|| {
        resolve_sizing_keyword(
            size_style.width,
            Some(grid_area_minus_item_margins_size.width),
            Some(containing_block_size.width),
            |val, basis| tree.calc(val, basis),
        )
    });
    let keyword_height = inherent_size.height.is_none().then(|| {
        resolve_sizing_keyword(
            size_style.height,
            Some(grid_area_minus_item_margins_size.height),
            Some(containing_block_size.height),
            |val, basis| tree.calc(val, basis),
        )
    });

    // If both axes need to be measured then resolve them with a single measure call
    // VIPER-PATCH(layout-float-alias): the alias.
    let keyword_measured_size: Size<Option<LayoutFloat>> = match (&keyword_width, &keyword_height) {
        (
            Some(Some(SizingKeywordResolution::Measure(available_width))),
            Some(Some(SizingKeywordResolution::Measure(available_height))),
        ) if !position.is_out_of_flow() => tree
            .measure_child_size_both(
                node,
                Size::NONE,
                containing_block_size.map(Option::Some),
                Size { width: *available_width, height: *available_height },
                SizingMode::InherentSize,
                Line::FALSE,
            )
            .map(Option::Some),
        _ => Size::NONE,
    };

    // If node is absolutely positioned and width is not set explicitly, then deduce it
    // from left, right and container_content_box if both are set.
    let width = inherent_size.width.or_else(|| {
        // Apply width derived from both the left and right properties of an absolutely
        // positioned element being set
        if position.is_out_of_flow() {
            if let (Some(left), Some(right)) = (inset_horizontal.start, inset_horizontal.end) {
                return Some(f32_max(grid_area_minus_item_margins_size.width - left - right, 0.0));
            }
        }

        if let Some(Some(resolution)) = keyword_width {
            return Some(match resolution {
                SizingKeywordResolution::Exact(width) => width,
                SizingKeywordResolution::Measure(available_width) => keyword_measured_size.width.unwrap_or_else(|| {
                    tree.measure_child_size(
                        node,
                        Size::NONE,
                        containing_block_size.map(Option::Some),
                        Size {
                            width: available_width,
                            height: AvailableSpace::Definite(grid_area_minus_item_margins_size.height),
                        },
                        SizingMode::InherentSize,
                        AbsoluteAxis::Horizontal,
                        Line::FALSE,
                    )
                }),
            });
        }

        // Apply width based on stretch alignment if:
        //  - Alignment style is "stretch"
        //  - The node is not absolutely positioned
        //  - The node does not have auto margins in this axis.
        if margin.left.is_some()
            && margin.right.is_some()
            && alignment_styles.horizontal == AlignSelf::STRETCH
            && !position.is_out_of_flow()
        {
            return Some(grid_area_minus_item_margins_size.width);
        }

        None
    });

    // Reapply aspect ratio after stretch and absolute position width adjustments
    let Size { width, height } = Size { width, height: inherent_size.height }.maybe_apply_aspect_ratio(aspect_ratio);

    let height = height.or_else(|| {
        if position.is_out_of_flow() {
            if let (Some(top), Some(bottom)) = (inset_vertical.start, inset_vertical.end) {
                return Some(f32_max(grid_area_minus_item_margins_size.height - top - bottom, 0.0));
            }
        }

        if let Some(Some(resolution)) = keyword_height {
            return Some(match resolution {
                SizingKeywordResolution::Exact(height) => height,
                SizingKeywordResolution::Measure(available_height) => {
                    keyword_measured_size.height.unwrap_or_else(|| {
                        tree.measure_child_size(
                            node,
                            Size { width, height: None },
                            containing_block_size.map(Option::Some),
                            Size {
                                width: width
                                    .map(AvailableSpace::Definite)
                                    .unwrap_or(AvailableSpace::Definite(grid_area_minus_item_margins_size.width)),
                                height: available_height,
                            },
                            SizingMode::InherentSize,
                            AbsoluteAxis::Vertical,
                            Line::FALSE,
                        )
                    })
                }
            });
        }

        // Apply height based on stretch alignment if:
        //  - Alignment style is "stretch"
        //  - The node is not absolutely positioned
        //  - The node does not have auto margins in this axis.
        if margin.top.is_some()
            && margin.bottom.is_some()
            && alignment_styles.vertical == AlignSelf::STRETCH
            && !position.is_out_of_flow()
        {
            return Some(grid_area_minus_item_margins_size.height);
        }

        None
    });
    // Reapply aspect ratio after stretch and absolute position height adjustments
    let Size { width, height } = Size { width, height }.maybe_apply_aspect_ratio(aspect_ratio);

    // Clamp size by min and max width/height
    let Size { width, height } = Size { width, height }.maybe_clamp(min_size, max_size);

    // Layout node
    let size = if position.is_out_of_flow() && (width.is_none() || height.is_none()) {
        tree.measure_child_size_both(
            node,
            Size { width, height },
            containing_block_size.map(Option::Some),
            grid_area_minus_item_margins_size.map(AvailableSpace::Definite),
            SizingMode::InherentSize,
            Line::FALSE,
        )
        .map(Some)
    } else {
        Size { width, height }
    };

    let mut layout_output = tree.perform_child_layout(
        node,
        size,
        containing_block_size.map(Option::Some),
        grid_area_minus_item_margins_size.map(AvailableSpace::Definite),
        SizingMode::InherentSize,
        Line::FALSE,
    );

    // Resolve final size
    let Size { width, height } = size.unwrap_or(layout_output.size).maybe_clamp(min_size, max_size);

    let (x, x_margin) = align_item_within_area(
        Line { start: grid_area.left, end: grid_area.right },
        alignment_styles.horizontal,
        width,
        position,
        inset_horizontal,
        margin.horizontal_components(),
        // VIPER-PATCH(last-baseline): the inline-axis shim.
        baseline_shims.width,
        direction,
    );
    let (y, y_margin) = align_item_within_area(
        Line { start: grid_area.top, end: grid_area.bottom },
        alignment_styles.vertical,
        height,
        position,
        inset_vertical,
        margin.vertical_components(),
        // VIPER-PATCH(last-baseline): the block-axis shims.
        baseline_shims.height,
        Direction::Ltr,
    );

    let scrollbar_size = Size {
        width: if overflow.y == Overflow::Scroll { scrollbar_width } else { 0.0 },
        height: if overflow.x == Overflow::Scroll { scrollbar_width } else { 0.0 },
    };

    let resolved_margin = Rect { left: x_margin.start, right: x_margin.end, top: y_margin.start, bottom: y_margin.end };

    // Bubble out-of-flow candidates from the item's subtree, translating anchors from
    // item-relative to container-relative coordinates
    let mut item_candidates = layout_output.oof_candidates.take();
    if !item_candidates.is_empty() {
        item_candidates.translate(Point { x, y });
        bubbled_candidates.append(&mut item_candidates);
    }

    tree.set_unrounded_layout(
        node,
        &Layout {
            order,
            location: Point { x, y },
            size: Size { width, height },
            #[cfg(feature = "content_size")]
            scrollable_overflow_rect: layout_output.scrollable_overflow_rect,
            scrollbar_size,
            padding,
            border,
            margin: resolved_margin,
        },
    );

    #[cfg(feature = "content_size")]
    let contribution = {
        // Contributions to the container's scrollable overflow rect are measured from the
        // container's padding-box origin (mirrored for RTL), matching the scrollable overflow region.
        let contribution_location = if direction.is_rtl() {
            Point { x: container_border_box_width - (x + width) - container_border.right, y: y - container_border.top }
        } else {
            Point { x: x - container_border.left, y: y - container_border.top }
        };
        compute_scrollable_overflow_contribution(
            contribution_location,
            Size { width, height },
            layout_output.scrollable_overflow_rect,
            overflow,
            contain,
            container_is_scroll_container,
        )
    };
    #[cfg(not(feature = "content_size"))]
    let contribution = Rect::ZERO;

    // VIPER-PATCH(last-baseline): the item's baselines from its border box's top edge, a scroll container's clamped
    // to its border box as the flex and block algorithms clamp them.
    // VIPER-PATCH(layout-float-alias): the alias.
    let clamp = |baseline: Option<LayoutFloat>| {
        if overflow.y.is_scroll_container() {
            baseline.map(|baseline| baseline.min(height).max(0.0))
        } else {
            baseline
        }
    };
    // VIPER-PATCH(scroll-last-baseline): a last baseline at the block-end margin edge (CSS Box Alignment 3 §9.1).
    let last =
        if last_baseline_at_margin_edge { Some(height + y_margin.end) } else { clamp(layout_output.baselines.last) };
    let baselines = Baselines { first: clamp(layout_output.baselines.first), last };

    (contribution, Point { x, y }, Size { width, height }, baselines)
}

/// Align and size a grid item along a single axis
#[allow(clippy::too_many_arguments)]
pub(super) fn align_item_within_area(
    // VIPER-PATCH(layout-float-alias): the alias.
    grid_area: Line<LayoutFloat>,
    alignment_style: AlignSelf,
    // VIPER-PATCH(layout-float-alias): the alias.
    resolved_size: LayoutFloat,
    position: Position,
    // VIPER-PATCH(layout-float-alias): the alias.
    inset: Line<Option<LayoutFloat>>,
    margin: Line<Option<LayoutFloat>>,
    // VIPER-PATCH(last-baseline): a shim at each end.
    // VIPER-PATCH(layout-float-alias): the alias.
    baseline_shims: Line<LayoutFloat>,
    direction: Direction,
    // VIPER-PATCH(layout-float-alias): the alias.
) -> (LayoutFloat, Line<LayoutFloat>) {
    // Calculate grid area dimension in the axis
    let non_auto_margin = Line {
        start: margin.start.unwrap_or(0.0) + baseline_shims.start,
        end: margin.end.unwrap_or(0.0) + baseline_shims.end,
    };
    let grid_area_size = f32_max(grid_area.end - grid_area.start, 0.0);
    let free_space = f32_max(grid_area_size - resolved_size - non_auto_margin.sum(), 0.0);

    // Expand auto margins to fill available space
    let auto_margin_count = margin.start.is_none() as u8 + margin.end.is_none() as u8;
    // VIPER-PATCH(layout-float-alias): the alias.
    let auto_margin_size = if auto_margin_count > 0 { free_space / auto_margin_count as LayoutFloat } else { 0.0 };
    // VIPER-PATCH(grid-baseline-margin): the item's own margin is returned, and the baseline shim only moves the
    // item. Upstream returns the start margin with the shim that moves a baseline-aligned item down to its row's
    // baseline, so the used margin-top it reports holds the shim (a flex item's holds none). Chrome moves the item by
    // its baseline offset and reports the item's own margin.
    let own_margin =
        Line { start: margin.start.unwrap_or(auto_margin_size), end: margin.end.unwrap_or(auto_margin_size) };
    // VIPER-PATCH(last-baseline): the last-baseline shim moves a last-baseline item up from the end of its area.
    let resolved_margin =
        Line { start: own_margin.start + baseline_shims.start, end: own_margin.end + baseline_shims.end };

    let overflows = resolved_size + non_auto_margin.sum() > grid_area_size;
    // VIPER-PATCH(last-baseline): the fallback alignment of `last baseline` is `safe self-end`, so an item that
    // overflows its area is start-aligned (CSS Box Alignment 3 §4.2, §4.4).
    let alignment_keyword = if alignment_style.keyword == AlignItemsKeyword::LastBaseline && overflows {
        AlignItemsKeyword::Start
    } else {
        resolve_self_alignment_safety(alignment_style, overflows)
    };

    // Compute offset in the axis
    let alignment_based_offset = match alignment_keyword {
        // TODO: Add support for baseline alignment. For now we treat it as "start".
        AlignItemsKeyword::Start
        | AlignItemsKeyword::FlexStart
        | AlignItemsKeyword::Baseline
        | AlignItemsKeyword::Stretch => {
            if direction.is_rtl() {
                grid_area_size - resolved_size - resolved_margin.end
            } else {
                resolved_margin.start
            }
        }
        // VIPER-PATCH(last-baseline): a last-baseline item is end-aligned with its shim after it, which puts its last
        // baseline at its group's shared distance from the end of its area: the group takes its fallback alignment,
        // `safe self-end` (CSS Box Alignment 3 §9.3), and a lone item is end-aligned.
        AlignItemsKeyword::End | AlignItemsKeyword::FlexEnd | AlignItemsKeyword::LastBaseline => {
            if direction.is_rtl() {
                resolved_margin.start
            } else {
                grid_area_size - resolved_size - resolved_margin.end
            }
        }
        AlignItemsKeyword::Center => {
            (grid_area_size - resolved_size + resolved_margin.start - resolved_margin.end) / 2.0
        }
        // Normal is resolved, and SelfStart/SelfEnd are resolved to Start/End against
        // the item's own direction, in `align_and_position_item`.
        AlignItemsKeyword::Normal | AlignItemsKeyword::SelfStart | AlignItemsKeyword::SelfEnd => unreachable!(),
    };

    let offset_within_area = if position.is_out_of_flow() {
        match (inset.start, inset.end) {
            (Some(start), Some(end)) => {
                if direction.is_rtl() {
                    grid_area_size - end - resolved_size - non_auto_margin.end
                } else {
                    start + non_auto_margin.start
                }
            }
            (Some(start), None) => start + non_auto_margin.start,
            (None, Some(end)) => grid_area_size - end - resolved_size - non_auto_margin.end,
            (None, None) => alignment_based_offset,
        }
    } else {
        alignment_based_offset
    };

    let mut start = grid_area.start + offset_within_area;
    if position == Position::Relative {
        let relative_inset = if direction.is_rtl() {
            inset.end.map(|pos| -pos).or(inset.start)
        } else {
            inset.start.or(inset.end.map(|pos| -pos))
        };
        start += relative_inset.unwrap_or(0.0);
    }

    // VIPER-PATCH(grid-baseline-margin): the margin without the baseline shim.
    (start, own_margin)
}

/// `sizes` with `auto` in the axes in which the item is a subgrid
// VIPER-PATCH(subgrid-placement): a subgrid's sizes are ignored in its subgridded axes.
fn subgrid_unsized<T>(sizes: Size<T>, subgridded: InBothAbsAxis<bool>, auto: T) -> Size<T>
where
    T: Copy,
{
    Size {
        width: if subgridded.horizontal { auto } else { sizes.width },
        height: if subgridded.vertical { auto } else { sizes.height },
    }
}
