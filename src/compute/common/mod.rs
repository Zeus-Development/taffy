//! Generic code that is shared between multiple layout algorithms
pub(crate) mod alignment;
// VIPER-PATCH(aspect-ratio-limits), VIPER-PATCH(aspect-ratio-content-minimum): the shared aspect-ratio code.
pub(crate) mod aspect_ratio;
// VIPER-PATCH(size-containment): the shared containment code.
pub(crate) mod containment;
pub(crate) mod sizing_keyword;

#[cfg(feature = "content_size")]
pub(crate) mod scrollable_overflow;
