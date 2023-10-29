use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;
use serde::{Deserialize, Serialize};
use text_splitter::TextSplitter;
use thiserror::Error;
use tokenizers::Tokenizer;

#[derive(Serialize, Deserialize, Debug, Error)]
pub enum ClassificationError {
    #[error("runetime error: {0}")]
    RuntimeError(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct Classification {
    pub label: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Message)]
#[rtype(result = "std::result::Result<Vec<Classification>, ClassificationError>")]
pub struct Classify {
    pub text: String,
    pub categories: Vec<String>,
}

pub struct ClassificationActor {
    model: Arc<Mutex<Option<ZeroShotClassificationModel>>>,
    tokenizer: Tokenizer,
}

impl ClassificationActor {
    pub fn new() -> Self {
        let model = Arc::new(Mutex::new(None));
        let model_clone = model.clone();

        std::thread::spawn(move || {
            let zero_shot_classification_model =
                ZeroShotClassificationModel::new(Default::default()).expect("failed to load model");
            let mut model = model_clone.lock().unwrap();
            *model = Some(zero_shot_classification_model);
        });

        let tokenizer = Tokenizer::from_pretrained("bert-base-cased", None).unwrap();

        ClassificationActor { model, tokenizer }
    }
}

impl Actor for ClassificationActor {
    type Context = Context<Self>;
}

impl Handler<Classify> for ClassificationActor {
    type Result = Result<Vec<Classification>, ClassificationError>;

    fn handle(&mut self, msg: Classify, _: &mut Context<Self>) -> Self::Result {
        let formatted_input: String = msg
            .text
            .trim()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        let categories = msg
            .categories
            .iter()
            .map(|category| category.as_str())
            .collect::<Vec<&str>>();
        let max_tokens: usize = 512;
        let splitter: TextSplitter<Tokenizer> =
            TextSplitter::new(self.tokenizer.clone()).with_trim_chunks(true);

        let current_text: String = formatted_input.clone();

        loop {
            let chunks: Vec<&str> = splitter.chunks(&current_text, max_tokens).collect();
            let mut classifications: HashMap<String, Classification> = HashMap::new();

            loop {
                let model: std::sync::MutexGuard<'_, Option<ZeroShotClassificationModel>> =
                    self.model.lock().unwrap();
                if let Some(model) = &*model {
                    for chunk in chunks {
                        let matches = &model
                            .predict_multilabel(&[chunk], categories.as_slice(), None, 1024)
                            .expect("failed to classify")[0];
                        let chunk_classifications: Vec<Classification> = matches
                            .into_iter()
                            .filter(|label| label.score > 0.75)
                            .map(|label| Classification {
                                label: label.text.clone(),
                                score: label.score,
                            })
                            .collect();
                        for classification in chunk_classifications {
                            if let Some(existing_classification) =
                                classifications.get(&classification.label)
                            {
                                if classification.score > existing_classification.score {
                                    classifications
                                        .insert(classification.label.clone(), classification);
                                }
                            } else {
                                classifications
                                    .insert(classification.label.clone(), classification);
                            }
                        }
                    }
                    break;
                }
            }

            let classifications: Vec<Classification> = classifications.into_values().collect();
            return Ok(classifications);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_actor() {
        System::new().block_on(async {
            let addr: Addr<ClassificationActor> = ClassificationActor::new().start();

            // Education Article: https://www.politico.com/news/2023/10/09/harvard-students-israel-hamas-attacks-00120624
            let classify = Classify {
                text: "
Some of Harvard University's most prominent political alumni are criticizing the school for not condemning a student-led statement that blamed Israel for the surprise Hamas attack over the weekend.

“The silence from Harvard's leadership, so far, coupled with a vocal and widely reported student groups' statement blaming Israel solely, has allowed Harvard to appear at best neutral towards acts of terror against the Jewish state of Israel,” Lawrence Summers, a former Harvard president and longtime Washington economic policy hand, wrote on X, the platform formerly known as Twitter. 

Summers, a Democrat who served as Treasury secretary under Bill Clinton, added: “I am sickened. I cannot fathom the Administration's failure to disassociate the University and condemn this statement.”

In their comments, prominent figures who studied at the university — many of them Republicans — blasted the school for not standing up for Israel. The story made the rounds on Sunday and Monday across a plethora of mostly conservative news sites, picking up the attention of Washington figures like Sen. Ted Cruz (R-Tex.) and Rep. Elise Stefanik (R-N.Y.)

“What the hell is wrong with Harvard?” Cruz, who attended Harvard Law School, wrote Monday on X.

Stefanik, the House Republican Conference Chair, wrote Sunday night on X: “It is abhorrent and heinous that Harvard student groups are blaming Israel for Hamas' barbaric terrorist attacks that have killed over 700 Israelis.”

Following much of the backlash, Harvard's leadership released a statement Monday night that did not directly address the student organizations but instead focused on the school's commitment to fostering open dialogue.

“We have no illusion that Harvard alone can readily bridge the widely different views of the Israeli-Palestinian conflict, but we are hopeful that, as a community devoted to learning, we can take steps that will draw on our common humanity and shared values in order to modulate rather than amplify the deep-seated divisions and animosities so distressingly evident in the wider world,” the school's leadership said.

The students originally wrote in a Saturday statement that the Hamas-led attack “did not occur in a vacuum” and that Israel was “entirely responsible for all unfolding violence.”

“In the coming days, Palestinians will be forced to bear the full brunt of Israel's violence,” the students wrote.

A review of the statement shows that most of the 35 student organizations signing the letter are identity-based groups or caucuses — and several of them, in name, expressly support the rights of Palestinian people. Activist student groups that support Palestinians are common across the country, and they often lead demonstrations and protests critical of Israel on campuses.

The development could represent an early challenge for Claudine Gay, who recently became Harvard's president this summer. The university has often been the target of conservative criticism that higher education panders to elites and teaches liberal viewpoints, and it was the main target of the Supreme Court case that toppled affirmative action in June.
                ".to_string(),
                categories: vec![
                    "Agriculture".to_string(), 
                    "Cannabis".to_string(), 
                    "Cybersecurity".to_string(), 
                    "Defense".to_string(), 
                    "Education".to_string(), 
                    "Energy".to_string(), 
                    "Environment".to_string(), 
                    "Finance".to_string(), 
                    "Tax".to_string(), 
                    "Health Care".to_string(), 
                    "Immigration".to_string(), 
                    "Labor".to_string(), 
                    "Sustainability".to_string(), 
                    "Technology".to_string(), 
                    "Trade".to_string(), 
                    "Transportation".to_string()],
            };

            let result = addr.send(classify).await.expect("msg failed to send").expect("msg failed to process");

            assert!(result.len() == 1);
            assert!(result[0].label == "Education".to_string());
        });
    }
}
