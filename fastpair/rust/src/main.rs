// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    collections::HashSet,
    io::{self, Write},
    sync::Arc,
    thread,
};

use futures::{
    executor::{self, block_on},
    lock::Mutex,
};

mod bluetooth;

use bluetooth::{Adapter, Address, BleAdapter, ClassicAddress, Device};

async fn get_user_input(
    device_vec: Arc<Mutex<Vec<<BleAdapter as Adapter>::Device>>>,
) -> Result<(), anyhow::Error> {
    let mut buffer = String::new();
    loop {
        io::stdout().flush()?;
        buffer.clear();
        io::stdin().read_line(&mut buffer)?;

        let val = match buffer.trim().parse::<usize>() {
            Ok(val) => val,
            Err(_) => {
                println!("Please enter a valid digit.");
                continue;
            }
        };

        let index_to_device = device_vec.lock().await;
        match index_to_device.get(val) {
            Some(device) => {
                let addr = device.address();

                // Dynamic dispatch is necessary here because `BleDevice` and
                // `ClassicDevice` share the `Device` trait (and thus must have
                // the same return type for `address()` method). This can be
                // changed later if `Device` trait should exclusively define
                // shared cross-platform behavior.
                let classic_addr = match addr {
                    Address::Ble(ble) => ClassicAddress::try_from(ble),
                    Address::Classic(_) => unreachable!(
                        "Address should come from BLE Device, therefore \
                                    shouldn't be Classic."
                    ),
                }?;

                let classic_device =
                    bluetooth::new_classic_device(classic_addr).await?;

                match classic_device.pair().await {
                    Ok(_) => {
                        println!("Pairing success!");
                    }
                    Err(err) => println!("Error {}", err),
                }
            }
            None => println!("Please enter a valid digit."),
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let run = async {
        let mut adapter = bluetooth::default_adapter().await?;
        adapter.start_scan()?;

        let mut addr_set = HashSet::new();
        let device_vec = Arc::new(Mutex::new(Vec::new()));

        {
            // Process user input in a separate thread.
            let device_vec = device_vec.clone();
            thread::spawn(|| block_on(get_user_input(device_vec)));
        }

        let mut counter: u32 = 0;

        // Retrieve incoming device advertisements.
        while let Ok(ble_device) = adapter.next_device().await {
            for service_data in ble_device.get_service_data() {
                let uuid = service_data.get_uuid();

                // This is a Fast Pair device.
                if uuid == 0x2cfe {
                    let addr: Address = ble_device.address();
                    let name = ble_device.name()?;

                    if addr_set.insert(addr) {
                        // New FP device discovered.
                        println!("{}: {}", counter, name);
                        device_vec.lock().await.push(ble_device);
                        counter += 1;
                    }
                    break;
                }
            }
        }
        println!("Done scanning");
        Ok(())
    };

    executor::block_on(run)
}
