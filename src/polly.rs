use std::error::Error;

use aws_config::ConfigLoader;
use aws_credential_types::Credentials;
use aws_sdk_polly::error::SynthesizeSpeechError;
use aws_sdk_polly::model::{Engine, LanguageCode, OutputFormat, VoiceId};
use aws_sdk_polly::types::SdkError;
use bytes::Bytes;

use crate::AwsKey;
use crate::paragraph::Paragraph;

pub async fn process(input: Paragraph, aws_key: &AwsKey) -> Result<Bytes, SdkError<SynthesizeSpeechError>> {
    let creds = Credentials::from_keys(&aws_key.access_key_id,
                                   &aws_key.secret_access_key, None);
    let shared_config = ConfigLoader::default().credentials_provider(creds).load().await;

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
            Ok(bytes.into_bytes())
        }
        Err(e) => {
            Err(e)
        }
    }
}
