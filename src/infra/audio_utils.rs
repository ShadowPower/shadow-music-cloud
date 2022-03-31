extern crate ffmpeg_next as ffmpeg;

use std::path::Path;

use anyhow::{Context, Ok, Result};
use ffmpeg::{codec, decoder, encoder, format, media, Stream};
use lofty::FileProperties;

/// 获取最佳音频流索引
/// @param input_ctx 输入媒体文件上下文
/// @return 最佳音频流索引
pub fn get_best_audio_stream_index(input_ctx: &format::context::Input) -> Option<usize> {
    input_ctx.streams().best(media::Type::Audio).map(|stream| stream.index())
}

// ---- 用于转码 ----

/// 获取最佳音频流
/// @param input_ctx 输入媒体文件上下文
/// @return 最佳音频流
pub fn find_best_stream(input_ctx: &format::context::Input) -> Result<Stream> {
    input_ctx
        .streams()
        .best(media::Type::Audio)
        .with_context(|| "Failed to find best stream")
}

/// 根据音频流创建音频解码器
/// @param stream 音频流
/// @return 音频解码器
pub fn create_decoder_by_stream(stream: Stream) -> Result<decoder::Audio> {
    let context = codec::context::Context::from_parameters(stream.parameters())?;
    let mut decoder = context.decoder().audio()?;
    decoder.set_parameters(stream.parameters())?;
    Ok(decoder)
}

/// 根据输出的文件路径猜测音频编码
/// @param path 输出文件路径
/// @param output_ctx 输出媒体文件上下文
/// @return 音频编码
pub fn guess_codec_by_path<P: AsRef<Path>>(path: &P, output_ctx: &format::context::Output) -> Result<codec::Audio> {
    Ok(encoder::find(output_ctx.format().codec(path, media::Type::Audio))
        .with_context(|| "Failed to find audio codec")?
        .audio()?)
}

/// 根据名称创建音频编码
/// @param name 音频编码库名称（例如: "libopus"）
/// @return 音频编码
pub fn create_codec_by_name(name: &str) -> Result<codec::Audio> {
    Ok(encoder::find_by_name(name)
        .with_context(|| format!("Failed to find {} codec", name))?
        .audio()?)
}

/// 创建编码器，并配置输出上下文
/// @param codec 音频编码
/// @param output_ctx 输出媒体文件上下文
/// @param channels 通道数
/// @param sample_rate 采样率
/// @param bit_rate 码率
/// @param max_bit_rate 最大码率
/// @return 编码器，输出时间基
pub fn create_encoder_with_output_ctx(
    codec: codec::Audio,
    output_ctx: &mut format::context::Output,
    channels: i32,
    source_sample_rate: i32,
    target_sample_rate: i32,
    bit_rate: usize,
    max_bit_rate: usize,
) -> Result<(encoder::Audio, ffmpeg::Rational)> {
    let global = output_ctx
        .format()
        .flags()
        .contains(format::flag::Flags::GLOBAL_HEADER);

    let mut output = output_ctx.add_stream(codec)?;
    let context = codec::context::Context::from_parameters(output.parameters())?;
    let mut encoder = context.encoder().audio()?;

    if global {
        encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
    }

    let channel_layout = codec
        .channel_layouts()
        .map(|cls| cls.best(channels))
        .unwrap_or(ffmpeg::channel_layout::ChannelLayout::STEREO);
    encoder.set_channel_layout(channel_layout);
    encoder.set_channels(channel_layout.channels());

    encoder.set_format(
        codec
            .formats()
            .expect("Unknown supported formats")
            .next()
            .with_context(|| "Failed to find supported format")?,
    );

    encoder.set_bit_rate(bit_rate);
    encoder.set_max_bit_rate(max_bit_rate);

    encoder.set_rate(target_sample_rate);
    encoder.set_time_base((1, source_sample_rate));
    output.set_time_base((1, source_sample_rate));

    let encoder = encoder.open_as(codec)?;
    output.set_parameters(&encoder);

    Ok((encoder, output.time_base()))
}

// ---- 文件信息 ----

/// 从媒体文件中获取标签
/// @param file_path 媒体文件路径
/// @return 标签
pub fn get_tags_from_media_file<P: AsRef<Path>>(file_path: &P) -> Result<lofty::Tag> {
    let tagged_file = lofty::Probe::open(file_path)?.read(true)?;

    let tag = tagged_file.primary_tag()
        .unwrap_or(tagged_file.first_tag().with_context(|| "No tags found")?);

    Ok(tag.clone())
}

/// 从媒体文件中获取属性
/// @param file_path 媒体文件路径
/// @return 属性
pub fn get_properties_from_media_file<P: AsRef<Path>>(file_path: &P) -> Result<lofty::FileProperties> {
    Ok(lofty::Probe::open(file_path)?.read(true)?.properties().clone())
}