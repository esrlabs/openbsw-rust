#![no_std]
pub use embedded_can::Frame as CanFrameTrait;
use embedded_can::{ExtendedId, StandardId};
// TODO: bindings from cpp2can should be separated into a different crate
// TODO: design proper rust interface for sending and receiving
pub(crate) mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    #![allow(unsafe_op_in_unsafe_fn)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

const EXTENDED_QUALIFIER_BIT_VALUE: u32 = 0x80000000;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct CanFrame {
    inner: bindings::can_CANFrame,
}

impl embedded_can::Frame for CanFrame {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        const MAX_PAYLOAD_LEN: usize = 8;
        let id: embedded_can::Id = id.into();
        let id = match id {
            embedded_can::Id::Standard(standard_id) => standard_id.as_raw() as u32,
            embedded_can::Id::Extended(extended_id) => {
                extended_id.as_raw() | EXTENDED_QUALIFIER_BIT_VALUE
            }
        };
        if data.len() > MAX_PAYLOAD_LEN {
            return None;
        }
        let mut payload = [0u8; MAX_PAYLOAD_LEN];
        for (i, b) in data.iter().enumerate() {
            payload[i] = *b;
        }
        Some(Self {
            inner: bindings::can_CANFrame {
                _id: id,
                _timestamp: 0,
                _payload: payload,
                _payloadLength: data.len() as u8,
            },
        })
    }

    fn new_remote(_id: impl Into<embedded_can::Id>, _dlc: usize) -> Option<Self> {
        unimplemented!()
    }

    fn is_extended(&self) -> bool {
        self.inner._id & EXTENDED_QUALIFIER_BIT_VALUE != 0
    }

    fn is_remote_frame(&self) -> bool {
        unimplemented!()
    }

    fn id(&self) -> embedded_can::Id {
        let id = self.inner._id;
        if id & EXTENDED_QUALIFIER_BIT_VALUE == 0 {
            embedded_can::Id::Standard(unsafe {
                StandardId::new_unchecked(self.inner._id as u16 & StandardId::MAX.as_raw())
            })
        } else {
            embedded_can::Id::Extended(unsafe {
                ExtendedId::new_unchecked(self.inner._id & ExtendedId::MAX.as_raw())
            })
        }
    }

    fn dlc(&self) -> usize {
        self.inner._payloadLength as usize
    }

    fn data(&self) -> &[u8] {
        &self.inner._payload[0..self.dlc()]
    }
}

#[cfg(test)]
mod test {
    use crate::{CanFrame, bindings};

    #[test]
    fn assert_same_size_frame_type() {
        assert_eq!(
            core::mem::size_of::<CanFrame>(),
            core::mem::size_of::<bindings::can_CANFrame>()
        )
    }
}
