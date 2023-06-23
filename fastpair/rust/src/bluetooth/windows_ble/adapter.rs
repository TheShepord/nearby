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
use super::BleDevice;

use std::sync::Arc;

use futures::{stream::Stream, StreamExt};
use tracing::{error, warn};
use windows::{
    Devices::Bluetooth::{
        Advertisement::{
            // Struct that receives Bluetooth Low Energy (LE) advertisements.
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.advertisement.bluetoothleadvertisementwatcher?view=winrt-22621
            BluetoothLEAdvertisementReceivedEventArgs,

            // Enum describing the type of advertisement (whether it's connectable, directed, etc).
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.bluetoothaddresstype?view=winrt-22621
            BluetoothLEAdvertisementType,

            // Provides data for a Received event on a BluetoothLEAdvertisementWatcher. Instance is
            // created when the Received event occurs in the watcher struct.
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.advertisement.bluetoothleadvertisementreceivedeventargs?view=winrt-22621
            BluetoothLEAdvertisementWatcher,

            // Provides data for a Stopped event on a BluetoothLEAdvertisementWatcher. Instance is
            // created when the Stopped event occurs on a watcher struct.
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.advertisement.bluetoothleadvertisementwatcherstoppedeventargs?view=winrt-22621
            BluetoothLEAdvertisementWatcherStoppedEventArgs,

            // Defines constants that specify a Bluetooth LE scanning mode.
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.advertisement.bluetoothlescanningmode?view=winrt-22621
            BluetoothLEScanningMode,
        },
        // Struct for obtaining global constant information about a computer's Bluetooth adapter.
        // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.bluetoothadapter?view=winrt-22621
        BluetoothAdapter,
    },
    // Wraps a closure for handling events associated with a struct (e.g. Received and Stopped
    // events in BluetoothLEAdvertisementWatcher).
    // https://learn.microsoft.com/en-us/uwp/api/windows.foundation.typedeventhandler-2?view=winrt-22621
    Foundation::TypedEventHandler,
};

pub struct BleAdapter {
    inner: BluetoothAdapter,
}

impl BleAdapter {
    pub async fn default() -> Result<Self, anyhow::Error> {
        let inner = BluetoothAdapter::GetDefaultAsync()?.await?;

        if !inner.IsLowEnergySupported()? {
            return Err(anyhow::anyhow!(
                "This device's Bluetooth Adapter doesn't support Bluetooth LE Transport type."
            ));
        }
        if !inner.IsCentralRoleSupported()? {
            return Err(anyhow::anyhow!(
                "This device's Bluetooth Adapter doesn't support Bluetooth LE central role."
            ));
        }

        Ok(BleAdapter { inner })
    }

    pub fn scanner(&self) -> Result<impl Stream<Item = BleDevice>, anyhow::Error> {
        let watcher = BluetoothLEAdvertisementWatcher::new()?;
        watcher.SetScanningMode(BluetoothLEScanningMode::Active)?;

        if self.inner.IsExtendedAdvertisingSupported()? {
            watcher.SetAllowExtendedAdvertisements(true)?;
        }

        let (sender, receiver) = futures::channel::mpsc::channel(16);
        let sender = Arc::new(std::sync::Mutex::new(sender));

        // `received_handler` closure holds non-owning channel reference, to ensure
        // `stopped_handler` can close the channel when `received_handler` is done.
        let weak_sender = Arc::downgrade(&sender);
        let received_handler = TypedEventHandler::new(
            // Move `weak_sender` into closure.
            move |watcher: &Option<BluetoothLEAdvertisementWatcher>,
                  event_args: &Option<BluetoothLEAdvertisementReceivedEventArgs>| {
                if watcher.is_some() {
                    if let Some(event_args) = event_args {
                        if let Some(sender) = weak_sender.upgrade() {
                            match sender.lock().unwrap().try_send(event_args.clone()) {
                                Ok(_) => (),
                                Err(err) => {
                                    error!("Error while handling Received event: {:?}", err)
                                }
                            }
                        }
                    }
                }

                Ok(())
            },
        );

        // `stopped_handler` closure owns channel reference, so it can close the channel.
        let mut sender = Some(sender);
        let stopped_handler = TypedEventHandler::new(
            // Move `sender` into closure.
            move |_watcher, _event_args: &Option<BluetoothLEAdvertisementWatcherStoppedEventArgs>| {
                // Drop `sender`, closing the channel.
                let _sender = sender.take();
                println!("Watcher stopped receiving BLE advertisements.");
                Ok(())
            },
        );

        watcher.Received(&received_handler)?;
        watcher.Stopped(&stopped_handler)?;
        watcher.Start()?;

        // `receiver` is a `futures::channel::mpsc::Receiver`, which implements
        // `futures::stream::Stream`. This is essentially an async Iterator. We apply a FilterMap to
        // map from advertisement packet to a future returning `BleDevice` and filter out undesired
        // connections. We need a pinned box to satisfy trait bounds for `Stream`.
        Ok(Box::pin(receiver.filter_map(move |event_args| {
            //  Move `watcher` into `FilterMap` closure. This ensures `watcher` is only dropped when
            // the stream is closed.
            let _watcher = &watcher;

            // Move `event_args` into async block.
            async move {
                match event_args.AdvertisementType().ok()? {
                    BluetoothLEAdvertisementType::NonConnectableUndirected => None,
                    _ => {
                        let addr = event_args.BluetoothAddress().ok()?;
                        let kind = event_args.BluetoothAddressType().ok()?;

                        match BleDevice::from_addr(addr, kind).await {
                            Ok(device) => Some(device),
                            Err(err) => {
                                warn!("Error creating device: {:?}", err);
                                None
                            }
                        }
                    }
                }
            }
        })))
    }
}

mod tests {
    use super::*;

    // TODO b/288592509 unit tests
}
