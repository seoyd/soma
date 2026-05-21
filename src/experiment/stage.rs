use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StageStatus {
    Pending,
    Passed,
    Failed,
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExperimentStage {
    LoadData,
    ValidateData,
    Resample,
    BuildFeatures,
    BuildDataset,
    BaselineEvaluate,
    PythonValidateDataset,
    PythonTrain,
    ImportPredictions,
    ExternalEvaluate,
    CompareModels,
    WriteReportBundle,
}

impl ExperimentStage {
    pub fn all() -> [Self; 12] {
        [
            Self::LoadData,
            Self::ValidateData,
            Self::Resample,
            Self::BuildFeatures,
            Self::BuildDataset,
            Self::BaselineEvaluate,
            Self::PythonValidateDataset,
            Self::PythonTrain,
            Self::ImportPredictions,
            Self::ExternalEvaluate,
            Self::CompareModels,
            Self::WriteReportBundle,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::LoadData => "LoadData",
            Self::ValidateData => "ValidateData",
            Self::Resample => "Resample",
            Self::BuildFeatures => "BuildFeatures",
            Self::BuildDataset => "BuildDataset",
            Self::BaselineEvaluate => "BaselineEvaluate",
            Self::PythonValidateDataset => "PythonValidateDataset",
            Self::PythonTrain => "PythonTrain",
            Self::ImportPredictions => "ImportPredictions",
            Self::ExternalEvaluate => "ExternalEvaluate",
            Self::CompareModels => "CompareModels",
            Self::WriteReportBundle => "WriteReportBundle",
        }
    }
}
