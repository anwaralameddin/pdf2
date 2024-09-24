pub(crate) mod direct;
pub(crate) mod indirect;
pub(crate) mod r#type;

pub(crate) trait BorrowedBuffer {
    type OwnedBuffer;

    fn to_owned_buffer(self) -> Self::OwnedBuffer;
}
