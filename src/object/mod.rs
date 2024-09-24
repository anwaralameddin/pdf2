pub(crate) mod direct;
pub(crate) mod indirect;
pub(crate) mod r#type;

trait BorrowedBuffer {
    type OwnedBuffer;

    fn to_owned_buffer(self) -> Self::OwnedBuffer;
}
