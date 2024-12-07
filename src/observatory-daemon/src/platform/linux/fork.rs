/* sys_info_v2/observatory-daemon/src/platform/linux/fork.rs
 *
 * This file was originally part of the LACT project (https://github.com/ilya-zlobintsev/LACT)
 *
 * Copyright (c) 2023 Ilya Zlobintsev
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * SPDX-License-Identifier: MIT
 */

use std::{
    fmt::Debug,
    io::{BufReader, Read, Write},
    mem::size_of,
    os::unix::net::UnixStream,
};

use anyhow::{anyhow, Context};
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::debug;

pub unsafe fn run_forked<T, F>(f: F) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned + Debug,
    F: FnOnce() -> Result<T, String>,
{
    let (rx, mut tx) = UnixStream::pair()?;
    rx.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;
    tx.set_write_timeout(Some(std::time::Duration::from_secs(1)))?;
    let mut rx = BufReader::new(rx);

    match fork()? {
        ForkResult::Parent { child } => {
            debug!("Gatherer::Fork", "Waiting for message from child");

            let mut size_buf = [0u8; size_of::<usize>()];
            rx.read_exact(&mut size_buf)?;
            let size = usize::from_ne_bytes(size_buf);

            let mut data_buf = vec![0u8; size];
            rx.read_exact(&mut data_buf)?;

            debug!(
                "Gatherer::Fork",
                "Received {} data bytes from child",
                data_buf.len()
            );

            waitpid(child, None)?;

            let data: Result<T, String> = bincode::deserialize(&data_buf)
                .context("Could not deserialize response from child")?;

            data.map_err(|err| anyhow!("{err}"))
        }
        ForkResult::Child => {
            let response = f();
            debug!("Gatherer::Fork", "Sending response to parent: {response:?}");

            let send_result = (|| {
                let data = bincode::serialize(&response)?;
                tx.write_all(&data.len().to_ne_bytes())?;
                tx.write_all(&data)?;
                Ok::<_, anyhow::Error>(())
            })();

            let exit_code = match send_result {
                Ok(()) => 0,
                Err(_) => 1,
            };
            debug!("Gatherer::Fork", "Exiting child with code {exit_code}");
            std::process::exit(exit_code);
        }
    }
}
