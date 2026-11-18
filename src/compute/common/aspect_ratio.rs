//! VIPER-PATCH(aspect-ratio-limits) and VIPER-PATCH(aspect-ratio-content-minimum): a box's preferred aspect ratio
//! in the block and leaf algorithms (CSS Sizing 4, <https://drafts.csswg.org/css-sizing-4/#aspect-ratio>).
//!
//! A box's minimum and maximum sizes are transferred through its ratio into an axis whose size is `auto`, never
//! into a definite size (<https://drafts.csswg.org/css-sizing-4/#aspect-ratio-size-transfers>), and its automatic
//! minimum height is content-based when the ratio gives the height, for a box that is neither replaced nor a
//! scroll container (<https://drafts.csswg.org/css-sizing-4/#aspect-ratio-minimum>).
use crate::geometry::{ratio_div, Size};
use crate::style::CoreStyle;
use crate::util::MaybeMath;
use crate::LayoutFloat;

/// VIPER-PATCH(aspect-ratio-limits): `limits`, a box's resolved minimum or maximum size, with the limit of each axis
/// whose size is `auto` in `size` (the box's resolved size) filled from the other axis through `aspect_ratio`.
/// Upstream fills either axis, so `width: 200px; aspect-ratio: 4; min-height: 80px` is 320px wide there and 200px
/// wide in Chrome, and `width: 30%; aspect-ratio: 1; max-height: 90px` is 90px wide there and 30% wide in Chrome;
/// with both sizes `auto` Chrome transfers either way too (`max-height: 20px; aspect-ratio: 2` narrows a stretched
/// block to 40px).
pub(crate) fn transfer_limits(
    limits: Size<Option<LayoutFloat>>,
    size: Size<Option<LayoutFloat>>,
    aspect_ratio: Option<f32>,
) -> Size<Option<LayoutFloat>> {
    let through = limits.maybe_apply_aspect_ratio(aspect_ratio);
    Size {
        width: if size.width.is_none() { through.width } else { limits.width },
        height: if size.height.is_none() { through.height } else { limits.height },
    }
}

/// VIPER-PATCH(aspect-ratio-content-minimum): whether a height that `style`'s aspect ratio gives is only a floor
/// for its content's height, because the box's automatic minimum height is content-based: it has a preferred
/// aspect ratio, `min-height: auto`, and is neither replaced nor a scroll container. Upstream takes that height as
/// the box's size, so a 150px-wide `aspect-ratio: 3` box is 50px high however much text it holds, where Chrome
/// grows it to its text (and keeps it at 50px under `overflow: hidden` or `min-height: 0`).
pub(crate) fn height_has_content_minimum(style: &impl CoreStyle) -> bool {
    let overflow = style.overflow();
    style.aspect_ratio().is_some()
        && style.min_size().height.is_auto()
        && !overflow.x.is_scroll_container()
        && !overflow.y.is_scroll_container()
        && !style.is_compressible_replaced()
}

/// VIPER-PATCH(aspect-ratio-content-minimum): `size`, a box's resolved size, through `aspect_ratio`, but for a
/// height that only the ratio would give when it is a floor (`content_minimum`): that height is left unknown,
/// for the box's own layout to take the larger of it and its content's height.
pub(crate) fn size_through_ratio(
    size: Size<Option<LayoutFloat>>,
    aspect_ratio: Option<f32>,
    content_minimum: bool,
) -> Size<Option<LayoutFloat>> {
    if content_minimum && size.height.is_none() {
        size
    } else {
        size.maybe_apply_aspect_ratio(aspect_ratio)
    }
}

/// VIPER-PATCH(aspect-ratio-content-minimum): the border-box height through `aspect_ratio` of a box whose border
/// box is `width` wide, in the box that `box-sizing` names (`box_sizing_adjustment` is the padding and border it
/// leaves out). Upstream divides the border-box width, so a `content-box` box 180px wide with 15px of padding and
/// border a side and `aspect-ratio: 3 / 2` is 140px high there and 150px high in Chrome.
pub(crate) fn ratio_height(
    width: LayoutFloat,
    aspect_ratio: f32,
    box_sizing_adjustment: Size<LayoutFloat>,
) -> LayoutFloat {
    ratio_div(width - box_sizing_adjustment.width, aspect_ratio) + box_sizing_adjustment.height
}

/// VIPER-PATCH(aspect-ratio-content-minimum): `known`, a box's known border-box size, with the axis it lacks
/// transferred through `aspect_ratio` in the box that `box-sizing` names
pub(crate) fn known_through_ratio(
    known: Size<Option<LayoutFloat>>,
    aspect_ratio: Option<f32>,
    box_sizing_adjustment: Size<LayoutFloat>,
) -> Size<Option<LayoutFloat>> {
    known.maybe_sub(box_sizing_adjustment).maybe_apply_aspect_ratio(aspect_ratio).maybe_add(box_sizing_adjustment)
}
