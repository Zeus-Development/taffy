//! Contains numerical helper traits and functions
#![allow(clippy::manual_clamp)]

use crate::geometry::Size;
use crate::style::AvailableSpace;
// VIPER-PATCH(layout-float-alias): the alias.
use crate::LayoutFloat;

/// A trait to conveniently calculate minimums and maximums when some data may not be defined
///
/// If the left-hand value is [`None`], these operations return [`None`].
/// If the right-hand value is [`None`], it is treated as zero.
pub trait MaybeMath<In, Out> {
    /// Returns the minimum of `self` and `rhs`
    fn maybe_min(self, rhs: In) -> Out;

    /// Returns the maximum of `self` and `rhs`
    fn maybe_max(self, rhs: In) -> Out;

    /// Returns `self` clamped between `min` and `max`
    fn maybe_clamp(self, min: In, max: In) -> Out;

    /// Adds `self` and `rhs`.
    fn maybe_add(self, rhs: In) -> Out;

    /// Subtracts rhs from `self`, treating [`None`] values as default
    fn maybe_sub(self, rhs: In) -> Out;
}

// VIPER-PATCH(layout-float-alias): the alias.
impl MaybeMath<Option<LayoutFloat>, Option<LayoutFloat>> for Option<LayoutFloat> {
    fn maybe_min(self, rhs: Option<LayoutFloat>) -> Option<LayoutFloat> {
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l.min(r)),
            (Some(_l), None) => self,
            (None, Some(_r)) => None,
            (None, None) => None,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_max(self, rhs: Option<LayoutFloat>) -> Option<LayoutFloat> {
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l.max(r)),
            (Some(_l), None) => self,
            (None, Some(_r)) => None,
            (None, None) => None,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_clamp(self, min: Option<LayoutFloat>, max: Option<LayoutFloat>) -> Option<LayoutFloat> {
        match (self, min, max) {
            (Some(base), Some(min), Some(max)) => Some(base.min(max).max(min)),
            (Some(base), None, Some(max)) => Some(base.min(max)),
            (Some(base), Some(min), None) => Some(base.max(min)),
            (Some(_), None, None) => self,
            (None, _, _) => None,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_add(self, rhs: Option<LayoutFloat>) -> Option<LayoutFloat> {
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l + r),
            (Some(_l), None) => self,
            (None, Some(_r)) => None,
            (None, None) => None,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_sub(self, rhs: Option<LayoutFloat>) -> Option<LayoutFloat> {
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l - r),
            (Some(_l), None) => self,
            (None, Some(_r)) => None,
            (None, None) => None,
        }
    }
}

// VIPER-PATCH(layout-float-alias): the alias.
impl MaybeMath<LayoutFloat, Option<LayoutFloat>> for Option<LayoutFloat> {
    fn maybe_min(self, rhs: LayoutFloat) -> Option<LayoutFloat> {
        self.map(|val| val.min(rhs))
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_max(self, rhs: LayoutFloat) -> Option<LayoutFloat> {
        self.map(|val| val.max(rhs))
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_clamp(self, min: LayoutFloat, max: LayoutFloat) -> Option<LayoutFloat> {
        self.map(|val| val.min(max).max(min))
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_add(self, rhs: LayoutFloat) -> Option<LayoutFloat> {
        self.map(|val| val + rhs)
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_sub(self, rhs: LayoutFloat) -> Option<LayoutFloat> {
        self.map(|val| val - rhs)
    }
}

// VIPER-PATCH(layout-float-alias): the alias.
impl MaybeMath<Option<LayoutFloat>, LayoutFloat> for LayoutFloat {
    fn maybe_min(self, rhs: Option<LayoutFloat>) -> LayoutFloat {
        match rhs {
            Some(val) => self.min(val),
            None => self,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_max(self, rhs: Option<LayoutFloat>) -> LayoutFloat {
        match rhs {
            Some(val) => self.max(val),
            None => self,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_clamp(self, min: Option<LayoutFloat>, max: Option<LayoutFloat>) -> LayoutFloat {
        match (min, max) {
            (Some(min), Some(max)) => self.min(max).max(min),
            (None, Some(max)) => self.min(max),
            (Some(min), None) => self.max(min),
            (None, None) => self,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_add(self, rhs: Option<LayoutFloat>) -> LayoutFloat {
        match rhs {
            Some(val) => self + val,
            None => self,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_sub(self, rhs: Option<LayoutFloat>) -> LayoutFloat {
        match rhs {
            Some(val) => self - val,
            None => self,
        }
    }
}

// VIPER-PATCH(layout-float-alias): the alias.
impl MaybeMath<LayoutFloat, AvailableSpace> for AvailableSpace {
    fn maybe_min(self, rhs: LayoutFloat) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(val) => AvailableSpace::Definite(val.min(rhs)),
            AvailableSpace::MinContent => AvailableSpace::Definite(rhs),
            AvailableSpace::MaxContent => AvailableSpace::Definite(rhs),
        }
    }
    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_max(self, rhs: LayoutFloat) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(val) => AvailableSpace::Definite(val.max(rhs)),
            AvailableSpace::MinContent => AvailableSpace::MinContent,
            AvailableSpace::MaxContent => AvailableSpace::MaxContent,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_clamp(self, min: LayoutFloat, max: LayoutFloat) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(val) => AvailableSpace::Definite(val.min(max).max(min)),
            AvailableSpace::MinContent => AvailableSpace::MinContent,
            AvailableSpace::MaxContent => AvailableSpace::MaxContent,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_add(self, rhs: LayoutFloat) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(val) => AvailableSpace::Definite(val + rhs),
            AvailableSpace::MinContent => AvailableSpace::MinContent,
            AvailableSpace::MaxContent => AvailableSpace::MaxContent,
        }
    }
    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_sub(self, rhs: LayoutFloat) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(val) => AvailableSpace::Definite(val - rhs),
            AvailableSpace::MinContent => AvailableSpace::MinContent,
            AvailableSpace::MaxContent => AvailableSpace::MaxContent,
        }
    }
}

// VIPER-PATCH(layout-float-alias): the alias.
impl MaybeMath<Option<LayoutFloat>, AvailableSpace> for AvailableSpace {
    fn maybe_min(self, rhs: Option<LayoutFloat>) -> AvailableSpace {
        match (self, rhs) {
            (AvailableSpace::Definite(val), Some(rhs)) => AvailableSpace::Definite(val.min(rhs)),
            (AvailableSpace::Definite(val), None) => AvailableSpace::Definite(val),
            (AvailableSpace::MinContent, Some(rhs)) => AvailableSpace::Definite(rhs),
            (AvailableSpace::MinContent, None) => AvailableSpace::MinContent,
            (AvailableSpace::MaxContent, Some(rhs)) => AvailableSpace::Definite(rhs),
            (AvailableSpace::MaxContent, None) => AvailableSpace::MaxContent,
        }
    }
    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_max(self, rhs: Option<LayoutFloat>) -> AvailableSpace {
        match (self, rhs) {
            (AvailableSpace::Definite(val), Some(rhs)) => AvailableSpace::Definite(val.max(rhs)),
            (AvailableSpace::Definite(val), None) => AvailableSpace::Definite(val),
            (AvailableSpace::MinContent, _) => AvailableSpace::MinContent,
            (AvailableSpace::MaxContent, _) => AvailableSpace::MaxContent,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_clamp(self, min: Option<LayoutFloat>, max: Option<LayoutFloat>) -> AvailableSpace {
        match (self, min, max) {
            (AvailableSpace::Definite(val), Some(min), Some(max)) => AvailableSpace::Definite(val.min(max).max(min)),
            (AvailableSpace::Definite(val), None, Some(max)) => AvailableSpace::Definite(val.min(max)),
            (AvailableSpace::Definite(val), Some(min), None) => AvailableSpace::Definite(val.max(min)),
            (AvailableSpace::Definite(val), None, None) => AvailableSpace::Definite(val),
            (AvailableSpace::MinContent, _, _) => AvailableSpace::MinContent,
            (AvailableSpace::MaxContent, _, _) => AvailableSpace::MaxContent,
        }
    }

    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_add(self, rhs: Option<LayoutFloat>) -> AvailableSpace {
        match (self, rhs) {
            (AvailableSpace::Definite(val), Some(rhs)) => AvailableSpace::Definite(val + rhs),
            (AvailableSpace::Definite(val), None) => AvailableSpace::Definite(val),
            (AvailableSpace::MinContent, _) => AvailableSpace::MinContent,
            (AvailableSpace::MaxContent, _) => AvailableSpace::MaxContent,
        }
    }
    // VIPER-PATCH(layout-float-alias): the alias.
    fn maybe_sub(self, rhs: Option<LayoutFloat>) -> AvailableSpace {
        match (self, rhs) {
            (AvailableSpace::Definite(val), Some(rhs)) => AvailableSpace::Definite(val - rhs),
            (AvailableSpace::Definite(val), None) => AvailableSpace::Definite(val),
            (AvailableSpace::MinContent, _) => AvailableSpace::MinContent,
            (AvailableSpace::MaxContent, _) => AvailableSpace::MaxContent,
        }
    }
}

impl<In, Out, T: MaybeMath<In, Out>> MaybeMath<Size<In>, Size<Out>> for Size<T> {
    fn maybe_min(self, rhs: Size<In>) -> Size<Out> {
        Size { width: self.width.maybe_min(rhs.width), height: self.height.maybe_min(rhs.height) }
    }

    fn maybe_max(self, rhs: Size<In>) -> Size<Out> {
        Size { width: self.width.maybe_max(rhs.width), height: self.height.maybe_max(rhs.height) }
    }

    fn maybe_clamp(self, min: Size<In>, max: Size<In>) -> Size<Out> {
        Size {
            width: self.width.maybe_clamp(min.width, max.width),
            height: self.height.maybe_clamp(min.height, max.height),
        }
    }

    fn maybe_add(self, rhs: Size<In>) -> Size<Out> {
        Size { width: self.width.maybe_add(rhs.width), height: self.height.maybe_add(rhs.height) }
    }

    fn maybe_sub(self, rhs: Size<In>) -> Size<Out> {
        Size { width: self.width.maybe_sub(rhs.width), height: self.height.maybe_sub(rhs.height) }
    }
}

#[cfg(test)]
mod tests {
    mod lhs_option_f32_rhs_option_f32 {
        use crate::util::MaybeMath;

        #[test]
        fn test_maybe_min() {
            assert_eq!(Some(3.0).maybe_min(Some(5.0)), Some(3.0));
            assert_eq!(Some(5.0).maybe_min(Some(3.0)), Some(3.0));
            assert_eq!(Some(3.0).maybe_min(None), Some(3.0));
            assert_eq!(None.maybe_min(Some(3.0)), None);
            assert_eq!(None.maybe_min(None), None);
        }

        #[test]
        fn test_maybe_max() {
            assert_eq!(Some(3.0).maybe_max(Some(5.0)), Some(5.0));
            assert_eq!(Some(5.0).maybe_max(Some(3.0)), Some(5.0));
            assert_eq!(Some(3.0).maybe_max(None), Some(3.0));
            assert_eq!(None.maybe_max(Some(3.0)), None);
            assert_eq!(None.maybe_max(None), None);
        }

        #[test]
        fn test_maybe_add() {
            assert_eq!(Some(3.0).maybe_add(Some(5.0)), Some(8.0));
            assert_eq!(Some(5.0).maybe_add(Some(3.0)), Some(8.0));
            assert_eq!(Some(3.0).maybe_add(None), Some(3.0));
            assert_eq!(None.maybe_add(Some(3.0)), None);
            assert_eq!(None.maybe_add(None), None);
        }

        #[test]
        fn test_maybe_sub() {
            assert_eq!(Some(3.0).maybe_sub(Some(5.0)), Some(-2.0));
            assert_eq!(Some(5.0).maybe_sub(Some(3.0)), Some(2.0));
            assert_eq!(Some(3.0).maybe_sub(None), Some(3.0));
            assert_eq!(None.maybe_sub(Some(3.0)), None);
            assert_eq!(None.maybe_sub(None), None);
        }
    }

    mod lhs_option_f32_rhs_f32 {
        use crate::util::MaybeMath;

        #[test]
        fn test_maybe_min() {
            assert_eq!(Some(3.0).maybe_min(5.0), Some(3.0));
            assert_eq!(Some(5.0).maybe_min(3.0), Some(3.0));
            assert_eq!(None.maybe_min(3.0), None);
        }

        #[test]
        fn test_maybe_max() {
            assert_eq!(Some(3.0).maybe_max(5.0), Some(5.0));
            assert_eq!(Some(5.0).maybe_max(3.0), Some(5.0));
            assert_eq!(None.maybe_max(3.0), None);
        }

        #[test]
        fn test_maybe_add() {
            assert_eq!(Some(3.0).maybe_add(5.0), Some(8.0));
            assert_eq!(Some(5.0).maybe_add(3.0), Some(8.0));
            assert_eq!(None.maybe_add(3.0), None);
        }

        #[test]
        fn test_maybe_sub() {
            assert_eq!(Some(3.0).maybe_sub(5.0), Some(-2.0));
            assert_eq!(Some(5.0).maybe_sub(3.0), Some(2.0));
            assert_eq!(None.maybe_sub(3.0), None);
        }
    }

    mod lhs_f32_rhs_option_f32 {
        use crate::util::MaybeMath;

        #[test]
        fn test_maybe_min() {
            assert_eq!(3.0.maybe_min(Some(5.0)), 3.0);
            assert_eq!(5.0.maybe_min(Some(3.0)), 3.0);
            assert_eq!(3.0.maybe_min(None), 3.0);
        }

        #[test]
        fn test_maybe_max() {
            assert_eq!(3.0.maybe_max(Some(5.0)), 5.0);
            assert_eq!(5.0.maybe_max(Some(3.0)), 5.0);
            assert_eq!(3.0.maybe_max(None), 3.0);
        }

        #[test]
        fn test_maybe_add() {
            assert_eq!(3.0.maybe_add(Some(5.0)), 8.0);
            assert_eq!(5.0.maybe_add(Some(3.0)), 8.0);
            assert_eq!(3.0.maybe_add(None), 3.0);
        }

        #[test]
        fn test_maybe_sub() {
            assert_eq!(3.0.maybe_sub(Some(5.0)), -2.0);
            assert_eq!(5.0.maybe_sub(Some(3.0)), 2.0);
            assert_eq!(3.0.maybe_sub(None), 3.0);
        }
    }
}
