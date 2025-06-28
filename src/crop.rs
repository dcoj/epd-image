use crate::dither;
use crate::epd;
use crate::error::Result;
use exoquant::Color;
use image::DynamicImage;
use std::io::BufWriter;
use std::num::NonZeroU32;

pub async fn crop_image(image: DynamicImage, width: u32, height: u32) -> Result<Vec<u8>> {
    let res = smartcrop::find_best_crop(
        &image,
        NonZeroU32::new(width.into()).unwrap(),
        NonZeroU32::new(height.into()).unwrap(),
    )
    .expect("Failed to find crop");
    // println!(
    //     "cropping to x{} y{} w{} h{} ({})",
    //     res.crop.x, res.crop.y, res.crop.width, res.crop.height, res.score.total
    // );
    let c = res.crop;
    let cropped = image.crop_imm(c.x, c.y, c.width, c.height);
    let scaled = cropped.resize(width, height, image::imageops::FilterType::Lanczos3);

    //   return indexed_image ESP
    let pixels = scaled
        .as_rgb8()
        .unwrap()
        .pixels()
        .map(|p| Color::new(p[0], p[1], p[2], 0xFF))
        .collect();

    let (indexed_image, _) = dither::dither_image(pixels)?;
    let mut data = Vec::new();
    {
        let mut writer = BufWriter::new(&mut data);
        epd::write_epd(&mut writer, &indexed_image).unwrap();
    }
    Ok(data)

    // Return JPEG
    // let mut out = Vec::new();
    // scaled.write_to(
    //     &mut Cursor::new(&mut out),
    //     image::ImageOutputFormat::Jpeg(100),
    // )?;
    // Ok(out.clone())
}
