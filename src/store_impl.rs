use arrayref::array_ref;
use db_key::Key;
use osmpbfreader::NodeId;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct Long(pub i64);

impl Key for Long {
    fn from_u8(other: &[u8]) -> Self {
        Long::from(other)
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(self.0.to_be_bytes().as_slice())
    }
}

impl From<&[u8]> for Long {
    fn from(other: &[u8]) -> Self {
        let inner = i64::from_be_bytes(array_ref!(other, 0, 8).clone());
        Long(inner)
    }
}

impl From<NodeId> for Long {
    fn from(other: NodeId) -> Long {
        Long(other.0)
    }
}

impl From<Long> for NodeId {
    fn from(other: Long) -> NodeId {
        NodeId(other.0)
    }
}
