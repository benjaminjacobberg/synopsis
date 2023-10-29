use std::sync::{Arc, Mutex};

use actix::prelude::*;
use rust_bert::pipelines::summarization::SummarizationModel;
use serde::{Deserialize, Serialize};
use text_splitter::TextSplitter;
use thiserror::Error;
use tokenizers::Tokenizer;

#[derive(Serialize, Deserialize, Debug, Error)]
pub enum SummarizationError {
    #[error("runetime error: {0}")]
    RuntimeError(String),
}

#[derive(Serialize, Deserialize)]
pub struct WordCountRange {
    pub min: usize,
    pub max: usize,
}

#[derive(Serialize, Deserialize, Message)]
#[rtype(result = "std::result::Result<String, SummarizationError>")]
pub struct Summarize {
    pub text: String,
    pub range: WordCountRange,
}

pub struct SummarizeActor {
    model: Arc<Mutex<Option<SummarizationModel>>>,
    tokenizer: Tokenizer,
}

impl SummarizeActor {
    pub fn new() -> Self {
        let model = Arc::new(Mutex::new(None));
        let model_clone = model.clone();

        std::thread::spawn(move || {
            let summarization_model =
                SummarizationModel::new(Default::default()).expect("failed to load model");
            let mut model = model_clone.lock().unwrap();
            *model = Some(summarization_model);
        });

        let tokenizer = Tokenizer::from_pretrained("bert-base-cased", None).unwrap();

        SummarizeActor { model, tokenizer }
    }
}

impl Actor for SummarizeActor {
    type Context = Context<Self>;
}

impl Handler<Summarize> for SummarizeActor {
    type Result = Result<String, SummarizationError>;

    fn handle(&mut self, msg: Summarize, _: &mut Context<Self>) -> Self::Result {
        let formatted_input: String = msg
            .text
            .trim()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        let max_tokens: usize = 512;
        let splitter: TextSplitter<Tokenizer> =
            TextSplitter::new(self.tokenizer.clone()).with_trim_chunks(true);

        let mut current_text = formatted_input.clone();
        let mut previous_summary = String::new();

        loop {
            let chunks: Vec<&str> = splitter.chunks(&current_text, max_tokens).collect();
            let mut summary = String::new();

            loop {
                let model: std::sync::MutexGuard<'_, Option<SummarizationModel>> =
                    self.model.lock().unwrap();
                if let Some(model) = &*model {
                    for chunk in &chunks {
                        let summarized_chunk = &model.summarize(&[chunk])[0].to_owned();
                        summary.push_str(&summarized_chunk);
                        summary.push_str(" ");
                    }
                    break;
                }
            }

            let summarized_word_count: usize = summary.trim().split_whitespace().count();

            if summarized_word_count >= msg.range.min && summarized_word_count <= msg.range.max
                || previous_summary == summary
            {
                return Ok(summary.trim().to_string());
            }

            previous_summary = summary.clone();
            current_text = summary;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_actor() {
        System::new().block_on(async {
            let actor: Addr<SummarizeActor> = SummarizeActor::new().start();
            let input: &str = "In findings published Tuesday in Cornell University's arXiv by a team of scientists \
                from the University of Montreal and a separate report published Wednesday in Nature Astronomy by a team \
                from University College London (UCL), the presence of water vapour was confirmed in the atmosphere of K2-18b, \
                a planet circling a star in the constellation Leo. This is the first such discovery in a planet in its star's \
                habitable zone — not too hot and not too cold for liquid water to exist. The Montreal team, led by Björn Benneke, \
                used data from the NASA's Hubble telescope to assess changes in the light coming from K2-18b's star as the planet \
                passed between it and Earth. They found that certain wavelengths of light, which are usually absorbed by water, \
                weakened when the planet was in the way, indicating not only does K2-18b have an atmosphere, but the atmosphere \
                contains water in vapour form. The team from UCL then analyzed the Montreal team's data using their own software \
                and confirmed their conclusion. This was not the first time scientists have found signs of water on an exoplanet, \
                but previous discoveries were made on planets with high temperatures or other pronounced differences from Earth. \
                \"This is the first potentially habitable planet where the temperature is right and where we now know there is water,\" \
                said UCL astronomer Angelos Tsiaras. \"It's the best candidate for habitability right now.\" \"It's a good sign\", \
                said Ryan Cloutier of the Harvard–Smithsonian Center for Astrophysics, who was not one of either study's authors. \
                \"Overall,\" he continued, \"the presence of water in its atmosphere certainly improves the prospect of K2-18b being \
                a potentially habitable planet, but further observations will be required to say for sure. \"
                K2-18b was first identified in 2015 by the Kepler space telescope. It is about 110 light-years from Earth and larger \
                but less dense. Its star, a red dwarf, is cooler than the Sun, but the planet's orbit is much closer, such that a year \
                on K2-18b lasts 33 Earth days. According to The Guardian, astronomers were optimistic that NASA's James Webb space \
                telescope — scheduled for launch in 2021 — and the European Space Agency's 2028 ARIEL program, could reveal more \
                about exoplanets like K2-18b.";
            let expected_output: &str = "K2-18b is a planet in its star's habitable zone, not too hot and not too cold for liquid water to exist. Scientists used data from the NASA's Hubble telescope to assess changes in the light coming from K2- 18b's star. They found that certain wavelengths of light, which are usually absorbed by water, weakened when the planet was in the way. Astronomers were optimistic that NASA's James Webb space telescope — scheduled for launch in 2021 — and the European Space Agency's 2028 ARIEL program, could reveal more about exoplanets like K2-18b. According to The Guardian, astronomers were optimistic.";
            let summary: String = actor
                .send(Summarize {
                    text: input.to_owned(),
                    range: WordCountRange { min: 50, max: 100 },
                })
                .await
                .expect("failed to send message")
                .expect("failed to summarize");

            assert_eq!(summary, expected_output);
        });
    }
}
