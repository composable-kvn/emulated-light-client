use super::path::SequencePath;
use super::{ibc, ids};


/// A key used for indexing entries in the provable storage.
///
/// The key is built from IBC storage paths.  The first byte is discriminant
/// determining the type of path and the rest are concatenated components
/// encoded in binary.  The key format can be visualised as the following enum:
///
/// ```ignore
/// enum TrieKey {
///     ClientState      { client_id: u32 },
///     ConsensusState   { client_id: u32, epoch: u64, height: u64 },
///     Connection       { connection_id: u32 },
///     ChannelEnd       { port_id: [u8; 9], channel_id: u32 },
///     NextSequenceSend { port_id: [u8; 9], channel_id: u32 },
///     NextSequenceRecv { port_id: [u8; 9], channel_id: u32 },
///     NextSequenceAck  { port_id: [u8; 9], channel_id: u32 },
///     Commitment       { port_id: [u8; 9], channel_id: u32, sequence: u64 },
///     Receipts         { port_id: [u8; 9], channel_id: u32, sequence: u64 },
///     Acks             { port_id: [u8; 9], channel_id: u32, sequence: u64 },
/// }
/// ```
///
/// Integers are encoded using big-endian to guarantee dense encoding of
/// consecutive keys (i.e. sequence 10 is immediately followed by 11 which would
/// not be the case in little-endian encoding).  This is also one reason why we
/// don’t just use Borsh encoding.
pub struct TrieKey {
    // tag (1) + port_id (9) + channel_id (4) + sequence (8) = max 22 bytes
    bytes: [u8; 22],
    len: u8,
}


impl TrieKey {
    /// Constructs a new key for a client state path for client with given
    /// counter.
    ///
    /// The hash stored under the key is `hash(borsh((client_id.as_str(),
    /// client_state)))`.
    pub fn for_client_state(client: ids::ClientIdx) -> Self {
        Self::new(Tag::ClientState, u32::from(client))
    }

    /// Constructs a new key for a consensus state path for client with given
    /// counter and specified height.
    ///
    /// The hash stored under the key is `hash(borsh(consensus_state))`.
    pub fn for_consensus_state(
        client: ids::ClientIdx,
        height: ibc::Height,
    ) -> Self {
        Self::new(Tag::ConsensusState, (u32::from(client), height))
    }

    /// Constructs a new key for a connection end path.
    ///
    /// The hash stored under the key is `hash(borsh(connection_end))`.
    pub fn for_connection(connection: ids::ConnectionIdx) -> Self {
        Self::new(Tag::Connection, u32::from(connection))
    }

    /// Constructs a new key for a channel end path.
    ///
    /// The hash stored under the key is `hash(borsh(channel_end))`.
    pub fn for_channel_end(port_channel: &ids::PortChannelPK) -> Self {
        Self::for_channel_path(Tag::ChannelEnd, port_channel)
    }

    /// Constructs a new key for next sequence counters.
    ///
    /// The hash stored under the key is built by `SequenceTriple::hash` method
    /// and directly encodes next send, receive and ack sequence numbers.
    pub fn for_next_sequence(port_channel: &ids::PortChannelPK) -> Self {
        Self::for_channel_path(Tag::NextSequence, port_channel)
    }

    /// Constructs a new key for a `(port_id, channel_id)` path.
    ///
    /// This is internal method used by other public-facing methods which use
    /// only (port, channel) tuple as the key component.
    fn for_channel_path(tag: Tag, port_channel: &ids::PortChannelPK) -> Self {
        Self::new(tag, port_channel)
    }

    /// Constructs a new key for a `(port_id, channel_id, sequence)` path.
    ///
    /// Returns an error if `port_id` or `channel_id` is invalid.
    ///
    /// This is internal method used by other public-facing interfaces which use
    /// only (port, channel, sequence) tuple as the key component.
    fn try_for_sequence_path(
        tag: Tag,
        port_id: &ibc::PortId,
        channel_id: &ibc::ChannelId,
        sequence: ibc::Sequence,
    ) -> Result<Self, ibc::ChannelError> {
        let port_channel = ids::PortChannelPK::try_from(port_id, channel_id)?;
        Ok(Self::new(tag, (port_channel, u64::from(sequence))))
    }

    /// Constructs a new key with given tag and key component.
    ///
    /// For keys consisting of a multiple components, a tuple component can be
    /// used.
    fn new(tag: Tag, component: impl AsComponent) -> Self {
        let mut key = TrieKey { bytes: [0; 22], len: 1 };
        key.bytes[0] = tag.into();
        component.append_into(&mut key);
        key
    }

    /// Internal function to append bytes into the internal buffer.
    fn extend(&mut self, bytes: &[u8]) {
        let start = usize::from(self.len);
        let end = start + bytes.len();
        self.bytes[start..end].copy_from_slice(bytes);
        self.len = end as u8;
    }
}

impl core::ops::Deref for TrieKey {
    type Target = [u8];
    fn deref(&self) -> &[u8] { &self.bytes[..usize::from(self.len)] }
}

impl TryFrom<SequencePath<'_>> for TrieKey {
    type Error = ibc::ChannelError;
    fn try_from(path: SequencePath<'_>) -> Result<Self, Self::Error> {
        let port_channel =
            ids::PortChannelPK::try_from(path.port_id, path.channel_id)?;
        Ok(Self::for_channel_path(Tag::NextSequence, &port_channel))
    }
}

impl TryFrom<&ibc::path::CommitmentPath> for TrieKey {
    type Error = ibc::ChannelError;
    fn try_from(path: &ibc::path::CommitmentPath) -> Result<Self, Self::Error> {
        Self::try_for_sequence_path(
            Tag::Commitment,
            &path.port_id,
            &path.channel_id,
            path.sequence,
        )
    }
}

impl TryFrom<&ibc::path::ReceiptPath> for TrieKey {
    type Error = ibc::ChannelError;
    fn try_from(path: &ibc::path::ReceiptPath) -> Result<Self, Self::Error> {
        Self::try_for_sequence_path(
            Tag::Receipt,
            &path.port_id,
            &path.channel_id,
            path.sequence,
        )
    }
}

impl TryFrom<&ibc::path::AckPath> for TrieKey {
    type Error = ibc::ChannelError;
    fn try_from(path: &ibc::path::AckPath) -> Result<Self, Self::Error> {
        Self::try_for_sequence_path(
            Tag::Ack,
            &path.port_id,
            &path.channel_id,
            path.sequence,
        )
    }
}

/// A discriminant used as the first byte of each trie key to create namespaces
/// for different objects stored in the trie.
#[repr(u8)]
enum Tag {
    ClientState = 0,
    ConsensusState = 1,
    Connection = 2,
    ChannelEnd = 3,
    NextSequence = 4,
    Commitment = 5,
    Receipt = 6,
    Ack = 8,
}

impl From<Tag> for u8 {
    fn from(tag: Tag) -> u8 { tag as u8 }
}

/// Component of a [`TrieKey`].
///
/// A `TrieKey` is constructed by concatenating a sequence of components.
trait AsComponent {
    /// Appends the component into the trie key.
    fn append_into(&self, dest: &mut TrieKey);
}

impl AsComponent for ids::PortChannelPK {
    fn append_into(&self, dest: &mut TrieKey) {
        self.port_key.as_bytes().append_into(dest);
        u32::from(self.channel_idx).append_into(dest);
    }
}

impl AsComponent for ibc::Height {
    fn append_into(&self, dest: &mut TrieKey) {
        (self.revision_number(), self.revision_height()).append_into(dest);
    }
}

impl AsComponent for u32 {
    fn append_into(&self, dest: &mut TrieKey) {
        self.to_be_bytes().append_into(dest)
    }
}

impl AsComponent for u64 {
    fn append_into(&self, dest: &mut TrieKey) {
        self.to_be_bytes().append_into(dest)
    }
}

impl<const N: usize> AsComponent for [u8; N] {
    fn append_into(&self, dest: &mut TrieKey) { dest.extend(self); }
}

impl<T: AsComponent> AsComponent for &T {
    fn append_into(&self, dest: &mut TrieKey) { (*self).append_into(dest) }
}

impl<T: AsComponent, U: AsComponent> AsComponent for (T, U) {
    fn append_into(&self, dest: &mut TrieKey) {
        self.0.append_into(dest);
        self.1.append_into(dest);
    }
}