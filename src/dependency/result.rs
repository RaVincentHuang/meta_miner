pub trait AlgorithmResult {
    fn dispaly(&self);
    fn save_as_file(&self) -> Result<(), std::io::Error>;
}
