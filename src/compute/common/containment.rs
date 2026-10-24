//! VIPER-PATCH(size-containment): size and inline-size containment, shared by the block, flexbox,
//! grid and leaf algorithms, and VIPER-PATCH(scroll-last-baseline): a block-axis scroll
//! container's last baseline.
//!
//! A size containment box is laid out in two phases (CSS Containment 2 §3.1): its size is first
//! determined as if it had no content ("sizing as if empty"), then its content is laid out in that
//! now fixed size ("laying out in-place"). Inline-size containment does the first phase in the
//! inline axis only (§3.2), the width in Taffy's horizontal axes.
use crate::geometry::Size;
use crate::style::{BaselineSource, Contain, CoreStyle};
use crate::tree::{LayoutInput, LayoutOutput, RequestedAxis, RunMode};

/// The inputs with which a box with containment `contain` lays out its content: `inputs` whose
/// unknown dimensions in the size-contained axes are the box's size as if it had no content,
/// which `lay_out_as_if_empty` computes from the inputs it is given (the box's own algorithm run
/// with no children). Those dimensions are fixed sizes, so they are definite. Other inputs, and
/// hidden layout, are returned unchanged.
pub(crate) fn size_contained_inputs(
    contain: Contain,
    inputs: LayoutInput,
    lay_out_as_if_empty: impl FnOnce(LayoutInput) -> LayoutOutput,
) -> LayoutInput {
    let axes = contain.size_contained_axes();
    let known = inputs.known_dimensions;
    let width = axes.width && known.width.is_none();
    let height = axes.height && known.height.is_none();
    if (!width && !height) || inputs.run_mode == RunMode::PerformHiddenLayout {
        return inputs;
    }
    let empty =
        lay_out_as_if_empty(LayoutInput { run_mode: RunMode::ComputeSize, axis: RequestedAxis::Both, ..inputs });
    LayoutInput {
        known_dimensions: Size {
            width: known.width.or(width.then_some(empty.size.width)),
            height: known.height.or(height.then_some(empty.size.height)),
        },
        known_dimensions_are_definite: Size {
            width: inputs.known_dimensions_are_definite.width || width,
            height: inputs.known_dimensions_are_definite.height || height,
        },
        ..inputs
    }
}

/// Whether the box with `style` has a last baseline set at its block-end margin edge: CSS Box
/// Alignment 3 §9.1 gives one, "for legacy reasons", to a block container that is a block-axis
/// scroll container and whose `baseline-source` is `auto`. Its parent then uses that edge instead
/// of the box's content's last baseline clamped to its border box.
pub(crate) fn last_baseline_at_margin_edge(style: &impl CoreStyle) -> bool {
    style.overflow().y.is_scroll_container()
        && style.baseline_source() == BaselineSource::Auto
        && style.is_block_container()
}
