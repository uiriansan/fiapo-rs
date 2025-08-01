use gtk4::{gdk, gdk_pixbuf, glib};
use image::DynamicImage;

pub fn dynamic_image_to_pixbuf(img: &DynamicImage) -> Result<gdk_pixbuf::Pixbuf, glib::Error> {
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let bytes = rgba_img.into_raw();
    let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
        &glib::Bytes::from(&bytes),
        gdk_pixbuf::Colorspace::Rgb,
        true, // alpha
        8,    // bits per pixel
        width as i32,
        height as i32,
        (width * 4) as i32,
    );
    Ok(pixbuf)
}

// Convert a DynamicImage from the Image crate to Gdk.Texture
pub fn dynamic_image_to_texture(img: &DynamicImage) -> Result<gdk::Texture, glib::Error> {
    let pixbuf = dynamic_image_to_pixbuf(&img).unwrap();
    let texture = gdk::Texture::for_pixbuf(&pixbuf);
    Ok(texture)
}
