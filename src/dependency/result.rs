pub trait AlgorithmResult {
    fn display(&self);
    fn save_as_file(&self) -> Result<(), std::io::Error>;
}
