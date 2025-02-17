//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use crate::{RCodec, WCodec, Zenoh060};
use zenoh_buffers::{
    reader::{DidntRead, Reader},
    writer::{DidntWrite, Writer},
    ZSlice,
};

#[cfg(feature = "shared-memory")]
pub(crate) mod kind {
    pub(crate) const RAW: u8 = 0;
    pub(crate) const SHM_INFO: u8 = 1;
}

impl<W> WCodec<&ZSlice, &mut W> for Zenoh060
where
    W: Writer,
{
    type Output = Result<(), DidntWrite>;

    fn write(self, writer: &mut W, x: &ZSlice) -> Self::Output {
        self.write(&mut *writer, x.len())?;
        writer.write_zslice(x)?;
        Ok(())
    }
}

impl<R> RCodec<ZSlice, &mut R> for Zenoh060
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<ZSlice, Self::Error> {
        let len: usize = self.read(&mut *reader)?;
        let zslice = reader.read_zslice(len)?;
        Ok(zslice)
    }
}
