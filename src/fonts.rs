use eg_seven_segment::{SevenSegmentStyle, SevenSegmentStyleBuilder};
use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::{Alignment, Baseline, TextStyle, TextStyleBuilder};
use profont::{PROFONT_12_POINT, PROFONT_14_POINT, PROFONT_18_POINT, PROFONT_24_POINT};

pub struct CharacterStyles<'a> {
    pub default_text_style: Option<TextStyle>,
    pub fill_style: Option<PrimitiveStyle<Rgb565>>,
    pub time_segment_style: Option<SevenSegmentStyle<Rgb565>>,
    pub center_aligned_text_style: Option<TextStyle>,
    pub default_character_style: Option<MonoTextStyle<'a, Rgb565>>,
    pub small_character_style: Option<MonoTextStyle<'a, Rgb565>>,
    pub large_character_style: Option<MonoTextStyle<'a, Rgb565>>,
    pub medium_character_style: Option<MonoTextStyle<'a, Rgb565>>,

}

impl<'a> CharacterStyles<'a> {
    pub fn new() -> Self {
        Self::new_with_color(Rgb565::WHITE)
    }

    pub fn new_with_color(text_color: Rgb565) -> Self {
        CharacterStyles {
            default_text_style: Some(TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Top)
                .build()),
            fill_style: Some(PrimitiveStyle::with_fill(Rgb565::BLACK)),
            time_segment_style: Some(SevenSegmentStyleBuilder::new()
                .digit_size(Size::new(40, 80)) // digits are 10x20 pixels
                .digit_spacing(5)              // 5px spacing between digits
                .segment_width(10)              // 5px wide segments
                .segment_color(text_color)  // active segments are green
                .build()),
            center_aligned_text_style: Some(TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .baseline(Baseline::Top)
                .build()),
            default_character_style: Some(MonoTextStyle::new(
                &PROFONT_14_POINT,
                text_color)),
            small_character_style: Some(MonoTextStyle::new(
                &PROFONT_12_POINT,
                text_color)),
            large_character_style: Some(MonoTextStyle::new(
                &PROFONT_24_POINT,
                text_color)),
            medium_character_style: Some(MonoTextStyle::new(
                &PROFONT_18_POINT,
                text_color)),
        }
    }

    pub fn set_background_color(&mut self, background_color: Rgb565) {
        let mut style = self.default_character_style();
        style.background_color = Some(background_color);
        self.default_character_style.replace(style);

        style = self.date_character_style();
        style.background_color = Some(background_color);
        self.medium_character_style.replace(style);

        style = self.small_character_style();
        style.background_color = Some(background_color);
        self.small_character_style.replace(style);

        style = self.large_character_style();
        style.background_color = Some(background_color);
        self.large_character_style.replace(style);

        let mut segment_style = self.time_segment_style();
        segment_style.inactive_segment_color = Some(background_color);
        self.time_segment_style.replace(segment_style);
    }

    pub fn default_text_style(&self) -> TextStyle {
        self.default_text_style.unwrap()
    }
    pub fn fill_style(&self) -> PrimitiveStyle<Rgb565> {
        self.fill_style.unwrap()
    }
    pub fn time_segment_style(&self) -> SevenSegmentStyle<Rgb565> {
        self.time_segment_style.unwrap()
    }
    pub fn center_aligned_text_style(&self) -> TextStyle {
        self.center_aligned_text_style.unwrap()
    }
    pub fn date_character_style(&self) -> MonoTextStyle<'a, Rgb565> {
        self.medium_character_style.unwrap()
    }
    pub fn default_character_style(&self) -> MonoTextStyle<'a, Rgb565> {
        self.default_character_style.unwrap()
    }
    pub fn small_character_style(&self) -> MonoTextStyle<'a, Rgb565> {
        self.small_character_style.unwrap()
    }
    pub fn large_character_style(&self) -> MonoTextStyle<'a, Rgb565> {
        self.large_character_style.unwrap()
    }
    pub fn medium_character_style(&self) -> MonoTextStyle<'a, Rgb565> {
        self.medium_character_style.unwrap()
    }
}
