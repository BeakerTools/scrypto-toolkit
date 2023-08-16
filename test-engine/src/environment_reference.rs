pub trait EnvRef {
    fn format(&self) -> String;
}

impl<T> EnvRef for T
where
    T: ToString,
{
    fn format(&self) -> String {
        self.to_string().to_lowercase().replace("_", " ")
    }
}
