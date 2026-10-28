//! Subgrids (CSS Grid 2 section 9): the tracks and line names a parent grid gives the subgridded axes of a grid item
//! that is a subgrid, and how the subgrid places its own items in them.
// VIPER-PATCH(subgrid-placement): the module.

use super::types::{GridItem, GridTrack, GridTrackKind, NamedLineResolver, TrackCounts};
use super::OriginZeroLine;
use crate::geometry::{AbsoluteAxis, InBothAbsAxis, Line, Rect};
use crate::sys::{DefaultCheapStr, Vec};
use crate::{CheapCloneStr, LayoutFloat};

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
pub(super) fn parent_tracks<S: CheapCloneStr>(
    tracks: &mut Vec<GridTrack>,
    given: &SubgridAxis<S>,
    insets: Line<LayoutFloat>,
) {
    let last = given.sizes.len() - 1;
    tracks.clear();
    tracks.push(GridTrack::exact(GridTrackKind::Gutter, 0.0));
    for (index, size) in given.sizes.iter().enumerate() {
        let start = if index == 0 { insets.start } else { 0.0 };
        let end = if index == last { insets.end } else { 0.0 };
        let kind = if index % 2 == 0 { GridTrackKind::Track } else { GridTrackKind::Gutter };
        tracks.push(GridTrack::exact(kind, (size - start - end).max(0.0)));
    }
    tracks.push(GridTrack::exact(GridTrackKind::Gutter, 0.0));
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
        SubgridItem { index, axes, reversed, line_names }
    }

    /// The tracks the parent gives the item, from its final tracks; `rtl` when the parent's are right to left
    pub(super) fn tracks(
        &self,
        item: &GridItem,
        columns: &[GridTrack],
        rows: &[GridTrack],
        rtl: bool,
    ) -> SubgridTracks<S> {
        let columns = self.axes.horizontal.then(|| {
            let (start, end) = (item.column_indexes.start as usize, item.column_indexes.end as usize);
            let mut sizes = span_sizes(columns, start, end, rtl);
            if self.reversed {
                sizes.reverse();
            }
            SubgridAxis { sizes, line_names: self.line_names.horizontal.clone() }
        });
        let rows = self.axes.vertical.then(|| {
            let (start, end) = (item.row_indexes.start as usize, item.row_indexes.end as usize);
            SubgridAxis { sizes: span_sizes(rows, start, end, false), line_names: self.line_names.vertical.clone() }
        });
        SubgridTracks { columns, rows }
    }
}
