use std::{path::Path, cmp, fs};

use anyhow::Result;
use image::{imageops::FilterType, ImageOutputFormat};

/// 生成缩略图
/// @param input 输入文件
/// @param output 输出文件
pub fn convert_to_thumbnail<P: AsRef<Path>>(input: &P, output: &P) -> Result<()> {
    let img = image::open(input)?;
    let new_width = cmp::min(img.width(), 512);
    let new_heigth = cmp::min(img.height(), 512);
    
    let scaled = img.resize(new_width, new_heigth, FilterType::Triangle);
    fs::create_dir_all(output.as_ref().parent().unwrap())?;
    let mut output_file = fs::File::create(output)?;
    scaled.write_to(&mut output_file, ImageOutputFormat::Jpeg(80))?;
    Ok(())
}