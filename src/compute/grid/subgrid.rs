//! Subgrids (CSS Grid 2 section 9): the tracks and line names a parent grid gives the subgridded axes of a grid item
//! that is a subgrid, and how the subgrid places its own items in them.
// VIPER-PATCH(subgrid-placement): the module.
// VIPER-PATCH(subgrid-sizing): its items in the parent's track sizing.
//! The subgrid's items size the parent's tracks of those axes in its place, with extra margins.
// VIPER-PATCH(subgrid-own-tracks): its own tracks.
//! A subgrid of one axis sizes its own tracks of the other in its box once the parent has sized its tracks there, and
//! its items are measured in them.

use super::types::{GridItem, GridTrack, GridTrackKind, NamedLineResolver, TrackCounts};
use super::OriginZeroLine;
// VIPER-PATCH(subgrid-own-tracks): the abstract axis.
use crate::geometry::{AbsoluteAxis, AbstractAxis, InBothAbsAxis, Line, Rect};
// VIPER-PATCH(subgrid-own-tracks): auto tracks.
use crate::style::{MaxTrackSizingFunction, MinTrackSizingFunction};
use crate::style_helpers::TaffyAuto;
use crate::sys::{DefaultCheapStr, Vec};
use crate::{CheapCloneStr, LayoutFloat};
// VIPER-PATCH(subgrid-own-tracks): the collected items' range.
use core::ops::Range;

/// The parent's tracks and line names for one subgridded axis, in the subgrid's own order of that axis
#[derive(Debug, Clone, PartialEq)]
pub struct SubgridAxis<S: CheapCloneStr = DefaultCheapStr> {
    /// The used sizes of the parent's tracks the subgrid spans and of the gutters between them, alternating, a track
    /// first and last (`2n - 1` entries for `n` tracks). A gutter's size includes the space the parent's content
    /// distribution (`space-between` and the like) adds to it.
    pub sizes: Vec<LayoutFloat>,
    /// The parent's names of each of the `n + 1` lines, including the `<area>-start` and `<area>-end` names of its
    /// areas' edges
    pub line_names: Vec<Vec<S>>,
    /// The parent's gap, without the content distribution's space: a subgrid's `normal` gap is this one, and half the
    /// difference of a gap of its own from it is extra margin on the items it separates
    // VIPER-PATCH(subgrid-sizing): the field.
    pub gap: LayoutFloat,
}

impl<S: CheapCloneStr> SubgridAxis<S> {
    /// The number of tracks the subgrid spans
    pub fn track_count(&self) -> u16 {
        u16::try_from((self.sizes.len() + 1) / 2).unwrap_or(u16::MAX)
    }
}

/// What a parent grid gives a subgrid: the tracks of each of its subgridded axes
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubgridTracks<S: CheapCloneStr = DefaultCheapStr> {
    /// The columns, when the subgrid's columns are subgridded
    pub columns: Option<SubgridAxis<S>>,
    /// The rows, when the subgrid's rows are subgridded
    pub rows: Option<SubgridAxis<S>>,
}

/// The parent's tracks of each subgridded axis, from what the tree holds for the subgrid
pub(super) fn given<S: CheapCloneStr>(tracks: Option<SubgridTracks<S>>) -> InBothAbsAxis<Option<SubgridAxis<S>>> {
    let tracks = tracks.unwrap_or_default();
    let usable = |axis: &SubgridAxis<S>| !axis.sizes.is_empty();
    InBothAbsAxis { horizontal: tracks.columns.filter(usable), vertical: tracks.rows.filter(usable) }
}

/// A subgridded axis's explicit track count, without repetitions, in place of its template's
pub(super) fn explicit_counts<S: CheapCloneStr>(given: &Option<SubgridAxis<S>>, template: (u16, u16)) -> (u16, u16) {
    given.as_ref().map_or(template, |axis| (0, axis.track_count()))
}

/// An item's lines in a subgridded axis of `count` tracks, clamped into it as an overlarge grid clamps them: a
/// subgridded axis has no implicit tracks (CSS Grid 2 section 9), so an area extending beyond it ends at its edge,
/// and one entirely beyond it takes its edge track
pub(super) fn clamp_to_span(line: Line<OriginZeroLine>, count: u16) -> Line<OriginZeroLine> {
    let last = i16::try_from(count).unwrap_or(i16::MAX);
    let (start, end) = (line.start.0.clamp(0, last), line.end.0.clamp(0, last));
    let (start, end) = match (start < end, start == last) {
        (true, _) => (start, end),
        (false, true) => (last - 1, last),
        (false, false) => (0, 1),
    };
    Line { start: OriginZeroLine(start), end: OriginZeroLine(end) }
}

/// Clamp the items into the subgridded axes, and give those axes their explicit tracks only
pub(super) fn clamp_items<S: CheapCloneStr>(
    given: &InBothAbsAxis<Option<SubgridAxis<S>>>,
    items: &mut [GridItem],
    counts: &mut InBothAbsAxis<TrackCounts>,
) {
    if let Some(axis) = &given.horizontal {
        let count = axis.track_count();
        items.iter_mut().for_each(|item| item.column = clamp_to_span(item.column, count));
        counts.horizontal = TrackCounts { negative_implicit: 0, explicit: count, positive_implicit: 0 };
    }
    if let Some(axis) = &given.vertical {
        let count = axis.track_count();
        items.iter_mut().for_each(|item| item.row = clamp_to_span(item.row, count));
        counts.vertical = TrackCounts { negative_implicit: 0, explicit: count, positive_implicit: 0 };
    }
}

/// Replace a subgridded axis's tracks with the parent's: tracks and gutters of exactly the given sizes, the first track
/// less the subgrid's margin, border and padding at its start (`insets.start`) and the last less those at its end,
/// never below zero, so that the subgrid's lines fall on the parent's (its edge items keep clear of those edges)
// VIPER-PATCH(subgrid-sizing): its own gap.
///
/// With a `gap` of its own, each gutter is less the difference from the parent's gap and the tracks beside it take
/// half of it each, so that the gutters stay centred on the parent's.
pub(super) fn parent_tracks<S: CheapCloneStr>(
    tracks: &mut Vec<GridTrack>,
    given: &SubgridAxis<S>,
    insets: Line<LayoutFloat>,
    // VIPER-PATCH(subgrid-sizing): the subgrid's own gap.
    gap: Option<LayoutFloat>,
) {
    let last = given.sizes.len() - 1;
    // VIPER-PATCH(subgrid-sizing): the difference, and half of it at each side of a track with a gutter there.
    let difference = gap.map_or(0.0, |gap| given.gap - gap);
    let half = |index: usize| if index < given.sizes.len() { difference / 2.0 } else { 0.0 };
    tracks.clear();
    tracks.push(GridTrack::exact(GridTrackKind::Gutter, 0.0));
    for (index, size) in given.sizes.iter().enumerate() {
        let start = if index == 0 { insets.start } else { 0.0 };
        let end = if index == last { insets.end } else { 0.0 };
        // VIPER-PATCH(subgrid-sizing): with the difference.
        let (kind, size) = match index % 2 {
            0 => (GridTrackKind::Track, size + if index > 0 { half(index - 1) } else { 0.0 } + half(index + 1)),
            _ => (GridTrackKind::Gutter, size - difference),
        };
        tracks.push(GridTrack::exact(kind, (size - start - end).max(0.0)));
    }
    tracks.push(GridTrack::exact(GridTrackKind::Gutter, 0.0));
}

/// A container's gap in an axis as its subgrid items see it: a subgrid's `normal` gap in a subgridded axis is its
/// parent's, any other its resolved `gap`
// VIPER-PATCH(subgrid-sizing): the gap the subgrid items' gap differences are taken from.
pub(super) fn effective_gap<S: CheapCloneStr>(
    given: &Option<SubgridAxis<S>>,
    normal: bool,
    gap: LayoutFloat,
) -> LayoutFloat {
    match given {
        Some(axis) if normal => axis.gap,
        _ => gap,
    }
}

/// What a parent grid asks of a subgrid item to size its tracks with the subgrid's items (CSS Grid 2 section 9)
// VIPER-PATCH(subgrid-sizing): the request.
pub(super) struct Collect<S: CheapCloneStr> {
    /// Stand-in tracks for its subgridded axes: the parent's line names and gap
    pub given: InBothAbsAxis<Option<SubgridAxis<S>>>,
    /// The parent's gap in each axis, as [`effective_gap`] gives it
    pub gap: InBothAbsAxis<LayoutFloat>,
    /// The subgrid's items, in its lines and with their extra margins, for the parent to translate into its own
    pub items: Vec<GridItem>,
    /// The axis a subgrid of one axis does not subgrid, when the parent has sized its tracks there: the subgrid sizes
    /// its own tracks of it, in its box, and gives its items their areas there
    // VIPER-PATCH(subgrid-own-tracks): the field.
    pub own: Option<AbstractAxis>,
    /// Whether `given` holds the parent's sized tracks rather than stand-ins
    // VIPER-PATCH(subgrid-own-tracks): the field.
    pub sized: bool,
    /// Set by the parent when it does not size the own tracks of a subgrid of one axis: the subgrid then hands over
    /// its items only when at most one own track fills its area in the other axis
    // VIPER-PATCH(subgrid-own-tracks): the field.
    pub fallback: bool,
    /// Set when the subgrid hands over no items and sizes the parent's tracks as an ordinary item instead
    // VIPER-PATCH(subgrid-own-tracks): the field, as subgrid-sizing's was.
    pub ordinary: bool,
}

impl<S: CheapCloneStr> Collect<S> {
    /// The request for `subgrid`, of the parent's gap `gap`
    pub(super) fn new(subgrid: &SubgridItem<S>, item: &GridItem, gap: InBothAbsAxis<LayoutFloat>) -> Self {
        let stand_in = |subgridded: bool, line: Line<OriginZeroLine>, names: &Vec<Vec<S>>, gap: LayoutFloat| {
            subgridded.then(|| {
                let count = usize::try_from(line.end.0 - line.start.0).unwrap_or(0).max(1);
                let sizes = core::iter::repeat(0.0).take(2 * count - 1).collect();
                SubgridAxis { sizes, line_names: names.clone(), gap }
            })
        };
        let given = InBothAbsAxis {
            horizontal: stand_in(subgrid.axes.horizontal, item.column, &subgrid.line_names.horizontal, gap.horizontal),
            vertical: stand_in(subgrid.axes.vertical, item.row, &subgrid.line_names.vertical, gap.vertical),
        };
        // VIPER-PATCH(subgrid-own-tracks): no own tracks to size.
        Collect { given, gap, items: Vec::new(), own: None, sized: false, fallback: false, ordinary: false }
    }

    /// The parent's sized tracks in place of the stand-ins, for `subgrid`, its item `item`
    // VIPER-PATCH(subgrid-own-tracks): the method.
    pub(super) fn sized(
        &mut self,
        subgrid: &SubgridItem<S>,
        item: &GridItem,
        columns: &[GridTrack],
        rows: &[GridTrack],
    ) {
        let sizes = |tracks: &[GridTrack], line: Line<u16>| -> Vec<LayoutFloat> {
            let span = &tracks[usize::from(line.start) + 1..usize::from(line.end)];
            span.iter().map(|track| track.exact_size.unwrap_or(track.base_size)).collect()
        };
        if let Some(axis) = &mut self.given.horizontal {
            axis.sizes = sizes(columns, item.column_indexes);
            if subgrid.reversed {
                axis.sizes.reverse();
            }
        }
        if let Some(axis) = &mut self.given.vertical {
            axis.sizes = sizes(rows, item.row_indexes);
        }
        self.sized = true;
    }
}

/// Make the tracks, not the gutters, `auto`: a subgridded axis's stand-ins while the parent has not sized it, so that
/// the items are measured in an indefinite area there
// VIPER-PATCH(subgrid-own-tracks): the function.
pub(super) fn auto_tracks(tracks: &mut [GridTrack]) {
    for track in tracks.iter_mut().filter(|track| track.kind == GridTrackKind::Track) {
        *track = GridTrack::new(MinTrackSizingFunction::AUTO, MaxTrackSizingFunction::AUTO);
    }
}

/// Give a subgrid's items their extra margins as items of its parent (`insets` its margin, border and padding in its
/// own order of each axis, `rtl` that of its columns, `half` half the difference of its gaps from the parent's in each
/// axis): at the subgrid's edge lines in a subgridded axis its insets there, at the others `half`, and all its insets
/// in an axis it does not subgrid
// VIPER-PATCH(subgrid-own-tracks): none in an axis where an item has an area of its own, which is inside them.
// VIPER-PATCH(subgrid-sizing): the extra margins.
pub(super) fn add_margins<S: CheapCloneStr>(
    items: &mut [GridItem],
    given: &InBothAbsAxis<Option<SubgridAxis<S>>>,
    insets: InBothAbsAxis<Line<LayoutFloat>>,
    half: InBothAbsAxis<LayoutFloat>,
    rtl: bool,
) {
    let extra =
        |line: Line<OriginZeroLine>, given: &Option<SubgridAxis<S>>, insets: Line<LayoutFloat>, half: LayoutFloat| {
            let Some(axis) = given else { return insets };
            let last = i16::try_from(axis.track_count()).unwrap_or(i16::MAX);
            Line {
                start: if line.start.0 == 0 { insets.start } else { half },
                end: if line.end.0 == last { insets.end } else { half },
            }
        };
    for item in items {
        let mut columns = extra(item.column, &given.horizontal, insets.horizontal, half.horizontal);
        let mut rows = extra(item.row, &given.vertical, insets.vertical, half.vertical);
        // VIPER-PATCH(subgrid-own-tracks): none there.
        if item.own_area.width.is_some() {
            columns = Line { start: 0.0, end: 0.0 };
        }
        if item.own_area.height.is_some() {
            rows = Line { start: 0.0, end: 0.0 };
        }
        let (left, right) = if rtl { (columns.end, columns.start) } else { (columns.start, columns.end) };
        item.subgrid_margin = item.subgrid_margin + Rect { left, right, top: rows.start, bottom: rows.end };
    }
}

/// Translate a subgrid item's items, collected in its lines, into the parent's: into its area in a subgridded axis (in
/// reverse when its columns run against the parent's), its area in the other, where they size no tracks; those that
/// then size none are dropped
// VIPER-PATCH(subgrid-sizing): the translation.
pub(super) fn translate<S: CheapCloneStr>(
    subgrid: &SubgridItem<S>,
    item: &GridItem,
    mut items: Vec<GridItem>,
) -> Vec<GridItem> {
    let (columns, rows) = (item.column, item.row);
    let into = |line: Line<OriginZeroLine>, area: Line<OriginZeroLine>, reversed: bool| {
        let (start, end) = if reversed {
            (area.end.0 - line.end.0, area.end.0 - line.start.0)
        } else {
            (area.start.0 + line.start.0, area.start.0 + line.end.0)
        };
        Line { start: OriginZeroLine(start), end: OriginZeroLine(end) }
    };
    items.retain_mut(|inner| {
        inner.column = if subgrid.axes.horizontal { into(inner.column, columns, subgrid.reversed) } else { columns };
        inner.row = if subgrid.axes.vertical { into(inner.row, rows, false) } else { rows };
        inner.sizes_tracks.width &= subgrid.axes.horizontal;
        inner.sizes_tracks.height &= subgrid.axes.vertical;
        inner.sizes_tracks.width || inner.sizes_tracks.height
    });
    items
}

/// A subgrid's margin, border and padding at the start and end of each axis, in its own order of the axis (`edges`
/// is physical; the inline start is the right edge in a right-to-left subgrid)
pub(super) fn insets(edges: Rect<LayoutFloat>, rtl: bool) -> InBothAbsAxis<Line<LayoutFloat>> {
    let horizontal =
        if rtl { Line { start: edges.right, end: edges.left } } else { Line { start: edges.left, end: edges.right } };
    InBothAbsAxis { horizontal, vertical: Line { start: edges.top, end: edges.bottom } }
}

/// The sizes of the tracks, and of the gutters between them, from track-vector index `start` (a span's start line)
/// to `end` (its end line). A gutter is the space between its tracks' used positions, so it includes the content
/// distribution's space; `reversed` when the tracks' offsets run right to left.
fn span_sizes(tracks: &[GridTrack], start: usize, end: usize, reversed: bool) -> Vec<LayoutFloat> {
    (start + 1..end)
        .map(|index| {
            let track = &tracks[index];
            if track.kind == GridTrackKind::Track {
                return track.base_size;
            }
            let (before, after) = (&tracks[index - 1], &tracks[index + 1]);
            if reversed {
                before.offset - after.offset - after.base_size
            } else {
                after.offset - before.offset - before.base_size
            }
        })
        .collect()
}

/// What a parent grid knows of one of its items that is a subgrid
#[derive(Debug, Clone)]
pub(super) struct SubgridItem<S: CheapCloneStr> {
    /// The item's index among the grid's items
    pub index: usize,
    /// Its subgridded axes
    pub axes: InBothAbsAxis<bool>,
    /// Whether its inline axis runs against the parent's, so that its columns are the parent's in reverse
    pub reversed: bool,
    /// The parent's names of the lines it spans, per subgridded axis
    pub line_names: InBothAbsAxis<Vec<Vec<S>>>,
    /// Where its items are among the parent's, once collected
    // VIPER-PATCH(subgrid-own-tracks): the field.
    pub collected: Range<usize>,
}

impl<S: CheapCloneStr> SubgridItem<S> {
    /// The item's spanned lines' names, once it is placed
    pub(super) fn new(
        index: usize,
        item: &GridItem,
        axes: InBothAbsAxis<bool>,
        reversed: bool,
        names: &NamedLineResolver<S>,
    ) -> Self {
        let span = |axis: AbsoluteAxis, line: Line<OriginZeroLine>, subgridded: bool| {
            if !subgridded {
                return Vec::new();
            }
            let count = u16::try_from(line.end.0 - line.start.0).unwrap_or(0);
            names.span_line_names(axis, line.start, count)
        };
        let mut line_names = InBothAbsAxis {
            horizontal: span(AbsoluteAxis::Horizontal, item.column, axes.horizontal),
            vertical: span(AbsoluteAxis::Vertical, item.row, axes.vertical),
        };
        if reversed {
            line_names.horizontal.reverse();
        }
        // VIPER-PATCH(subgrid-own-tracks): none collected yet.
        SubgridItem { index, axes, reversed, line_names, collected: 0..0 }
    }

    /// The tracks the parent gives the item, from its final tracks; `rtl` when the parent's are right to left
    pub(super) fn tracks(
        &self,
        item: &GridItem,
        columns: &[GridTrack],
        rows: &[GridTrack],
        rtl: bool,
        // VIPER-PATCH(subgrid-sizing): the parent's gaps.
        gap: InBothAbsAxis<LayoutFloat>,
    ) -> SubgridTracks<S> {
        let columns = self.axes.horizontal.then(|| {
            let (start, end) = (item.column_indexes.start as usize, item.column_indexes.end as usize);
            let mut sizes = span_sizes(columns, start, end, rtl);
            if self.reversed {
                sizes.reverse();
            }
            // VIPER-PATCH(subgrid-sizing): the gap.
            SubgridAxis { sizes, line_names: self.line_names.horizontal.clone(), gap: gap.horizontal }
        });
        let rows = self.axes.vertical.then(|| {
            let (start, end) = (item.row_indexes.start as usize, item.row_indexes.end as usize);
            // VIPER-PATCH(subgrid-sizing): the gap.
            let sizes = span_sizes(rows, start, end, false);
            SubgridAxis { sizes, line_names: self.line_names.vertical.clone(), gap: gap.vertical }
        });
        SubgridTracks { columns, rows }
    }
}
