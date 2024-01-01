
use crate::web::CurrentQueryParameters;

pub enum ScenePreset {
    StanfordBunny,
    ChineseDragon,
}

impl Default for ScenePreset {
    fn default() -> Self {
        let query_parameters = CurrentQueryParameters::value();
        match query_parameters.as_deref() {
            Some("2024") => Self::ChineseDragon,
            Some("2023") => Self::StanfordBunny,
            _ => Self::StanfordBunny,
        }
    }
}

impl ScenePreset {
    pub fn model_name(&self) -> String {
        match self {
            Self::StanfordBunny => "stanford_bunny.glb".into(),
            Self::ChineseDragon => "dragon.glb".into(),
        }
    }
}
