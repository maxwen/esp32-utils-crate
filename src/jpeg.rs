extern crate alloc;

use alloc::vec::Vec;
use core::{iter, slice};
use core::marker::PhantomData;

use embedded_graphics::draw_target::{DrawTarget, DrawTargetExt};
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Size};
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::{PixelColor, Rgb555, Rgb565, Rgb888, RgbColor};
use embedded_graphics::pixelcolor::raw::{BigEndian, LittleEndian, RawU16, RawU24};
use embedded_graphics::primitives::Rectangle;

#[derive(Debug)]
pub struct Jpeg<'a, C> {
    pixels: &'a Vec<u8>,
    size: Size,
    color_type: PhantomData<C>,
}

impl<'a, C> Jpeg<'a, C>
    where
        C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub fn new(pixels: &'a Vec<u8>, size: Size) -> Self {
        Jpeg {
            pixels,
            size,
            color_type: PhantomData,
        }
    }
}

impl<C> OriginDimensions for Jpeg<'_, C> where C: From<Rgb555> + From<Rgb565> + From<Rgb888> + PixelColor {
    fn size(&self) -> Size {
        self.size
    }
}

impl<C> ImageDrawable for Jpeg<'_, C>
    where
        C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
        where
            D: DrawTarget<Color=C>,
    {
        let area = self.bounding_box();
        target.fill_contiguous(
            &area,
            RawColors::<RawU24>::new(self.pixels, self.size).map(|raw| {
                Rgb888::from(raw).into()
            }))
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
}

fn bytes_per_row(width: usize) -> usize {
    width * 3
}

impl<'a, R> RawColors<'a, R>
    where
        RawDataSlice<'a, R, BigEndian>: IntoIterator<Item=R>,
{
    pub(crate) fn new(pixels: &'a Vec<u8>, size: Size) -> Self {
        let width = size.width as usize;
        Self {
            rows: pixels.chunks_exact(bytes_per_row(width)),
            current_row: RawDataSlice::new(&[]).into_iter().take(0),
            width,
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

