use aws_sdk_polly::model::{Engine, LanguageCode, OutputFormat, VoiceId};

use crate::paragraph::Paragraph;

pub async fn process(mut input: Paragraph) -> Paragraph {
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_polly::Client::new(&shared_config);
    let output = client.synthesize_speech()
        .engine(Engine::Neural)
        .language_code(LanguageCode::CmnCn)
        .output_format(OutputFormat::Mp3)
        .text(&input.cleaned_text)
        .voice_id(VoiceId::Zhiyu)
        .send().await;

    match output {
        Ok(stream) => {
            let bytes = stream.audio_stream.collect().await.unwrap();
            input.audio = Some(bytes.into_bytes());
            input
        }
        Err(e) => {
            println!("{}", e);
            input
        }
    }
}
