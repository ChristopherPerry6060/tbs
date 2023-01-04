pub mod entry;
pub mod plan;

mod result {
    use thiserror::Error;
    #[derive(Debug, Error)]
    pub enum ErrorKind {
        #[error("Row is missing an Id")]
        MissingId,
        #[error("Row is missing an Fnsku")]
        MissingFnsku,
        #[error("Row is missing the PackType")]
        MissingPackType,
        #[error("Row is missing the unit quantity")]
        MissingUnits,
        #[error("Row is declared as packed with dimensions missing")]
        MissingPackedDimensions,
        #[error("Row is declared as packed with weight missing")]
        MissingPackedWeight,
        #[error(
            "Row is declared as packed with Units that are not evenly 
        divisible by the CaseQt"
        )]
        NonDivisibleCaseQt,
        #[error("Row is declared as packed with CaseQt missing")]
        MissingCaseQt,
        #[error("A PackType is included, but cannot be recognized")]
        InvalidPackType,
        #[error("Row is declared as Loose with StagingGroup missing")]
        MissingGroup,
        #[error("Row is declared as Loose with UnitWeight missing")]
        MissingUnitWeight,
        #[error("Unable to deserialized StringRecord")]
        CsvError,
    }
    pub type Result<T> = std::result::Result<T, ErrorKind>;

    impl From<csv::Error> for ErrorKind {
        fn from(_value: csv::Error) -> Self {
            Self::CsvError
        }
    }
}
