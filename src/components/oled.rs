use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{DrawTarget, Drawable};
use embedded_graphics::text::Text;
use esp_idf_hal::gpio::{Gpio21, Gpio22};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_hal::units::Hertz;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

pub fn initiate_display(
    sda: Gpio21<'static>,
    scl: Gpio22<'static>,
    i2c_driver: I2C0<'static>,
) -> Ssd1306<
    I2CInterface<I2cDriver<'static>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
> {
    let config = I2cConfig::new().baudrate(Hertz(100_000));

    let i2c = I2cDriver::new(i2c_driver, sda, scl, &config).unwrap();

    println!("Initializing OLED at 0x3C");

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    display
}

pub fn render_text(
    display: &mut Ssd1306<
        I2CInterface<I2cDriver<'static>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    text: &str,
) {
    display.clear(BinaryColor::Off).unwrap();
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    Text::new(text, Point::new(0, 16), style)
        .draw(display)
        .unwrap();

    display.flush().unwrap();

    println!("OLED text displayed successfully");
}
