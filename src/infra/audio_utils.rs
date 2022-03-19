use ffmpeg_next::{format, media};

pub fn get_best_audio_stream_index(input_ctx: &format::context::input::Input) -> Option<usize> {
    let input = input_ctx
        .streams()
        .best(media::Type::Audio);
    match input {
        Some(stream) => Some(stream.index()),
        None => None,
    }
}