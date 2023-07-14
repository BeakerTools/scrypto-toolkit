pub trait ToFormatted {
    fn format(&self) -> String;
}

impl<T> ToFormatted for T
    where
        T: ToString,
{
    fn format(&self) -> String {
        self.to_string().to_lowercase().replace("_", " ")
    }
}