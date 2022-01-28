use super::super::*;

use std::slice::from_raw_parts;

/// Module containing ICMPv6 related types and constants
pub mod icmpv6 {

    /// ICMPv6 type value indicating a "Destination Unreachable" message.
    pub const TYPE_DST_UNREACH: u8 = 1;

    /// ICMPv6 type value indicating a "Packet Too Big" message.
    pub const TYPE_PACKET_TOO_BIG: u8 = 2;

    /// ICMPv6 type value indicating a "Time Exceeded" message.
    pub const TYPE_TIME_EXCEEDED: u8 = 3;

    /// ICMPv6 type value indicating a "Parameter Problem" message.
    pub const TYPE_PARAM_PROB: u8 = 4;

    /// ICMPv6 type value indicating an "Echo Request" message.
    pub const TYPE_ECHO_REQUEST: u8 = 128;

    /// ICMPv6 type value indicating an "Echo Reply" message.
    pub const TYPE_ECHO_REPLY: u8 = 129;

    /// ICMPv6 type value indicating a "Multicast Listener Query" message.
    pub const TYPE_MULTICAST_LISTENER_QUERY: u8 = 130;

    /// ICMPv6 type value indicating a "Multicast Listener Report" message.
    pub const TYPE_MULTICAST_LISTENER_REPORT: u8 = 131;

    /// ICMPv6 type value indicating a "Multicast Listener Done" message.
    pub const TYPE_MULTICAST_LISTENER_REDUCTION: u8 = 132;

    /// ICMPv6 type value indicating a "Router Solicitation" message.
    pub const TYPE_ROUTER_SOLICITATION: u8 = 133;

    /// ICMPv6 type value indicating a "Router Advertisement" message.
    pub const TYPE_ROUTER_ADVERTISEMENT: u8 = 134;

    /// ICMPv6 type value indicating a "Neighbor Solicitation" message.
    pub const TYPE_NEIGHBOR_SOLICITATION: u8 = 135;

    /// ICMPv6 type value indicating a "Neighbor Advertisement" message.
    pub const TYPE_NEIGHBOR_ADVERTISEMENT: u8 = 136;

    /// ICMPv6 type value indicating a "Redirect Message" message.
    pub const TYPE_REDIRECT_MESSAGE: u8 = 137;

    /// ICMPv6 type value indicating a "Router Renumbering" message.
    pub const TYPE_ROUTER_RENUMBERING: u8 = 138;

    /// ICMPv6 type value indicating a "Inverse Neighbor Discovery Solicitation" message.
    pub const TYPE_INVERSE_NEIGHBOR_DISCOVERY_SOLICITATION: u8 = 141;

    /// ICMPv6 type value indicating a "Inverse Neighbor Discovery Advertisement" message.
    pub const TYPE_INVERSE_NEIGHBOR_DISCOVERY_ADVERTISEMENT: u8 = 142;

    /// ICMPv6 type value indicating a "Extended Echo Request" message.
    pub const TYPE_EXT_ECHO_REQUEST: u8 = 160;

    /// ICMPv6 type value indicating a "Extended Echo Reply" message.
    pub const TYPE_EXT_ECHO_REPLY: u8 = 161;

    /// ICMPv6 destination unreachable code for "no route to destination".
    pub const CODE_DST_UNREACH_NOROUTE: u8 = 0;

    /// ICMPv6 destination unreachable code for "communication with
    /// destination administratively prohibited".
    pub const CODE_DST_UNREACH_PROHIBITED: u8 = 1;

    /// ICMPv6 destination unreachable code for "beyond scope of source address".
    pub const CODE_DST_UNREACH_BEYONDSCOPE: u8 = 2;

    /// ICMPv6 destination unreachable code for "address unreachable".
    pub const CODE_DST_UNREACH_ADDR: u8 = 3;

    /// ICMPv6 destination unreachable code for "port unreachable".
    pub const CODE_DST_UNREACH_PORT: u8 = 4;

    /// ICMPv6 destination unreachable code for "source address failed ingress/egress policy".
    pub const CODE_DST_UNREACH_SOURCE_ADDRESS_FAILED_POLICY: u8 = 5;

    /// ICMPv6 destination unreachable code for "reject route to destination".
    pub const CODE_DST_UNREACH_REJECT_ROUTE_TO_DEST: u8 = 6;

    /// ICMPv6 time exceeded code for "hop limit exceeded in transit"
    pub const CODE_TIME_EXCEEDED_HOP_LIMIT_EXCEEDED: u8 = 0;

    /// ICMPv6 time exceeded code for "fragment reassembly time exceeded"
    pub const CODE_TIME_EXCEEDED_FRAGMENT_REASSEMBLY_TIME_EXCEEDED: u8 = 1;

    /// "Destination Unreachable" ICMPv6 header (without the invoking packet).
    ///
    /// # RFC 4443 Description:
    ///
    /// A Destination Unreachable message SHOULD be generated by a router, or
    /// by the IPv6 layer in the originating node, in response to a packet
    /// that cannot be delivered to its destination address for reasons other
    /// than congestion.  (An ICMPv6 message MUST NOT be generated if a
    /// packet is dropped due to congestion.)
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DestUnreachableHeader {
        /// In case of an unknown icmp code is received the header elements are stored raw.
        Raw{
            /// ICMP code (present in the 2nd byte of the ICMP packet).
            code: u8,
            /// Bytes located at th 5th, 6th, 7th and 8th position of the ICMP packet.
            bytes5to8: [u8;4],
        },
        /// No route to destination
        NoRoute,
        /// Communication with destination administratively prohibited
        Prohibited,
        /// Beyond scope of source address
        BeyondScope,
        /// Address unreachable
        Address,
        /// Port unreachable
        Port,
        /// Source address failed ingress/egress policy
        SourceAddressFailedPolicy,
        /// Reject route to destination
        RejectRoute,
    }

    impl DestUnreachableHeader {
        /// Converts the raw values from an ICMPv6 "destination unreachable"
        /// packet to an `icmpv6::DestUnreachableHeader` enum.
        ///
        /// `from_bytes` expects the second byte as first argument and 5th-8th
        /// bytes as second argument.
        ///
        /// # Example Usage:
        ///
        /// ```
        /// use etherparse::{icmpv6, icmpv6::DestUnreachableHeader};
        /// let icmp_packet: [u8;8] = [
        ///     icmpv6::TYPE_DST_UNREACH, icmpv6::CODE_DST_UNREACH_PORT, 0, 0,
        ///     0, 0, 0, 0,
        /// ];
        ///
        /// if icmpv6::TYPE_DST_UNREACH == icmp_packet[0] {
        ///     let dst = icmpv6::DestUnreachableHeader::from_bytes(
        ///         icmp_packet[1],
        ///         [icmp_packet[4], icmp_packet[5], icmp_packet[6], icmp_packet[7]],
        ///     );
        ///     assert_eq!(dst, icmpv6::DestUnreachableHeader::Port);
        /// }
        /// ```
        pub fn from_bytes(code: u8, bytes5to8: [u8;4]) -> DestUnreachableHeader {
            use DestUnreachableHeader::*;
            match code {
                CODE_DST_UNREACH_NOROUTE => NoRoute,
                CODE_DST_UNREACH_PROHIBITED => Prohibited,
                CODE_DST_UNREACH_BEYONDSCOPE => BeyondScope,
                CODE_DST_UNREACH_ADDR => Address,
                CODE_DST_UNREACH_PORT => Port,
                CODE_DST_UNREACH_SOURCE_ADDRESS_FAILED_POLICY => SourceAddressFailedPolicy,
                CODE_DST_UNREACH_REJECT_ROUTE_TO_DEST => RejectRoute,
                _ => Raw{code, bytes5to8},
            }
        }

        /// Returns the code value of the destination unreachable packet.
        ///
        /// This is the second byte of an ICMPv6 packet.
        pub fn code(&self) -> u8 {
            use DestUnreachableHeader::*;
            match self {
                Raw{code, bytes5to8: _} => *code,
                NoRoute => CODE_DST_UNREACH_NOROUTE,
                Prohibited => CODE_DST_UNREACH_PROHIBITED,
                BeyondScope => CODE_DST_UNREACH_BEYONDSCOPE,
                Address => CODE_DST_UNREACH_ADDR,
                Port => CODE_DST_UNREACH_PORT,
                SourceAddressFailedPolicy => CODE_DST_UNREACH_SOURCE_ADDRESS_FAILED_POLICY,
                RejectRoute => CODE_DST_UNREACH_REJECT_ROUTE_TO_DEST,
            }
        }

        /// Returns second and and 5th-8th bytes (inclusive) of
        /// the destination unreachable ICMPv6 packet.
        pub fn to_bytes(&self) -> (u8, [u8;4]) {
            use DestUnreachableHeader::*;
            match self {
                Raw{ code, bytes5to8 } => (*code, *bytes5to8),
                NoRoute => (CODE_DST_UNREACH_NOROUTE, [0;4]),
                Prohibited => (CODE_DST_UNREACH_PROHIBITED, [0;4]),
                BeyondScope => (CODE_DST_UNREACH_BEYONDSCOPE, [0;4]),
                Address => (CODE_DST_UNREACH_ADDR, [0;4]),
                Port => (CODE_DST_UNREACH_PORT, [0;4]),
                SourceAddressFailedPolicy => (CODE_DST_UNREACH_SOURCE_ADDRESS_FAILED_POLICY, [0;4]),
                RejectRoute => (CODE_DST_UNREACH_REJECT_ROUTE_TO_DEST, [0;4]),
            }
        }
    }

    /// Code values for ICMPv6 time exceeded message.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TimeExceededCode {
        /// In case of an unknown icmp code is received the header elements are stored raw.
        Raw{ code: u8 },
        /// "hop limit exceeded in transit"
        HopLimitExceeded,
        /// "fragment reassembly time exceeded"
        FragmentReassemblyTimeExceeded,
    }

    impl From<u8> for TimeExceededCode {
        fn from(code: u8) -> TimeExceededCode {
            use TimeExceededCode::*;
            match code {
                CODE_TIME_EXCEEDED_HOP_LIMIT_EXCEEDED => HopLimitExceeded,
                CODE_TIME_EXCEEDED_FRAGMENT_REASSEMBLY_TIME_EXCEEDED => FragmentReassemblyTimeExceeded,
                code => Raw { code },
            }
        }
    }

    impl From<TimeExceededCode> for u8 {
        fn from(code: TimeExceededCode) -> u8 {
            use TimeExceededCode::*;
            match code {
                Raw{ code } => code,
                HopLimitExceeded => CODE_TIME_EXCEEDED_HOP_LIMIT_EXCEEDED,
                FragmentReassemblyTimeExceeded => CODE_TIME_EXCEEDED_FRAGMENT_REASSEMBLY_TIME_EXCEEDED,
            }
        }
    }

    /// Code values for ICMPv6 parameter problem messages.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ParameterProblemCode {
        /// In case of an unknown icmp code is received the header elements are stored raw.
        Raw{ code: u8 },

    }

    impl From<u8> for ParameterProblemCode {
        fn from(code: u8) -> ParameterProblemCode {
            use ParameterProblemCode::*;
            match code {
                code => Raw { code },
            }
        }
    }

    impl From<ParameterProblemCode> for u8 {
        fn from(code: ParameterProblemCode) -> u8 {
            use ParameterProblemCode::*;
            match code {
                Raw{ code } => code,
            }
        }
    }

} // mod icmpv6

use icmpv6::*;

/// Different kinds of ICMPv6 messages.
///
/// The data stored in this enum corresponds to the statically sized data
/// at the start of an ICMPv6 packet without the checksum. If you also need
/// the checksum you can package and [`Icmp6Type`] value in an [`Icmpv6Header`]
/// struct.
///
/// # Decoding Example (complete packet):
///
/// ```
/// # use etherparse::{PacketBuilder};
/// # let mut builder = PacketBuilder::
/// #   ethernet2([0;6], [0;6])
/// #   .ipv6([0;16], [0;16], 20)
/// #   .icmp6_echo_request(1, 2);
/// # let payload = [1,2,3,4];
/// # let mut packet = Vec::<u8>::with_capacity(builder.size(payload.len()));
/// # builder.write(&mut packet, &payload);
/// use etherparse::PacketHeaders;
///
/// let headers = PacketHeaders::from_ethernet_slice(&packet).unwrap();
///
/// use etherparse::TransportHeader::*;
/// match headers.transport {
///     Some(Icmp6(icmp)) => {
///         use etherparse::Icmp6Type::*;
///         match icmp.icmp_type {
///             // Raw is used when further decoding is currently not supported for the icmp type & code.
///             // You can still further decode the packet on your own by using the raw data in this enum
///             // together with `headers.payload` (contains the packet data after the 8th byte)
///             Raw{ icmp_type, icmp_code, bytes5to8 } => println!("Raw{{ icmp_type: {}, icmp_code: {}, bytes5to8: {:?} }}", icmp_type, icmp_code, bytes5to8),
///             DestinationUnreachable(header) => println!("{:?}", header),
///             PacketTooBig { mtu } => println!("TimeExceeded{{ mtu: {} }}", mtu),
///             TimeExceeded{ code } => println!("TimeExceeded{{ code: {:?} }}", code),
///             ParameterProblem{ code, pointer } => println!("ParameterProblem{{ code: {:?}, pointer: {} }}", code, pointer),
///             EchoRequest(header) => println!("{:?}", header),
///             EchoReply(header) => println!("{:?}", header),
///         }
///     },
///     _ => {},
/// }
/// ```
///
/// # Encoding Example (only ICMPv6 part)
///
/// To get the on wire bytes of an Icmp6Type it needs to get packaged
/// into a [`Icmpv6Header`] so the checksum gets calculated.
///
/// ```
/// # use etherparse::Ipv6Header;
/// # let ip_header: Ipv6Header = Default::default();
/// # let invoking_packet : [u8;0] = [];
///
/// use etherparse::{Icmp6Type, icmpv6::DestUnreachableHeader};
/// let t = Icmp6Type::DestinationUnreachable(
///     DestUnreachableHeader::Address
/// );
///
/// // to calculate the checksum the ip header and the payload
/// // (in case of dest unreachable the invoking packet) are needed
/// let header = t.to_header(&ip_header, &invoking_packet).unwrap();
/// 
/// // an ICMPv6 packet is composed of the header and payload
/// let mut packet = Vec::with_capacity(header.header_len() + invoking_packet.len());
/// packet.extend_from_slice(&header.to_bytes());
/// packet.extend_from_slice(&invoking_packet);
/// #
/// # {
/// #   let checksum_be = header.checksum.to_be_bytes();
/// #   assert_eq!(
/// #       &packet,
/// #       &[
/// #           header.icmp_type.type_value(),
/// #           header.icmp_type.code_value(),
/// #           checksum_be[0],
/// #           checksum_be[1],
/// #           0,0,0,0
/// #       ]
/// #   );
/// # }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icmp6Type {
    /// In case of an unknown icmp type is received the header elements of
    /// the first 8 bytes/octets are stored raw.
    Raw{
        icmp_type: u8,
        icmp_code: u8,
        /// Bytes located at th 5th, 6th, 7th and 8th position of the ICMP packet.
        bytes5to8: [u8;4]
    },
    /// Start of "Destination Unreachable Message".
    ///
    /// # RFC 4443 Description
    ///
    /// A Destination Unreachable message SHOULD be generated by a router, or
    /// by the IPv6 layer in the originating node, in response to a packet
    /// that cannot be delivered to its destination address for reasons other
    /// than congestion.  (An ICMPv6 message MUST NOT be generated if a
    /// packet is dropped due to congestion.)
    DestinationUnreachable(icmpv6::DestUnreachableHeader),
    /// Start of "Packet Too Big Message"
    ///
    /// # RFC 4443 Description
    ///
    /// A Packet Too Big MUST be sent by a router in response to a packet
    /// that it cannot forward because the packet is larger than the MTU of
    /// the outgoing link.  The information in this message is used as part
    /// of the Path MTU Discovery process.
    PacketTooBig {
        /// The Maximum Transmission Unit of the next-hop link.
        mtu: u32
    },
    /// Start of "Time Exceeded Message"
    ///
    /// # RFC 4443 Description
    ///
    /// If a router receives a packet with a Hop Limit of zero, or if a
    /// router decrements a packet's Hop Limit to zero, it MUST discard the
    /// packet and originate an ICMPv6 Time Exceeded message with Code 0 to
    /// the source of the packet.  This indicates either a routing loop or
    /// too small an initial Hop Limit value.
    ///
    /// An ICMPv6 Time Exceeded message with Code 1 is used to report
    /// fragment reassembly timeout, as specified in [IPv6, Section 4.5].
    TimeExceeded {
        /// Code identifying which time as exceeded.
        code: icmpv6::TimeExceededCode,
    },
    /// Start of "Parameter Problem Message"
    ///
    /// # RFC 4443 Description
    ///
    /// If an IPv6 node processing a packet finds a problem with a field in
    /// the IPv6 header or extension headers such that it cannot complete
    /// processing the packet, it MUST discard the packet and SHOULD
    /// originate an ICMPv6 Parameter Problem message to the packet's source,
    /// indicating the type and location of the problem.
    ParameterProblem {
        /// The code can offer additional informations about what kind of parameter
        /// problem caused the error.
        code: icmpv6::ParameterProblemCode,
        /// Identifies the octet offset within the
        /// invoking packet where the error was detected.
        ///
        /// The pointer will point beyond the end of the ICMPv6
        /// packet if the field in error is beyond what can fit
        /// in the maximum size of an ICMPv6 error message.
        pointer: u32,
    },
    /// Start of "Echo Request Message"
    ///
    /// # RFC 4443 Description
    ///
    /// Every node MUST implement an ICMPv6 Echo responder function that
    /// receives Echo Requests and originates corresponding Echo Replies.  A
    /// node SHOULD also implement an application-layer interface for
    /// originating Echo Requests and receiving Echo Replies, for diagnostic
    /// purposes.
    EchoRequest(IcmpEchoHeader),
    /// Start of "Echo Reply Message"
    ///
    /// # RFC 4443 Description
    ///
    /// Every node MUST implement an ICMPv6 Echo responder function that
    /// receives Echo Requests and originates corresponding Echo Replies. A
    /// node SHOULD also implement an application-layer interface for
    /// originating Echo Requests and receiving Echo Replies, for diagnostic
    /// purposes.
    ///
    /// The source address of an Echo Reply sent in response to a unicast
    /// Echo Request message MUST be the same as the destination address of
    /// that Echo Request message.
    ///
    /// An Echo Reply SHOULD be sent in response to an Echo Request message
    /// sent to an IPv6 multicast or anycast address.  In this case, the
    /// source address of the reply MUST be a unicast address belonging to
    /// the interface on which the Echo Request message was received.
    ///
    /// The data received in the ICMPv6 Echo Request message MUST be returned
    /// entirely and unmodified in the ICMPv6 Echo Reply message.
    EchoReply(IcmpEchoHeader),
}

impl Icmp6Type {
    /// Decode the enum from the icmp type, code and bytes5to8 bytes (5th till and
    /// including 8th byte of the the ICMPv6 header).
    fn from_bytes(icmp_type: u8, icmp_code: u8, bytes5to8: [u8;4]) -> Icmp6Type {
        use Icmp6Type::*;
        match icmp_type {
            TYPE_DST_UNREACH => 
                DestinationUnreachable(icmpv6::DestUnreachableHeader::from_bytes(icmp_code, bytes5to8)),
            TYPE_PACKET_TOO_BIG => PacketTooBig {
                mtu: u32::from_be_bytes(bytes5to8),
            },
            TYPE_TIME_EXCEEDED => TimeExceeded{
                code: icmp_code.into()
            },
            TYPE_PARAM_PROB => ParameterProblem{
                code: icmp_code.into(),
                pointer: u32::from_be_bytes(bytes5to8),
            },
            TYPE_ECHO_REQUEST => EchoRequest(IcmpEchoHeader::from_bytes(bytes5to8)),
            TYPE_ECHO_REPLY => EchoReply(IcmpEchoHeader::from_bytes(bytes5to8)),
            _ => Raw{icmp_type, icmp_code, bytes5to8},
        }
    }

    /// Returns the type value (first byte of the ICMPv6 header) of this type.
    #[inline]
    pub fn type_value(&self) -> u8 {
        use Icmp6Type::*;
        match self {
            Raw{icmp_type, icmp_code: _, bytes5to8: _} => *icmp_type,
            DestinationUnreachable(_) => TYPE_DST_UNREACH,
            PacketTooBig{ mtu: _ } => TYPE_PACKET_TOO_BIG,
            TimeExceeded{ code: _ } => TYPE_TIME_EXCEEDED,
            ParameterProblem{ code: _, pointer: _ } => TYPE_PARAM_PROB,
            EchoRequest(_) => TYPE_ECHO_REQUEST,
            EchoReply(_) => TYPE_ECHO_REPLY,
        }
    }

    /// Returns the code value (second byte of the ICMPv6 header) of this type.
    #[inline]
    pub fn code_value(&self) -> u8 {
        use Icmp6Type::*;
        match self {
            Raw{icmp_type: _, icmp_code, bytes5to8: _} => *icmp_code,
            DestinationUnreachable(icmp_code) => icmp_code.code(),
            PacketTooBig{ mtu: _ } => 0,
            TimeExceeded{ code } => u8::from(*code),
            ParameterProblem{ code, pointer: _ } => u8::from(*code),
            EchoRequest(_) => 0,
            EchoReply(_) => 0,
        }
    }

    /// Calculates the checksum of the ICMPv6 header.
    pub fn calc_checksum(&self, ip_header: &Ipv6Header, payload: &[u8]) -> Result<u16, ValueError> {
        //check that the total length fits into the field
        let max_payload_len: usize = (std::u32::MAX as usize) - self.header_len();
        if max_payload_len < payload.len() {
            return Err(ValueError::Ipv6PayloadLengthTooLarge(payload.len()));
        }

        let (icmp_type, icmp_code, bytes5to8) = self.to_bytes();
        let msg_len = payload.len() + self.header_len();
        //calculate the checksum; icmp4 will always take an ip4 header
        Ok(
            // NOTE: rfc4443 section 2.3 - Icmp6 *does* use a pseudoheader, 
            // unlike Icmp4
            checksum::Sum16BitWords::new()
            .add_16bytes(ip_header.source)
            .add_16bytes(ip_header.destination)
            .add_2bytes([0, ip_number::IPV6_ICMP])
            .add_2bytes((msg_len as u16).to_be_bytes())
            .add_2bytes([icmp_type, icmp_code])
            .add_4bytes(bytes5to8)
            .add_slice(payload)
            .ones_complement()
            .to_be()
        )
    }

    /// Encode the enum to the on wire format.
    ///
    /// It returns the icmp type, code and bytes5to8 bytes (5th till and
    /// including 8th byte of the the ICMPv6 header).
    fn to_bytes(&self) -> (u8, u8, [u8;4]) {
        use Icmp6Type::*;
        match self {
            Raw{icmp_type, icmp_code, bytes5to8} => (*icmp_type, *icmp_code, *bytes5to8),
            DestinationUnreachable(icmp_code) => 
            (TYPE_DST_UNREACH, (icmp_code.code()), [0;4]),
            PacketTooBig{ mtu } => (TYPE_PACKET_TOO_BIG, 0, mtu.to_be_bytes()),
            TimeExceeded{ code } => (TYPE_TIME_EXCEEDED, u8::from(*code), [0;4]),
            ParameterProblem{ code, pointer } => (TYPE_PARAM_PROB, u8::from(*code), pointer.to_be_bytes()),
            EchoRequest(echo) => (TYPE_ECHO_REQUEST, 0, echo.to_bytes()),
            EchoReply(echo) => (TYPE_ECHO_REPLY, 0, echo.to_bytes()),
        }
    }

    /// Creates a header with the correct checksum.
    pub fn to_header(self, ip_header: &Ipv6Header, payload: &[u8]) -> Result<Icmpv6Header, ValueError> {
        Ok(Icmpv6Header {
            checksum: self.calc_checksum(ip_header, payload)?,
            icmp_type: self,
        })
    }

    /// Serialized length of the header in bytes/octets.
    ///
    /// Note that this size is not the size of the entire
    /// ICMPv6 packet but only the header.
    pub fn header_len(&self) -> usize {
        8
    }
}

/// The statically sized data at the start of an ICMPv6 packet (at least the first 8 bytes of an ICMPv6 packet).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Icmpv6Header {
    pub icmp_type: Icmp6Type,
    /// Checksum in the ICMPv6 header.
    pub checksum: u16,
}

impl Icmpv6Header {
    pub const MIN_SERIALIZED_SIZE: usize = 8;

    /// Serialized length of the header in bytes/octets.
    ///
    /// Note that this size is not the size of the entire
    /// ICMPv6 packet but only the header.
    pub fn header_len(&self) -> usize {
        8
    }

    /// Setups a new header with the checksum beeing set to 0.
    pub fn new(icmp_type: Icmp6Type) -> Icmpv6Header {
        Icmpv6Header{
            icmp_type,
            checksum: 0, // will be filled in later
        }
    }

    /// Creates a [`Icmpv6Header`] with a valid checksum.
    pub fn with_checksum(icmp_type: Icmp6Type, ip_header: &Ipv6Header, payload: &[u8]) -> Result<Icmpv6Header, ValueError> {
        let checksum = icmp_type.calc_checksum(ip_header, payload)?;
        Ok(
            Icmpv6Header{
                icmp_type,
                checksum,
            }
        )
    }

    /// Write the transport header to the given writer.
    pub fn write<T: io::Write + Sized>(&self, writer: &mut T) -> Result<(), WriteError> {
        writer.write_all(&self.to_bytes()).map_err(WriteError::from)
    }

    /// Validates the checksum givene the IPv6 header and payload (parts after the Icmpv6Header) of the packet.
    pub fn is_checksum_valid(&self, ip_header: &Ipv6Header, payload: &[u8]) -> Result<bool, ValueError> {
        Ok(self.checksum == self.icmp_type.calc_checksum(ip_header, payload)?)
    }

    /// Updates the checksum of the header.
    pub fn update_checksum(&mut self, ip_header: &Ipv6Header, payload: &[u8]) -> Result<(), ValueError> {
        self.checksum = self.icmp_type.calc_checksum(ip_header, payload)?;
        Ok(())
    }

    /// Reads an icmp6 header from a slice directly and returns a tuple containing the resulting header & unused part of the slice.
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Result<(Icmpv6Header, &[u8]), ReadError> {
        let header = Icmpv6HeaderSlice::from_slice(slice)?.to_header();
        let len = header.header_len();
        Ok((
            header,
            &slice[len..]
        ))
    }

    /// Returns the header on the wire bytes.
    #[inline]
    pub fn to_bytes(&self) -> [u8;8] {
        let (type_value, code_value, bytes5to8) = self.icmp_type.to_bytes();
        let checksum_be = self.checksum.to_be_bytes();
        [
            type_value, code_value, checksum_be[0], checksum_be[1],
            bytes5to8[0], bytes5to8[1], bytes5to8[2], bytes5to8[3],
        ]
    }
}

/// A slice containing an icmp6 header of a network package. Struct allows the selective read of fields in the header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Icmpv6HeaderSlice<'a> {
    slice: &'a [u8]
}

impl<'a> Icmpv6HeaderSlice<'a> {
    /// Creates a slice containing an icmp6 header.
    #[inline]
    pub fn from_slice(slice: &'a[u8]) -> Result<Icmpv6HeaderSlice<'a>, ReadError> {
        //check length
        use crate::ReadError::*;
        if slice.len() < Icmpv6Header::MIN_SERIALIZED_SIZE {
            return Err(UnexpectedEndOfSlice(Icmpv6Header::MIN_SERIALIZED_SIZE));
        }

        //done
        Ok(Icmpv6HeaderSlice{
            // SAFETY:
            // Safe as slice length is checked to be at least
            // Icmpv6Header::MIN_SERIALIZED_SIZE (8) before this.
            slice: unsafe {
                from_raw_parts(
                    slice.as_ptr(),
                    Icmpv6Header::MIN_SERIALIZED_SIZE
                )
            }
        })
    }

    /// Decode all the fields and copy the results to a [`Icmpv6Header`] struct
    #[inline]
    pub fn to_header(&self) -> Icmpv6Header {
        Icmpv6Header {
            icmp_type: unsafe {
                Icmp6Type::from_bytes(
                    *self.slice.get_unchecked(0),
                    *self.slice.get_unchecked(1),
                    [
                        *self.slice.get_unchecked(4),
                        *self.slice.get_unchecked(5),
                        *self.slice.get_unchecked(6),
                        *self.slice.get_unchecked(7),
                    ]
                )
            },
            checksum: self.checksum(),
        }
    }

    /// Returns "type" value in the ICMPv6 header.
    #[inline]
    pub fn type_value(&self) -> u8 {
        // SAFETY:
        // Safe as the contructor checks that the slice has
        // at least the length of Icmpv6Header::MIN_SERIALIZED_SIZE (8).
        unsafe {
            *self.slice.get_unchecked(0)
        }
    }

    /// Returns "code" value in the ICMPv6 header.
    #[inline]
    pub fn code_value(&self) -> u8 {
        // SAFETY:
        // Safe as the contructor checks that the slice has
        // at least the length of Icmpv6Header::MIN_SERIALIZED_SIZE (8).
        unsafe {
            *self.slice.get_unchecked(0)
        }
    }

    /// Returns "checksum" value in the ICMPv6 header.
    #[inline]
    pub fn checksum(&self) -> u16 {
        // SAFETY:
        // Safe as the contructor checks that the slice has
        // at least the length of UdpHeader::MIN_SERIALIZED_SIZE (8).
        unsafe {
            get_unchecked_be_u16(self.slice.as_ptr().add(2))
        }
    }

    /// Returns the bytes from position 4 till and including the 8th position
    /// in the ICMPv6 header.
    ///
    /// These bytes located at th 5th, 6th, 7th and 8th position of the ICMP
    /// packet can depending on the ICMPv6 type and code contain additional data.
    #[inline]
    pub fn bytes5to8(&self) -> [u8;4] {
        // SAFETY:
        // Safe as the contructor checks that the slice has
        // at least the length of UdpHeader::MIN_SERIALIZED_SIZE (8).
        unsafe {
            [
                *self.slice.get_unchecked(4),
                *self.slice.get_unchecked(5),
                *self.slice.get_unchecked(6),
                *self.slice.get_unchecked(7),
            ]
        }
    }

    /// Returns the slice containing the icmp6 header
    #[inline]
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }
}