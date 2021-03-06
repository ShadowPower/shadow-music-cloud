extern crate ffmpeg_next as ffmpeg;

use std::path::Path;

use anyhow::{Ok, Result};
use ffmpeg::{format, frame, Packet, decoder, encoder, filter};

use super::{audio_utils, audio_filter};

pub struct Transcoder {
    pub output_filter_spec: Option<String>,
    pub codec: Option<String>,
    pub channels: Option<i32>,
    pub sample_rate: Option<i32>,
    pub bit_rate: Option<usize>,
    pub max_bit_rate: Option<usize>,
}

impl Transcoder {
    fn process_filtered_frames(
        filter: &mut filter::Graph,
        decoder: &mut decoder::Audio,
        encoder: &mut encoder::Audio,
        output_time_base: ffmpeg::Rational,
        output_ctx: &mut format::context::Output
    ) -> Result<()> {
        let mut filtered = frame::Audio::empty();
        // 从音频滤镜接收解码后的帧
        while filter.get("out").unwrap().sink().frame(&mut filtered).is_ok() {
            // 发送给编码器处理
            encoder.send_frame(&filtered)?;
            Transcoder::receive_and_process_encoded_packet(decoder, encoder, output_time_base, output_ctx)?;
        }
        Ok(())
    }

    fn receive_and_process_encoded_packet(
        decoder: &mut decoder::Audio,
        encoder: &mut encoder::Audio,
        output_time_base: ffmpeg::Rational,
        output_ctx: &mut format::context::Output
    ) -> Result<()> {
        // 取得编码后的音频数据
        let mut encoded = Packet::empty();
        while encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
            encoded.rescale_ts(decoder.time_base(), output_time_base);
            encoded.write_interleaved(output_ctx)?;
        }
        Ok(())
    }

    fn receive_and_process_decoded_frame (
        decoder: &mut decoder::Audio,
        filter: &mut filter::Graph,
        encoder: &mut encoder::Audio,
        output_time_base: ffmpeg::Rational,
        output_ctx: &mut format::context::Output
    ) -> Result<()> {
        // 取得解码后的音频数据
        let mut decoded = frame::Audio::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            let timestamp = decoded.timestamp();
            decoded.set_pts(timestamp);

            // 发送给音频滤镜
            filter.get("in").unwrap().source().add(&decoded)?;
            Transcoder::process_filtered_frames(filter, decoder, encoder, output_time_base, output_ctx)?;
        }
        Ok(())
    }

    pub fn transcode<P: AsRef<Path>>(&self, input: &P, output: &P) -> Result<()> {
        // 输入输出上下文
        let mut input_ctx = format::input(&input)?;
        let mut output_ctx = format::output(&output)?;

        // 创建解码器
        let audio_stream = audio_utils::find_best_stream(&input_ctx)?;
        let audio_stream_index = audio_stream.index();
        let mut decoder = audio_utils::create_decoder_by_stream(audio_stream)?;

        // 编码器参数
        let codec = self.codec
            .as_deref()
            .map(|name| {audio_utils::create_codec_by_name(name)})
            .unwrap_or(audio_utils::guess_codec_by_path(output, &output_ctx))?;
        let channels = self.channels.unwrap_or(decoder.channel_layout().channels());
        let sample_rate = self.sample_rate.unwrap_or(decoder.rate() as i32);
        let bit_rate = self.bit_rate.unwrap_or(decoder.bit_rate());
        let max_bit_rate = self.max_bit_rate.unwrap_or(decoder.max_bit_rate());

        // 创建编码器并配置输出上下文
        let (mut encoder, output_time_base) = audio_utils::create_encoder_with_output_ctx(
            codec, &mut output_ctx, channels, decoder.rate() as i32,
            sample_rate, bit_rate, max_bit_rate)?;

        // 写文件头
        output_ctx.set_metadata(input_ctx.metadata().to_owned());
        output_ctx.write_header()?;

        let filter_spec = self.output_filter_spec.as_deref().unwrap_or("anull");
        let mut filter = audio_filter::filter(filter_spec, &decoder, &encoder)?;

        // 开始转码
        for (stream, mut packet) in input_ctx.packets() {
            // 取出容器内的音频数据
            if stream.index() == audio_stream_index {
                // 转换时间基
                packet.rescale_ts(stream.time_base(), decoder.time_base());
                decoder.send_packet(&packet)?;
                Transcoder::receive_and_process_decoded_frame(&mut decoder, &mut filter, &mut encoder, output_time_base, &mut output_ctx)?;
            }
        }

        // 解码结束
        decoder.send_eof()?;
        Transcoder::receive_and_process_decoded_frame(&mut decoder, &mut filter, &mut encoder, output_time_base, &mut output_ctx)?;

        // flush filter
        filter.get("in").unwrap().source().flush()?;
        Transcoder::process_filtered_frames(&mut filter, &mut decoder, &mut encoder, output_time_base, &mut output_ctx)?;

        // 编码结束
        encoder.send_eof()?;
        Transcoder::receive_and_process_encoded_packet(&mut decoder, &mut encoder, output_time_base, &mut output_ctx)?;

        // 写文件尾
        output_ctx.write_trailer()?;
        Ok(())
    }
}