/// The resource must be able to describe itself
/// by some means.
/// When a server provides many resources (i.e. services),
/// this description is what will be used by the server to know
/// which resources match the request from the client.
pub trait Describable<D> where D: PartialEq {
    fn describe(&self) -> D;
}
