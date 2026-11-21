pub trait UseCase {
    type Request;
    type Response;
    type Error;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;
}
