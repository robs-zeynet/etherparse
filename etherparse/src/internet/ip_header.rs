use super::super::*;
#[cfg(feature = "std")]
use crate::err::ip::HeaderWriteError;
use crate::err::{Layer, LenError, LenSource, ValueTooBigError};

/// Internet protocol headers version 4 & 6.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum IpHeader {
    /// IPv4 header & extension headers.
    Version4(Ipv4Header, Ipv4Extensions),
    /// IPv6 header & extension headers.
    Version6(Ipv6Header, Ipv6Extensions),
}

impl IpHeader {
    /// Maximum summed up length of all extension headers in bytes/octets.
    pub const MAX_LEN: usize = Ipv6Header::LEN + Ipv6Extensions::MAX_LEN;

    /// Renamed to `IpHeader::from_slice`
    #[deprecated(since = "0.10.1", note = "Renamed to `IpHeader::from_slice`")]
    #[inline]
    pub fn read_from_slice(
        slice: &[u8],
    ) -> Result<(IpHeader, IpNumber, &[u8]), err::ip::HeaderSliceError> {
        let (header, payload) = IpHeader::from_slice(slice)?;
        Ok((header, payload.ip_number, payload.payload))
    }

    /// Read an IpHeaders from a slice and return the header & slice
    /// containing the rest of the payload of IP packet (determined based
    /// on the header).
    ///
    /// Note that his function verfies `total_length` for IPv4 and
    /// `payload_lenght` for IPv6 and that the extension headers are contained
    /// within.
    pub fn from_slice(
        slice: &[u8],
    ) -> Result<(IpHeader, IpPayload<'_>), err::ip::HeaderSliceError> {
        use err::ip::{HeaderError::*, HeaderSliceError::*};

        if slice.is_empty() {
            Err(Len(err::LenError {
                required_len: 1,
                len: slice.len(),
                len_source: err::LenSource::Slice,
                layer: err::Layer::IpHeader,
                layer_start_offset: 0,
            }))
        } else {
            match slice[0] >> 4 {
                4 => {
                    // check length
                    if slice.len() < Ipv4Header::MIN_LEN {
                        return Err(Len(err::LenError {
                            required_len: Ipv4Header::MIN_LEN,
                            len: slice.len(),
                            len_source: err::LenSource::Slice,
                            layer: err::Layer::Ipv4Header,
                            layer_start_offset: 0,
                        }));
                    }

                    // read ihl
                    //
                    // SAFETY:
                    // Safe as the slice length is checked to be at least
                    // Ipv4Header::MIN_LEN (20) at the start.
                    let ihl = unsafe { slice.get_unchecked(0) } & 0xf;

                    //check that the ihl is correct
                    if ihl < 5 {
                        return Err(Content(Ipv4HeaderLengthSmallerThanHeader { ihl }));
                    }

                    // check that the slice contains enough data for the entire header + options
                    let header_len = usize::from(ihl) * 4;
                    if slice.len() < header_len {
                        return Err(Len(LenError {
                            required_len: header_len,
                            len: slice.len(),
                            len_source: LenSource::Slice,
                            layer: Layer::Ipv4Header,
                            layer_start_offset: 0,
                        }));
                    }

                    let header = unsafe {
                        // SAFETY: Safe as the IHL & slice len has been validated
                        Ipv4HeaderSlice::from_slice_unchecked(core::slice::from_raw_parts(
                            slice.as_ptr(),
                            header_len,
                        ))
                        .to_header()
                    };

                    // check that the total len is at least containing the header len
                    let total_len: usize = header.total_len.into();
                    if total_len < header_len {
                        return Err(Len(LenError {
                            required_len: header_len,
                            len: total_len,
                            len_source: LenSource::Ipv4HeaderTotalLen,
                            layer: Layer::Ipv4Packet,
                            layer_start_offset: 0,
                        }));
                    }

                    // restrict the rest of the slice based on the total len
                    let rest = if slice.len() < total_len {
                        return Err(Len(LenError {
                            required_len: total_len,
                            len: slice.len(),
                            len_source: LenSource::Slice,
                            layer: Layer::Ipv4Packet,
                            layer_start_offset: 0,
                        }));
                    } else {
                        unsafe {
                            core::slice::from_raw_parts(
                                // SAFETY: Safe as the slice length was validated to be at least header_length
                                slice.as_ptr().add(header_len),
                                // SAFETY: Safe as slice length has been validated to be at least total_length_usize long
                                total_len - header_len,
                            )
                        }
                    };

                    let (exts, next_protocol, rest) =
                        Ipv4Extensions::from_slice(header.protocol, rest).map_err(|err| {
                            use err::ip_auth::HeaderSliceError as I;
                            match err {
                                I::Len(mut err) => {
                                    err.layer_start_offset += header_len;
                                    err.len_source = LenSource::Ipv4HeaderTotalLen;
                                    Len(err)
                                }
                                I::Content(err) => Content(Ipv4Ext(err)),
                            }
                        })?;

                    let fragmented = header.is_fragmenting_payload();
                    Ok((
                        IpHeader::Version4(header, exts),
                        IpPayload {
                            ip_number: next_protocol,
                            fragmented,
                            len_source: LenSource::Ipv4HeaderTotalLen,
                            payload: rest,
                        },
                    ))
                }
                6 => {
                    if slice.len() < Ipv6Header::LEN {
                        return Err(Len(err::LenError {
                            required_len: Ipv6Header::LEN,
                            len: slice.len(),
                            len_source: err::LenSource::Slice,
                            layer: err::Layer::Ipv6Header,
                            layer_start_offset: 0,
                        }));
                    }
                    let header = {
                        // SAFETY:
                        // This is safe as the slice length is checked to be
                        // at least Ipv6Header::LEN (40) befpre this code block.
                        unsafe {
                            Ipv6HeaderSlice::from_slice_unchecked(core::slice::from_raw_parts(
                                slice.as_ptr(),
                                Ipv6Header::LEN,
                            ))
                            .to_header()
                        }
                    };

                    // restrict slice by the length specified in the header
                    let (header_payload, len_source) =
                        if 0 == header.payload_length && slice.len() > Ipv6Header::LEN {
                            // In case the payload_length is 0 assume that the entire
                            // rest of the slice is part of the packet until the jumbogram
                            // parameters can be parsed.

                            // TODO: Add payload length parsing from the jumbogram
                            unsafe {
                                (
                                    core::slice::from_raw_parts(
                                        slice.as_ptr().add(Ipv6Header::LEN),
                                        slice.len() - Ipv6Header::LEN,
                                    ),
                                    LenSource::Slice,
                                )
                            }
                        } else {
                            let payload_len: usize = header.payload_length.into();
                            let expected_len = Ipv6Header::LEN + payload_len;
                            if slice.len() < expected_len {
                                return Err(Len(LenError {
                                    required_len: expected_len,
                                    len: slice.len(),
                                    len_source: LenSource::Slice,
                                    layer: Layer::Ipv6Packet,
                                    layer_start_offset: 0,
                                }));
                            } else {
                                unsafe {
                                    (
                                        core::slice::from_raw_parts(
                                            slice.as_ptr().add(Ipv6Header::LEN),
                                            payload_len,
                                        ),
                                        LenSource::Ipv6HeaderPayloadLen,
                                    )
                                }
                            }
                        };

                    let (exts, next_header, rest) =
                        Ipv6Extensions::from_slice(header.next_header, header_payload).map_err(
                            |err| {
                                use err::ipv6_exts::HeaderSliceError as I;
                                match err {
                                    I::Len(mut err) => {
                                        err.layer_start_offset += Ipv6Header::LEN;
                                        err.len_source = len_source;
                                        Len(err)
                                    }
                                    I::Content(err) => Content(Ipv6Ext(err)),
                                }
                            },
                        )?;

                    let fragmented = exts.is_fragmenting_payload();
                    Ok((
                        IpHeader::Version6(header, exts),
                        IpPayload {
                            ip_number: next_header,
                            fragmented,
                            len_source,
                            payload: rest,
                        },
                    ))
                }
                version_number => Err(Content(UnsupportedIpVersion { version_number })),
            }
        }
    }

    /// Read an IPv4 header & extension headers from a slice and return the slice containing the payload
    /// according to the total_length field in the IPv4 header.
    pub fn ipv4_from_slice(
        slice: &[u8],
    ) -> Result<(IpHeader, IpPayload<'_>), err::ipv4::SliceError> {
        use err::ipv4::SliceError::*;

        // read the header
        let (header, header_rest) = Ipv4Header::from_slice(slice).map_err(|err| {
            use err::ipv4::HeaderSliceError as I;
            match err {
                I::Len(err) => Len(err),
                I::Content(err) => Header(err),
            }
        })?;

        // check that the total length at least contains the header
        let total_len: usize = header.total_len.into();
        let header_len = header.header_len();
        let payload_len = if total_len >= header_len {
            total_len - header_len
        } else {
            return Err(Len(LenError {
                required_len: header_len,
                len: total_len,
                len_source: LenSource::Ipv4HeaderTotalLen,
                layer: Layer::Ipv4Packet,
                layer_start_offset: 0,
            }));
        };

        // limit rest based on ipv4 total length
        let header_rest = if payload_len > header_rest.len() {
            return Err(Len(err::LenError {
                required_len: total_len,
                len: slice.len(),
                len_source: LenSource::Slice,
                layer: Layer::Ipv4Packet,
                layer_start_offset: 0,
            }));
        } else {
            unsafe {
                // Safe as the payload_len <= header_rest.len is verfied above
                core::slice::from_raw_parts(header_rest.as_ptr(), payload_len)
            }
        };

        // read the extension header
        let (exts, next_header, exts_rest) =
            Ipv4Extensions::from_slice(header.protocol, header_rest).map_err(|err| {
                use err::ip_auth::HeaderSliceError as I;
                match err {
                    I::Len(mut err) => {
                        err.layer_start_offset += header.header_len();
                        err.len_source = LenSource::Ipv4HeaderTotalLen;
                        Len(err)
                    }
                    I::Content(err) => Exts(err),
                }
            })?;

        let fragmented = header.is_fragmenting_payload();
        Ok((
            IpHeader::Version4(header, exts),
            IpPayload {
                ip_number: next_header,
                fragmented,
                len_source: LenSource::Ipv4HeaderTotalLen,
                payload: exts_rest,
            },
        ))
    }

    /// Read an IPv6 header & extension headers from a slice and return the slice
    /// containing the payload (e.g. TCP, UDP etc.) length limited by payload_length
    /// field in the IPv6 header.
    ///
    /// Note that slice length is used as a fallback value in case the
    /// payload_length in the IPv6 is set to zero. This is a temporary workaround
    /// to partially support jumbograms.
    pub fn ipv6_from_slice(
        slice: &[u8],
    ) -> Result<(IpHeader, IpPayload<'_>), err::ipv6::SliceError> {
        use err::ipv6::SliceError::*;

        // read ipv6 header
        let (header, header_rest) = Ipv6Header::from_slice(slice).map_err(|err| {
            use err::ipv6::HeaderSliceError as I;
            match err {
                I::Len(err) => Len(err),
                I::Content(err) => Header(err),
            }
        })?;

        // restrict slice by the length specified in the header
        let (header_payload, len_source) =
            if 0 == header.payload_length && slice.len() > Ipv6Header::LEN {
                // In case the payload_length is 0 assume that the entire
                // rest of the slice is part of the packet until the jumbogram
                // parameters can be parsed.

                // TODO: Add payload length parsing from the jumbogram
                (header_rest, LenSource::Slice)
            } else {
                let payload_len: usize = header.payload_length.into();
                if header_rest.len() < payload_len {
                    return Err(Len(LenError {
                        required_len: payload_len + Ipv6Header::LEN,
                        len: slice.len(),
                        len_source: LenSource::Slice,
                        layer: Layer::Ipv6Packet,
                        layer_start_offset: 0,
                    }));
                } else {
                    unsafe {
                        (
                            core::slice::from_raw_parts(header_rest.as_ptr(), payload_len),
                            LenSource::Ipv6HeaderPayloadLen,
                        )
                    }
                }
            };

        // read ipv6 extensions headers
        let (exts, next_header, exts_rest) =
            Ipv6Extensions::from_slice(header.next_header, header_payload).map_err(|err| {
                use err::ipv6_exts::HeaderSliceError as I;
                match err {
                    I::Len(mut err) => {
                        err.layer_start_offset += Ipv6Header::LEN;
                        err.len_source = len_source;
                        Len(err)
                    }
                    I::Content(err) => Exts(err),
                }
            })?;

        let fragmented = exts.is_fragmenting_payload();
        Ok((
            IpHeader::Version6(header, exts),
            IpPayload {
                ip_number: next_header,
                fragmented,
                len_source,
                payload: exts_rest,
            },
        ))
    }

    /// Reads an IP (v4 or v6) header from the current position (requires
    /// crate feature `std`).
    #[cfg(feature = "std")]
    pub fn read<T: std::io::Read + std::io::Seek + Sized>(
        reader: &mut T,
    ) -> Result<(IpHeader, IpNumber), err::ip::HeaderReadError> {
        use crate::io::LimitedReader;
        use err::ip::{HeaderError::*, HeaderReadError::*};

        let value = {
            let mut buf = [0; 1];
            reader.read_exact(&mut buf).map_err(Io)?;
            buf[0]
        };
        match value >> 4 {
            4 => {
                // get internet header length
                let ihl = value & 0xf;

                // check that the ihl is correct
                if ihl < 5 {
                    return Err(Content(Ipv4HeaderLengthSmallerThanHeader { ihl }));
                }

                // read the rest of the header
                let header_len_u16 = u16::from(ihl) * 4;
                let header_len = usize::from(header_len_u16);
                let mut buffer = [0u8; Ipv4Header::MAX_LEN];
                buffer[0] = value;
                reader.read_exact(&mut buffer[1..header_len]).map_err(Io)?;

                let header = unsafe {
                    // SAFETY: Safe as both the IHL and slice len have been verified
                    Ipv4HeaderSlice::from_slice_unchecked(&buffer[..header_len])
                }
                .to_header();

                // check that the total len is long enough to contain the header
                let total_len = usize::from(header.total_len);
                let mut reader = if total_len < header_len {
                    return Err(Len(LenError {
                        required_len: header_len,
                        len: total_len,
                        len_source: LenSource::Ipv4HeaderTotalLen,
                        layer: Layer::Ipv4Packet,
                        layer_start_offset: 0,
                    }));
                } else {
                    // create a new reader that is limited by the total_len value length
                    LimitedReader::new(
                        reader,
                        total_len - header_len,
                        LenSource::Ipv4HeaderTotalLen,
                        header_len,
                        Layer::Ipv4Header,
                    )
                };

                // read the extension headers if present
                Ipv4Extensions::read_limited(&mut reader, header.protocol)
                    .map(|(ext, next)| (IpHeader::Version4(header, ext), next))
                    .map_err(|err| {
                        use err::ip_auth::HeaderLimitedReadError as I;
                        match err {
                            I::Io(err) => Io(err),
                            I::Len(err) => Len(err),
                            I::Content(err) => Content(Ipv4Ext(err)),
                        }
                    })
            }
            6 => {
                let header = Ipv6Header::read_without_version(reader, value & 0xf).map_err(Io)?;

                // create a new reader that is limited by the payload_len value length
                let mut reader = LimitedReader::new(
                    reader,
                    header.payload_length.into(),
                    LenSource::Ipv6HeaderPayloadLen,
                    header.header_len(),
                    Layer::Ipv6Header,
                );

                Ipv6Extensions::read_limited(&mut reader, header.next_header)
                    .map(|(ext, next)| (IpHeader::Version6(header, ext), next))
                    .map_err(|err| {
                        use err::ipv6_exts::HeaderLimitedReadError as I;
                        match err {
                            I::Io(err) => Io(err),
                            I::Len(err) => Len(err),
                            I::Content(err) => Content(Ipv6Ext(err)),
                        }
                    })
            }
            version_number => Err(Content(UnsupportedIpVersion { version_number })),
        }
    }

    /// Writes an IP (v4 or v6) header to the current position (requires
    /// crate feature `std`).
    #[cfg(feature = "std")]
    pub fn write<T: std::io::Write + Sized>(&self, writer: &mut T) -> Result<(), HeaderWriteError> {
        use crate::IpHeader::*;
        use HeaderWriteError::*;
        match *self {
            Version4(ref header, ref extensions) => {
                header.write(writer).map_err(Io)?;
                extensions.write(writer, header.protocol).map_err(|err| {
                    use err::ipv4_exts::HeaderWriteError as I;
                    match err {
                        I::Io(err) => Io(err),
                        I::Content(err) => Ipv4Exts(err),
                    }
                })
            }
            Version6(ref header, ref extensions) => {
                header.write(writer).map_err(Io)?;
                extensions.write(writer, header.next_header).map_err(|err| {
                    use err::ipv6_exts::HeaderWriteError as I;
                    match err {
                        I::Io(err) => Io(err),
                        I::Content(err) => Ipv6Exts(err),
                    }
                })
            }
        }
    }

    /// Returns the size when the ip header & extensions are serialized
    pub fn header_len(&self) -> usize {
        use crate::IpHeader::*;
        match *self {
            Version4(ref header, ref extensions) => header.header_len() + extensions.header_len(),
            Version6(_, ref extensions) => Ipv6Header::LEN + extensions.header_len(),
        }
    }

    /// Returns the last next header number following the ip header
    /// and header extensions.
    pub fn next_header(&self) -> Result<IpNumber, err::ip_exts::ExtsWalkError> {
        use crate::err::ip_exts::ExtsWalkError::*;
        use crate::IpHeader::*;
        match *self {
            Version4(ref header, ref extensions) => {
                extensions.next_header(header.protocol).map_err(Ipv4Exts)
            }
            Version6(ref header, ref extensions) => {
                extensions.next_header(header.next_header).map_err(Ipv6Exts)
            }
        }
    }

    /// Sets all the next_header fields in the ipv4 & ipv6 header
    /// as well as in all extension headers and returns the ether
    /// type number.
    ///
    /// The given number will be set as the last "next_header" or
    /// protocol number.
    pub fn set_next_headers(&mut self, last_next_header: IpNumber) -> u16 {
        use IpHeader::*;
        match self {
            Version4(ref mut header, ref mut extensions) => {
                header.protocol = extensions.set_next_headers(last_next_header);
                EtherType::IPV4.0
            }
            Version6(ref mut header, ref mut extensions) => {
                header.next_header = extensions.set_next_headers(last_next_header);
                EtherType::IPV4.0
            }
        }
    }

    /// Tries to set the length field in the ip header given the length of data
    /// after the ip header and extension header(s).
    ///
    /// If the payload length is too large to be stored in the length fields
    /// of the ip header an error is returned.
    ///
    /// Note that this function will automatically add the length of the extension
    /// headers is they are present.
    pub fn set_payload_len(&mut self, len: usize) -> Result<(), ValueTooBigError<usize>> {
        use crate::err::ValueType;
        match self {
            IpHeader::Version4(ipv4_hdr, exts) => {
                if let Some(complete_len) = len.checked_add(exts.header_len()) {
                    ipv4_hdr.set_payload_len(complete_len)
                } else {
                    Err(ValueTooBigError {
                        actual: len,
                        max_allowed: usize::from(u16::MAX)
                            - ipv4_hdr.header_len()
                            - exts.header_len(),
                        value_type: ValueType::Ipv4PayloadLength,
                    })
                }
            }
            IpHeader::Version6(ipv6_hdr, exts) => {
                if let Some(complete_len) = len.checked_add(exts.header_len()) {
                    ipv6_hdr.set_payload_length(complete_len)
                } else {
                    Err(ValueTooBigError {
                        actual: len,
                        max_allowed: usize::from(u16::MAX) - exts.header_len(),
                        value_type: ValueType::Ipv4PayloadLength,
                    })
                }
            }
        }
    }

    /// Returns true if the payload is fragmented based on the IPv4 header
    /// or the IPv6 fragment header.
    pub fn is_fragmenting_payload(&self) -> bool {
        match self {
            IpHeader::Version4(ipv4, _) => ipv4.is_fragmenting_payload(),
            IpHeader::Version6(_, exts) => exts.is_fragmenting_payload(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        err::{
            ip::{HeaderError, HeaderSliceError},
            Layer, LenError, LenSource,
        },
        ip_number::*,
        test_gens::*,
        *,
    };
    use alloc::{borrow::ToOwned, format, vec::Vec};
    use proptest::prelude::*;
    use std::io::Cursor;

    const EXTESION_KNOWN_IP_NUMBERS: [IpNumber; 5] = [
        AUTH,
        IPV6_DEST_OPTIONS,
        IPV6_HOP_BY_HOP,
        IPV6_FRAG,
        IPV6_ROUTE,
    ];

    fn combine_v4(v4: &Ipv4Header, ext: &Ipv4Extensions, payload: &[u8]) -> IpHeader {
        IpHeader::Version4(
            {
                let mut v4 = v4.clone();
                v4.protocol = if ext.auth.is_some() { AUTH } else { UDP };
                v4.total_len = (v4.header_len() + ext.header_len() + payload.len()) as u16;
                v4.header_checksum = v4.calc_header_checksum();
                v4
            },
            ext.clone(),
        )
    }

    fn combine_v6(v6: &Ipv6Header, ext: &Ipv6Extensions, payload: &[u8]) -> IpHeader {
        let (ext, next_header) = {
            let mut ext = ext.clone();
            let next_header = ext.set_next_headers(UDP);
            (ext, next_header)
        };
        IpHeader::Version6(
            {
                let mut v6 = v6.clone();
                v6.next_header = next_header;
                v6.payload_length = (ext.header_len() + payload.len()) as u16;
                v6
            },
            ext,
        )
    }

    proptest! {
        #[test]
        fn debug(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            assert_eq!(
                format!(
                    "Version4({:?}, {:?})",
                    v4,
                    v4_exts
                ),
                format!("{:?}", IpHeader::Version4(v4, v4_exts))
            );
            assert_eq!(
                format!(
                    "Version6({:?}, {:?})",
                    v6,
                    v6_exts
                ),
                format!("{:?}", IpHeader::Version6(v6, v6_exts))
            );
        }
    }

    proptest! {
        #[test]
        fn clone_eq(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            {
                let v4 = IpHeader::Version4(v4, v4_exts);
                assert_eq!(v4, v4.clone());
            }
            {
                let v6 = IpHeader::Version6(v6, v6_exts);
                assert_eq!(v6, v6.clone());
            }
        }
    }

    proptest! {
        #[test]
        #[allow(deprecated)]
        fn read_from_slice(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
        ) {
            let header = combine_v4(&v4, &v4_exts, &[]);
            let mut buffer = Vec::with_capacity(header.header_len());
            header.write(&mut buffer).unwrap();

            let actual = IpHeader::read_from_slice(&buffer).unwrap();
            assert_eq!(actual.0, header);
            assert_eq!(actual.1, header.next_header().unwrap());
            assert_eq!(actual.2, &buffer[buffer.len()..]);
        }
    }

    proptest! {
        #[test]
        fn from_slice(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            let payload = [1,2,3,4];

            // v4
            {
                let header = combine_v4(&v4, &v4_exts, &payload);
                let mut buffer = Vec::with_capacity(header.header_len() + payload.len() + 1);
                header.write(&mut buffer).unwrap();
                buffer.extend_from_slice(&payload);
                buffer.push(1); // add some value to check the return slice

                // read
                {
                    let actual = IpHeader::from_slice(&buffer).unwrap();
                    assert_eq!(&actual.0, &header);
                    assert_eq!(
                        actual.1,
                        IpPayload{
                            ip_number: header.next_header().unwrap(),
                            fragmented: header.is_fragmenting_payload(),
                            len_source: LenSource::Ipv4HeaderTotalLen,
                            payload: &payload
                        }
                    );
                }

                // read error ipv4 header
                IpHeader::from_slice(&buffer[..1]).unwrap_err();

                // read error ipv4 extensions
                if v4_exts.header_len() > 0 {
                    IpHeader::from_slice(&buffer[..v4.header_len() + 1]).unwrap_err();
                }

                // total length smaller the header
                {
                    let bad_total_len = (v4.header_len() - 1) as u16;

                    let mut buffer = buffer.clone();
                    // inject bad total_len
                    let bad_total_len_be = bad_total_len.to_be_bytes();
                    buffer[2] = bad_total_len_be[0];
                    buffer[3] = bad_total_len_be[1];
                    assert_eq!(
                        IpHeader::from_slice(&buffer[..]).unwrap_err(),
                        HeaderSliceError::Len(LenError{
                            required_len: v4.header_len(),
                            len: bad_total_len as usize,
                            len_source: LenSource::Ipv4HeaderTotalLen,
                            layer: Layer::Ipv4Packet,
                            layer_start_offset: 0,
                        })
                    );
                }
            }

            // v6
            {
                let header = combine_v6(&v6, &v6_exts, &payload);
                let mut buffer = Vec::with_capacity(header.header_len() + payload.len() + 1);
                header.write(&mut buffer).unwrap();
                buffer.extend_from_slice(&payload);
                buffer.push(1); // add some value to check the return slice

                // len error
                {
                    let actual = IpHeader::from_slice(&buffer).unwrap();
                    assert_eq!(&actual.0, &header);
                    assert_eq!(
                        actual.1,
                        IpPayload{
                            ip_number: header.next_header().unwrap(),
                            fragmented: header.is_fragmenting_payload(),
                            len_source: LenSource::Ipv6HeaderPayloadLen,
                            payload: &payload
                        }
                    );
                }

                // read error header
                IpHeader::from_slice(&buffer[..1]).unwrap_err();

                // read error ipv4 extensions
                if v6_exts.header_len() > 0 {
                    IpHeader::from_slice(&buffer[..Ipv6Header::LEN + 1]).unwrap_err();
                }

                // len error (with payload len zero)
                if v6_exts.header_len() > 0 {
                    let mut buffer = buffer.clone();

                    // inject zero as payload len
                    buffer[4] = 0;
                    buffer[5] = 0;

                    assert!(
                        IpHeader::from_slice(
                            &buffer[..buffer.len() - payload.len() - 2]
                        ).is_err()
                    );
                }
            }
        }
    }

    proptest! {
        #[test]
        fn ipv4_from_slice(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
        ) {
            let payload = [1,2,3,4];

            let header = combine_v4(&v4, &v4_exts, &payload);
            let mut buffer = Vec::with_capacity(header.header_len() + payload.len() + 1);
            header.write(&mut buffer).unwrap();
            buffer.extend_from_slice(&payload);
            buffer.push(1); // add some value to check the return slice

            // read
            {
                let actual = IpHeader::ipv4_from_slice(&buffer).unwrap();
                assert_eq!(&actual.0, &header);
                assert_eq!(
                    actual.1,
                    IpPayload{
                        ip_number: header.next_header().unwrap(),
                        fragmented: header.is_fragmenting_payload(),
                        len_source: LenSource::Ipv4HeaderTotalLen,
                        payload: &payload
                    }
                );
            }

            // read error ipv4 header
            IpHeader::ipv4_from_slice(&buffer[..1]).unwrap_err();

            // read error ipv4 extensions
            if v4_exts.header_len() > 0 {
                IpHeader::ipv4_from_slice(&buffer[..v4.header_len() + 1]).unwrap_err();
            }

            // total length smaller the header
            {
                let bad_total_len = (v4.header_len() - 1) as u16;

                let mut buffer = buffer.clone();
                // inject bad total_len
                let bad_total_len_be = bad_total_len.to_be_bytes();
                buffer[2] = bad_total_len_be[0];
                buffer[3] = bad_total_len_be[1];
                assert_eq!(
                    IpHeader::ipv4_from_slice(&buffer[..]).unwrap_err(),
                    err::ipv4::SliceError::Len(LenError{
                        required_len: v4.header_len(),
                        len: bad_total_len as usize,
                        len_source: LenSource::Ipv4HeaderTotalLen,
                        layer: Layer::Ipv4Packet,
                        layer_start_offset: 0,
                    })
                );
            }
        }
    }

    proptest! {
        #[test]
        fn ipv6_from_slice(
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            let payload = [1,2,3,4];
            let header = combine_v6(&v6, &v6_exts, &payload);
            let mut buffer = Vec::with_capacity(header.header_len() + payload.len() + 1);
            header.write(&mut buffer).unwrap();
            buffer.extend_from_slice(&payload);
            buffer.push(1); // add some value to check the return slice

            // len error
            {
                let actual = IpHeader::ipv6_from_slice(&buffer).unwrap();
                assert_eq!(&actual.0, &header);
                assert_eq!(
                    actual.1,
                    IpPayload{
                        ip_number: header.next_header().unwrap(),
                        fragmented: header.is_fragmenting_payload(),
                        len_source: LenSource::Ipv6HeaderPayloadLen,
                        payload: &payload
                    }
                );
            }

            // read error header
            IpHeader::ipv6_from_slice(&buffer[..1]).unwrap_err();

            // read error ipv4 extensions
            if v6_exts.header_len() > 0 {
                IpHeader::ipv6_from_slice(&buffer[..Ipv6Header::LEN + 1]).unwrap_err();
            }

            // len error (with payload len zero)
            if v6_exts.header_len() > 0 {
                let mut buffer = buffer.clone();

                // inject zero as payload len
                buffer[4] = 0;
                buffer[5] = 0;

                assert!(
                    IpHeader::ipv6_from_slice(
                        &buffer[..buffer.len() - payload.len() - 2]
                    ).is_err()
                );
            }
        }
    }

    proptest! {
        #[test]
        fn read(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            bad_ihl in 0u8..5u8,
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            // no data error
            {
                let mut cursor = Cursor::new(&[]);
                assert!(
                    IpHeader::read(&mut cursor)
                    .unwrap_err()
                    .io()
                    .is_some()
                );
            }
            // v4
            {
                let header = combine_v4(&v4, &v4_exts, &[]);
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                // read
                {
                    let mut cursor = Cursor::new(&buffer[..]);
                    let actual = IpHeader::read(&mut cursor).unwrap();
                    assert_eq!(actual.0, header);
                    assert_eq!(actual.1, header.next_header().unwrap());
                }

                // read error ihl smaller then header
                {
                    let mut buffer = buffer.clone();
                    // inject bad ihl
                    buffer[0] = (buffer[0] & 0b1111_0000) | bad_ihl;
                    let mut cursor = Cursor::new(&buffer[..]);
                    assert_eq!(
                        IpHeader::read(&mut cursor)
                        .unwrap_err()
                        .content()
                        .unwrap(),
                        HeaderError::Ipv4HeaderLengthSmallerThanHeader{
                            ihl: bad_ihl
                        }
                    );
                }

                // total length smaller the header
                {
                    let bad_total_len = (v4.header_len() - 1) as u16;

                    let mut buffer = buffer.clone();
                    // inject bad total_len
                    let bad_total_len_be = bad_total_len.to_be_bytes();
                    buffer[2] = bad_total_len_be[0];
                    buffer[3] = bad_total_len_be[1];
                    let mut cursor = Cursor::new(&buffer[..]);
                    assert_eq!(
                        IpHeader::read(&mut cursor)
                        .unwrap_err()
                        .len()
                        .unwrap(),
                        LenError{
                            required_len: v4.header_len(),
                            len: bad_total_len as usize,
                            len_source: LenSource::Ipv4HeaderTotalLen,
                            layer: Layer::Ipv4Packet,
                            layer_start_offset: 0,
                        }
                    );
                }

                // read len error ipv4
                {
                    let mut cursor = Cursor::new(&buffer[..1]);
                    assert!(
                        IpHeader::read(&mut cursor)
                        .unwrap_err()
                        .io()
                        .is_some()
                    );
                }

                // read error ipv4 extensions
                if v4_exts.header_len() > 0 {
                    let mut cursor = Cursor::new(&buffer[..v4.header_len() + 1]);
                    IpHeader::read(&mut cursor).unwrap_err();
                }

                // len error in extensions
                if v4_exts.auth.is_some() {
                    let bad_total_len = (buffer.len() - 1) as u16;

                    let mut buffer = buffer.clone();
                    // inject bad total_len
                    let bad_total_len_be = bad_total_len.to_be_bytes();
                    buffer[2] = bad_total_len_be[0];
                    buffer[3] = bad_total_len_be[1];
                    let mut cursor = Cursor::new(&buffer[..]);
                    assert_eq!(
                        IpHeader::read(&mut cursor)
                        .unwrap_err()
                        .len()
                        .unwrap(),
                        LenError{
                            required_len: buffer.len() - v4.header_len(),
                            len: bad_total_len as usize - v4.header_len(),
                            len_source: LenSource::Ipv4HeaderTotalLen,
                            layer: Layer::IpAuthHeader,
                            layer_start_offset: v4.header_len(),
                        }
                    );
                }

                // extension content error
                if v4_exts.auth.is_some() {
                    let mut buffer = buffer.clone();
                    // inject zero as header len
                    buffer[v4.header_len() + 1] = 0;
                    let mut cursor = Cursor::new(&buffer[..]);
                    assert_eq!(
                        IpHeader::read(&mut cursor)
                        .unwrap_err()
                        .content()
                        .unwrap(),
                        HeaderError::Ipv4Ext(
                            err::ip_auth::HeaderError::ZeroPayloadLen
                        )
                    );
                }
            }

            // v6
            {
                let header = combine_v6(&v6, &v6_exts, &[]);
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                // ok case
                {
                    let mut cursor = Cursor::new(&buffer[..]);
                    let actual = IpHeader::read(&mut cursor).unwrap();
                    assert_eq!(actual.0, header);
                    assert_eq!(actual.1, header.next_header().unwrap());
                }

                // io error in v6 header section
                {
                    let mut cursor = Cursor::new(&buffer[..1]);
                    assert!(
                        IpHeader::read(&mut cursor).unwrap_err().io().is_some()
                    );
                }

                // io error ipv6 extensions
                if v6_exts.header_len() > 0 {
                    let mut cursor = Cursor::new(&buffer[..Ipv6Header::LEN + 1]);
                    assert!(
                        IpHeader::read(&mut cursor).unwrap_err().io().is_some()
                    );
                }

                // len error in ipv6 extensions
                if v6_exts.header_len() > 0 {
                    // inject an invalid length
                    let mut buffer = buffer.clone();
                    let bad_payload_len = (buffer.len() - header.header_len()) as u16;
                    let bad_payload_len_be = bad_payload_len.to_be_bytes();
                    buffer[4] = bad_payload_len_be[0];
                    buffer[5] = bad_payload_len_be[1];
                    // expect a length error
                    let mut cursor = Cursor::new(&buffer[..]);
                    assert!(
                        IpHeader::read(&mut cursor).unwrap_err().len().is_some()
                    );
                }

                // extension content error
                if let Some(auth) = v6_exts.auth.as_ref() {
                    // only do it if auth is the last header
                    if v6_exts.routing.is_none() {
                        // inject zero as header len
                        let mut buffer = buffer.clone();
                        let auth_offset = buffer.len() - auth.header_len();
                        buffer[auth_offset + 1] = 0;
                        let mut cursor = Cursor::new(&buffer[..]);
                        assert_eq!(
                            IpHeader::read(&mut cursor)
                            .unwrap_err()
                            .content()
                            .unwrap(),
                            HeaderError::Ipv6Ext(err::ipv6_exts::HeaderError::IpAuth(
                                err::ip_auth::HeaderError::ZeroPayloadLen
                            ))
                        );
                    }
                }
            }
        }
    }

    proptest! {
        #[test]
        fn write(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            // v4
            {
                let header = combine_v4(&v4, &v4_exts, &[]);
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                let actual = IpHeader::from_slice(&buffer).unwrap().0;
                assert_eq!(header, actual);

                // write error v4 header
                {
                    let mut buffer = [0u8;1];
                    let mut cursor = Cursor::new(&mut buffer[..]);
                    assert!(
                        header.write(&mut cursor)
                        .unwrap_err()
                        .io()
                        .is_some()
                    );
                }

                // write io error v4 extension headers
                if v4_exts.header_len() > 0 {
                    let mut buffer = [0u8;Ipv4Header::MAX_LEN + 1];
                    let mut cursor = Cursor::new(&mut buffer[..v4.header_len() + 1]);
                    assert!(
                        header.write(&mut cursor)
                        .unwrap_err()
                        .io()
                        .is_some()
                    );
                }

                // write content error v4 extension headers
                if v4_exts.header_len() > 0 {
                    // cause a missing reference error
                    let header = IpHeader::Version4(
                        {
                            let mut v4 = v4.clone();
                            // skips extension header
                            v4.protocol = ip_number::UDP;
                            v4.total_len = (v4.header_len() + v4_exts.header_len()) as u16;
                            v4.header_checksum = v4.calc_header_checksum();
                            v4
                        },
                        v4_exts.clone(),
                    );
                    let mut buffer = [0u8;Ipv4Header::MAX_LEN + IpAuthHeader::MAX_LEN];
                    let mut cursor = Cursor::new(&mut buffer[..]);
                    assert!(header.write(&mut cursor).is_err());
                }
            }

            // v6
            {
                let header = combine_v6(&v6, &v6_exts, &[]);

                // normal write
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                let actual = IpHeader::from_slice(&buffer).unwrap().0;
                assert_eq!(header, actual);

                // write error v6 header
                {
                    let mut buffer = [0u8;1];
                    let mut cursor = Cursor::new(&mut buffer[..]);
                    assert!(
                        header.write(&mut cursor)
                        .unwrap_err()
                        .io()
                        .is_some()
                    );
                }

                // write error v6 extension headers
                if v6_exts.header_len() > 0 {
                    let mut buffer = [0u8;Ipv6Header::LEN + 1];
                    let mut cursor = Cursor::new(&mut buffer[..]);
                    assert!(
                        header.write(&mut cursor)
                        .unwrap_err()
                        .io()
                        .is_some()
                    );
                }
                // write content error v4 extension headers
                if v6_exts.header_len() > 0 {
                    // cause a missing reference error
                    let header = IpHeader::Version6(
                        {
                            let mut v6 = v6.clone();
                            // skips extension header
                            v6.next_header = ip_number::UDP;
                            v6.payload_length = v6_exts.header_len() as u16;
                            v6
                        },
                        v6_exts.clone(),
                    );
                    let mut buffer = [0u8;Ipv4Header::MAX_LEN + IpAuthHeader::MAX_LEN];
                    let mut cursor = Cursor::new(&mut buffer[..]);
                    assert!(header.write(&mut cursor).is_err());
                }
            }
        }
    }

    proptest! {
        #[test]
        fn header_len(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            assert_eq!(
                v4.header_len() + v4_exts.header_len(),
                IpHeader::Version4(v4, v4_exts).header_len()
            );
            assert_eq!(
                Ipv6Header::LEN + v6_exts.header_len(),
                IpHeader::Version6(v6, v6_exts).header_len()
            );
        }
    }

    proptest! {
        #[test]
        fn next_header(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
            post_header in ip_number_any()
                .prop_filter("Must be a non ipv6 header relevant ip number".to_owned(),
                    |v| !EXTESION_KNOWN_IP_NUMBERS.iter().any(|&x| v == &x)
                )
        ) {
            {
                let mut header = v4.clone();
                let mut exts = v4_exts.clone();
                header.protocol = exts.set_next_headers(post_header);
                assert_eq!(
                    Ok(post_header),
                    IpHeader::Version4(header, exts).next_header()
                );
            }
            {
                let mut header = v6.clone();
                let mut exts = v6_exts.clone();
                header.next_header = exts.set_next_headers(post_header);
                assert_eq!(
                    Ok(post_header),
                    IpHeader::Version6(header, exts).next_header()
                );
            }
        }
    }

    // TODO set_next_headers

    proptest! {
        #[test]
        fn set_payload_len(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
            payload_len in 0usize..10
        ) {
            // ipv4 (with valid payload length)
            {
                let mut actual = IpHeader::Version4(
                    v4.clone(),
                    v4_exts.clone()
                );
                actual.set_payload_len(payload_len).unwrap();

                assert_eq!(
                    actual,
                    IpHeader::Version4(
                        {
                            let mut re = v4.clone();
                            re.set_payload_len(v4_exts.header_len() + payload_len).unwrap();
                            re
                        },
                        v4_exts.clone()
                    )
                );
            }
            // ipv6 (with valid payload length)
            {
                let mut actual = IpHeader::Version6(
                    v6.clone(),
                    v6_exts.clone()
                );
                actual.set_payload_len(payload_len).unwrap();

                assert_eq!(
                    actual,
                    IpHeader::Version6(
                        {
                            let mut re = v6.clone();
                            re.set_payload_length(v6_exts.header_len() + payload_len).unwrap();
                            re
                        },
                        v6_exts.clone()
                    )
                );
            }

            // v4 (with invalid size)
            {
                let mut actual = IpHeader::Version4(
                    v4.clone(),
                    v4_exts.clone()
                );
                assert!(actual.set_payload_len(usize::MAX).is_err());
            }

            // v6 (with invalid size)
            {
                let mut actual = IpHeader::Version6(
                    v6.clone(),
                    v6_exts.clone()
                );
                assert!(actual.set_payload_len(usize::MAX).is_err());
            }
        }
    }

    proptest! {
        #[test]
        fn is_fragmenting_payload(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any()
        ) {
            // ipv4
            assert_eq!(
                v4.is_fragmenting_payload(),
                IpHeader::Version4(v4.clone(), v4_exts.clone()).is_fragmenting_payload()
            );

            // ipv6
            assert_eq!(
                v6_exts.is_fragmenting_payload(),
                IpHeader::Version6(v6.clone(), v6_exts.clone()).is_fragmenting_payload()
            );
        }
    }
}
