//! Generic code that is shared between multiple layout algorithms
pub(crate) mod alignment;
// VIPER-PATCH(size-containment): the shared containment code.
pub(crate) mod containment;
pub(crate) mod sizing_keyword;

#[cfg(feature = "content_size")]
pub(crate) mod scrollable_overflow;
