use taffy::geometry::Size;
use taffy::LayoutFloat; // VIPER-PATCH(layout-f64): the alias.

pub struct ImageContext {
    // VIPER-PATCH(layout-f64): layout numbers of the alias type.
    pub width: LayoutFloat,
    pub height: LayoutFloat,
}

pub fn image_measure_function(
    // VIPER-PATCH(layout-f64): layout numbers of the alias type.
    known_dimensions: taffy::geometry::Size<Option<LayoutFloat>>,
    image_context: &ImageContext,
    // VIPER-PATCH(layout-f64): layout numbers of the alias type.
) -> taffy::geometry::Size<LayoutFloat> {
    match (known_dimensions.width, known_dimensions.height) {
        (Some(width), Some(height)) => Size { width, height },
        (Some(width), None) => Size { width, height: (width / image_context.width) * image_context.height },
        (None, Some(height)) => Size { width: (height / image_context.height) * image_context.width, height },
        (None, None) => Size { width: image_context.width, height: image_context.height },
    }
}
