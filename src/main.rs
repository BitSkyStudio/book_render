use std::{collections::HashMap, env::args, fmt::format, fs::File, io::Read, path::Path};

use image::{DynamicImage, EncodableLayout, codecs::webp::WebPEncoder};
use uuid::Uuid;
use webp::{Encoder, PixelLayout};
use zip::ZipArchive;

fn main() {
    let args = args().collect::<Vec<_>>();
    let book_folder = Path::new(&args[1]);
    let output_folder = Path::new(&args[2]);
    if std::fs::exists(output_folder).unwrap_or(false) {
        std::fs::remove_dir_all(output_folder).unwrap();
    }
    std::fs::create_dir(output_folder);
    std::fs::create_dir({
        let mut pages_folder = output_folder.to_path_buf();
        pages_folder.push("pages");
        pages_folder
    });
    let mut pages = HashMap::new();
    for volume in std::fs::read_dir(book_folder).unwrap() {
        let Ok(volume) = volume else {
            continue;
        };
        let mut zip = ZipArchive::new(File::open(volume.path()).unwrap()).unwrap();
        let mut own_pages = Vec::new();
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).unwrap();
            if entry.is_dir() {
                continue;
            }
            //println!("entry {}", entry.name());
            let id = Uuid::new_v4();
            own_pages.push(id);
            let mut data = Vec::new();
            entry.read_to_end(&mut data);
            pages.insert(id, image::load_from_memory(&data[..]).unwrap());
        }
        let mut html_path = output_folder.to_path_buf();
        html_path.push(volume.file_name());
        html_path.set_extension("html");
        let mut content = String::new();
        for (i, own_page) in own_pages.into_iter().enumerate() {
            let page_image = pages.get(&own_page).unwrap();
            content += &format!(
                r#"<h3 style="text-align: center;">{}</h3><img src="pages/{}.webm" loading="lazy" width={} height={} style="display:block;margin-left: auto;margin-right: auto;">"#,
                i + 1,
                own_page,
                page_image.width(),
                page_image.height()
            );
            /*content += &format!(
                r#"    <img src="pages/{}.webm" loading="lazy" style="display:block;width:100vw;height={}vw;">"#,
                own_page,
                page_image.width() as f32 / page_image.height() as f32 * 100.
            );*/
            content += "\n";
        }
        std::fs::write(
            html_path,
            include_str!("template.html")
                .to_string()
                .replace("$here$", &content)
                .replace("$title$", &args[2]),
        )
        .unwrap();
    }
    for (id, page) in pages {
        let mut page_path = output_folder.to_path_buf();
        page_path.push("pages");
        page_path.push(id.to_string());
        page_path.set_extension("webm");
        let rgba = page.to_rgba8();
        let raw_data = rgba.as_bytes();
        let encoder = Encoder::new(raw_data, PixelLayout::Rgba, page.width(), page.height());
        std::fs::write(page_path, &encoder.encode(70.)[..]);
    }
}
