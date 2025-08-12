use core::marker::PhantomData;

use embedded_graphics::{prelude::*, primitives::Rectangle, Pixel};

pub struct CroppedWrappedConvertedDrawTarget<'a, D, C>(
    pub &'a mut D,
    pub Rectangle,
    pub PhantomData<C>,
)
where
    D: Dimensions + DrawTarget,
    C: PixelColor + Into<D::Color>;

// impl<D, C> Dimensions for CroppedWrappedConvertedDrawTarget<D, C>
// where
//     D: DrawTargetExt,
//     C: PixelColor + Into<D::Color>,
// {
//     fn bounding_box(&self) -> Rectangle {
//         Rectangle::new(Point::new(0, 0), self.1.size)
//     }
// }

impl<'a, D, C> DrawTarget for CroppedWrappedConvertedDrawTarget<'a, D, C>
where
    D: DrawTargetExt,
    C: PixelColor + Into<D::Color>,
{
    type Color = C;
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
                pixel.1.into(),
            )
        });

        self.0.cropped(&self.1).draw_iter(wrapped)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let mut cropped = self.0.cropped(&self.1);
        let result =
            cropped.fill_solid(&area.intersection(&cropped.bounding_box()), color.into())?;
        if area.bottom_right().unwrap().y > self.1.bottom_right().unwrap().y {
            cropped.fill_solid(
                &area
                    .translate(Point::new(0, -self.1.bottom_right().unwrap().y + 1))
                    .intersection(&cropped.bounding_box()),
                color.into(),
            )?;
        }
        Ok(result)
    }
}

impl<'a, D, C> OriginDimensions for CroppedWrappedConvertedDrawTarget<'a, D, C>
where
    D: DrawTargetExt,
    C: PixelColor + Into<D::Color>,
{
    fn size(&self) -> Size {
        self.1.size
    }
}
