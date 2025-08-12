use embedded_graphics::{prelude::*, primitives::Rectangle, Pixel};

pub struct CroppedWrappedDrawTarget<D: Dimensions + DrawTarget>(pub D, pub Rectangle);

// impl<D: DrawTargetExt> Dimensions for CroppedWrappedDrawTarget<D> {
//     fn bounding_box(&self) -> Rectangle {
//         Rectangle::new(Point::new(0, 0), self.1.size)
//     }
// }

impl<D: DrawTargetExt> DrawTarget for CroppedWrappedDrawTarget<D> {
    type Color = D::Color;
    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let wrapped = pixels.into_iter().map(|pixel| {
            Pixel(
                Point::new(
                    pixel.0.x,
                    pixel.0.y % (self.1.bottom_right().unwrap().y + 1),
                ),
                pixel.1,
            )
        });

        self.0.cropped(&self.1).draw_iter(wrapped)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let mut cropped = self.0.cropped(&self.1);
        let result = cropped.fill_solid(&area.intersection(&cropped.bounding_box()), color)?;
        if area.bottom_right().unwrap().y > self.1.bottom_right().unwrap().y {
            cropped.fill_solid(
                &area
                    .translate(Point::new(0, -self.1.bottom_right().unwrap().y + 1))
                    .intersection(&cropped.bounding_box()),
                color,
            )?;
        }
        Ok(result)
    }
}

impl<D: DrawTargetExt> OriginDimensions for CroppedWrappedDrawTarget<D> {
    fn size(&self) -> Size {
        self.1.size
    }
}
