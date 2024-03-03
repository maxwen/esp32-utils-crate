extern crate alloc;

use alloc::vec::Vec;
use core::{iter, slice};
use core::marker::PhantomData;

use embedded_graphics::draw_target::{DrawTarget, DrawTargetExt};
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Size};
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::{PixelColor, Rgb555, Rgb565, Rgb888};
use embedded_graphics::pixelcolor::raw::{BigEndian, RawU24, RawU32, ToBytes};
use embedded_graphics::prelude::RawData;
use embedded_graphics::primitives::Rectangle;
use zune_png::zune_core::colorspace::ColorSpace;

#[derive(Debug)]
pub struct Png<'a, C> {
    pixels: &'a Vec<u8>,
    size: Size,
    color_type: PhantomData<C>,
    color_space: ColorSpace,
    background_color: Rgb565,
}

impl<'a, C> Png<'a, C>
    where
        C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub fn new(pixels: &'a Vec<u8>, size: Size, color_space: ColorSpace, background_color: Rgb565) -> Self {
        Png {
            pixels,
            size,
            color_type: PhantomData,
            color_space,
            background_color,
        }
    }
}

fn get_color_with_alpha(alpha: f32, color: &[u8; 4], background_color: &[u8; 3]) -> Rgb888 {
    let minus_alpha = 1f32 - alpha;
    let r = (color[0] as f32 * alpha) + (minus_alpha * background_color[0] as f32);
    let g = (color[1] as f32 * alpha) + (minus_alpha * background_color[1] as f32);
    let b = (color[2] as f32 * alpha) + (minus_alpha * background_color[2] as f32);
    Rgb888::new(r as u8, g as u8, b as u8)
}

impl<C> OriginDimensions for Png<'_, C> where C: From<Rgb555> + From<Rgb565> + From<Rgb888> + PixelColor {
    fn size(&self) -> Size {
        self.size
    }
}

impl<C> ImageDrawable for Png<'_, C>
    where
        C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
        where
            D: DrawTarget<Color=C>,
    {
        let area = self.bounding_box();
        let background_color_bytes = Rgb888::from(self.background_color).to_be_bytes();
        match self.color_space {
            ColorSpace::RGB =>
                target.fill_contiguous(
                    &area,
                    RawColors::<RawU24>::new(self.pixels, self.size, self.color_space).map(|raw| {
                        Rgb888::from(raw).into()
                    })),
            ColorSpace::RGBA =>
                target.fill_contiguous(
                    &area,
                    RawColors::<RawU32>::new(self.pixels, self.size, self.color_space).map(|raw| {
                        let color_bytes = raw.into_inner().to_be_bytes();
                        if color_bytes[3] != 255 {
                            let alpha = (((color_bytes[3] as f32 / 255.0) * 10.0) as i16) as f32 / 10.0;
                            if alpha == 0.0 {
                                self.background_color.into()
                            } else {
                                get_color_with_alpha(alpha, &color_bytes, &background_color_bytes).into()
                            }
                        } else {
                            Rgb888::from(RawU24::from(raw.into_inner() >> 8)).into()
                        }
                    })),
            _ => { Ok(()) }
        }
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error> where D: DrawTarget<Color=Self::Color> {
        self.draw(&mut target.translated(-area.top_left).clipped(area))
    }
}

/// Iterator over raw pixel colors.
#[allow(missing_debug_implementations)]
pub struct RawColors<'a, R>
    where
        RawDataSlice<'a, R, BigEndian>: IntoIterator<Item=R>,
{
    rows: slice::ChunksExact<'a, u8>,
    current_row: iter::Take<<RawDataSlice<'a, R, BigEndian> as IntoIterator>::IntoIter>,
    width: usize,
    color_space: ColorSpace,
}

fn bytes_per_row(width: usize, color_space: ColorSpace) -> usize {
    if color_space == ColorSpace::RGBA {
        width * 4
    } else {
        width * 3
    }
}

impl<'a, R> RawColors<'a, R>
    where
        RawDataSlice<'a, R, BigEndian>: IntoIterator<Item=R>,
{
    pub(crate) fn new(pixels: &'a Vec<u8>, size: Size, color_space: ColorSpace) -> Self {
        let width = size.width as usize;
        Self {
            rows: pixels.chunks_exact(bytes_per_row(width, color_space)),
            current_row: RawDataSlice::new(&[]).into_iter().take(0),
            width,
            color_space,
        }
    }
}

impl<'a, R> Iterator for RawColors<'a, R>
    where
        RawDataSlice<'a, R, BigEndian>: IntoIterator<Item=R>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_row.next().or_else(|| {
            let next_row = self.rows.next().unwrap();
            self.current_row = RawDataSlice::new(next_row).into_iter().take(self.width);
            self.current_row.next()
        })
    }
}

