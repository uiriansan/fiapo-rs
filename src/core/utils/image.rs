use image::ImageReader;

pub fn invert_image() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("assets/teapot.png")?.decode()?;
    img.invert();
    img.save("assets/invteapot.png")?;

    Ok(())
}
