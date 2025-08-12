//! Differential injector RPC calls

use crate::{
    platform::repl::rpc::{RpcCallSender, RpcResultReceiver},
    register_repl_fn_no_rpc,
};
use alloc::{borrow::ToOwned, boxed::Box};
use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use rhai::{Blob, Engine, EvalAltResult, Module, NativeCallContext, INT};

pub const SOF: bool = false;
pub const ID_A_LEN: usize = 11;
pub const ID_B_LEN: usize = 18;
pub const R0: bool = false;
pub const R1: bool = false;
pub const SRR: bool = true;
pub const DLC_LEN: usize = 4;
pub const CRC_LEN: usize = 15;
pub const CRC_DELIM: bool = true;
pub const ACK: bool = true;
pub const ACK_DELIM: bool = true;
pub const EOF_LEN: usize = 7;
pub const EOF: bool = true;

pub fn crc15(bits: &BitSlice<u8, Msb0>) -> u16 {
    const POLY: u16 = 0x4599;
    let mut crc: u16 = 0;

    for bit in bits {
        let msb = (crc >> 14) & 0x1;
        crc <<= 1;

        if msb ^ (*bit as u16) != 0 {
            crc ^= POLY;
        }

        crc &= 0x7FFF;
    }

    crc
}

pub fn stuff_bits(bits: &BitSlice<u8, Msb0>) -> BitVec<u8, Msb0> {
    let mut stuffed = BitVec::<u8, Msb0>::new();
    let mut run_len = 1;
    let mut prev_bit = bits[0];

    stuffed.push(prev_bit);

    for bit_ref in bits.iter().skip(1) {
        let bit = *bit_ref;

        if bit == prev_bit {
            run_len += 1;

            if run_len == 5 {
                stuffed.push(bit);
                stuffed.push(!bit);
                run_len = 0;
                continue;
            }
        } else {
            run_len = 1;
        }

        stuffed.push(bit);
        prev_bit = bit;
    }

    stuffed
}

pub(crate) fn repl_can_encode(
    ctx: &NativeCallContext,
    arb_id: INT,
    rtr: bool,
    payload: Blob,
) -> Result<Blob, Box<EvalAltResult>> {
    if payload.len() > 8 {
        return Err(Box::new(EvalAltResult::ErrorDataTooLarge(
            "CAN 2.0B payload must be <= 8 bytes.".to_owned(),
            ctx.call_position(),
        )));
    }

    if arb_id > 0x1FFF_FFF {
        return Err(Box::new(EvalAltResult::ErrorDataTooLarge(
            "CAN 2.0B identifier must be at most 29 bits.".to_owned(),
            ctx.call_position(),
        )));
    }

    let mut bits = BitVec::<u8, Msb0>::new();
    let ide = if arb_id > 0x7FF { true } else { false };
    let arb_id = arb_id as u32;
    let dlc = payload.len() as u8;

    bits.push(SOF);

    if ide {
        let id_a = (arb_id >> ID_B_LEN) & 0x7FF;
        let id_b = arb_id & 0x3_FFFF;

        for i in (0..ID_A_LEN).rev() {
            bits.push(((id_a >> i) & 0b1) == 1);
        }

        bits.push(SRR);
        bits.push(ide);

        for i in (0..ID_B_LEN).rev() {
            bits.push(((id_b >> i) & 0b1) == 1);
        }

        bits.push(rtr);
        bits.push(R1);
        bits.push(R0);
    } else {
        let id_a = (arb_id & 0x7FF) as u16;

        for i in (0..ID_A_LEN).rev() {
            bits.push(((id_a >> i) & 0b1) == 1);
        }

        bits.push(rtr);
        bits.push(ide);
        bits.push(R0);
    }

    for i in (0..DLC_LEN).rev() {
        bits.push(((dlc >> i) & 0b1) == 1);
    }

    for byte in payload {
        for i in (0..8).rev() {
            bits.push(((byte >> i) & 0b1) == 1);
        }
    }

    let crc = crc15(&bits);
    let mut crc_bits = BitVec::<u8, Msb0>::with_capacity(15);

    for i in (0..CRC_LEN).rev() {
        crc_bits.push(((crc >> i) & 0b1) == 1);
    }

    crc_bits.push(CRC_DELIM);
    bits.extend(crc_bits);
    let mut stuffed = stuff_bits(&bits);
    stuffed.push(ACK);
    stuffed.push(ACK_DELIM);

    for _ in 0..EOF_LEN {
        stuffed.push(EOF);
    }

    while stuffed.len() % 8 != 0 {
        stuffed.push(EOF);
    }

    let blob = stuffed.into_vec();

    Ok(blob)
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    _call_tx: RpcCallSender,
    _result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();
    register_repl_fn_no_rpc!(module, repl_can_encode, "encode", (arb_id: INT, rtr: bool, payload: Blob));
    engine.register_static_module("can", module.into());
}
