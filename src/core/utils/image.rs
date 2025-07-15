use gtk4::glib::object::Cast;
use image::{DynamicImage, ImageReader};

pub fn open_image(image: &str) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let img = ImageReader::open(image)?.decode()?;
    Ok(img)
}

pub fn dynamic_image_to_pixbuf(
    img: &DynamicImage,
) -> Result<gtk4::gdk_pixbuf::Pixbuf, gtk4::glib::Error> {
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let bytes = rgba_img.into_raw();
    let pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_bytes(
        &gtk4::glib::Bytes::from(&bytes),
        gtk4::gdk_pixbuf::Colorspace::Rgb,
        true, // alpha
        8,    // bits pp
        width as i32,
        height as i32,
        (width * 4) as i32,
    );
    Ok(pixbuf)
}

pub fn dynamic_image_to_texture(
    img: &DynamicImage,
) -> Result<gtk4::gdk::Texture, gtk4::glib::Error> {
    let pixbuf = dynamic_image_to_pixbuf(&img).unwrap();
    let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
    Ok(texture)
}

pub fn get_texture_from_image(img: gtk4::Image) -> gtk4::gdk::Texture {
    let paintable = img.paintable();
    return paintable.unwrap().downcast::<gtk4::gdk::Texture>().unwrap();
}

pub fn invert_image() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = open_image("assets/teapot.png").unwrap();
    img.invert();
    img.save("assets/invteapot.png")?;

    Ok(())
}
