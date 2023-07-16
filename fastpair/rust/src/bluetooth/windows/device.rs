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

use async_trait::async_trait;
use tracing::warn;
use windows::{
    Devices::{
        Bluetooth::{
            // Enum describing the type of address (public, random, unspecified).
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.bluetoothaddresstype?view=winrt-22621
            BluetoothAddressType,
            // TODO
            BluetoothDevice,
            // Struct for interacting with and pairing to a discovered BLE device.
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.bluetoothledevice?view=winrt-22621
            BluetoothLEDevice,
        },
        Enumeration::{
            // TODO
            DeviceInformationCustomPairing,
            // TODO
            DevicePairingKinds,
            // TODO
            DevicePairingRequestedEventArgs,
            // https://learn.microsoft.com/en-us/uwp/api/windows.devices.enumeration.devicepairingresultstatus?view=winrt-22621
            DevicePairingResultStatus,
        },
    },
    Foundation::TypedEventHandler,
};

use crate::bluetooth::common::{Address, BleAddress, ClassicAddress, Device, ServiceData};

/// Concrete type implementing `Device`, used for Windows BLE.
pub struct BleDevice {
    inner: BluetoothLEDevice,
    addr: BleAddress,
    service_data: Vec<ServiceData<u16>>
}

/// Concrete type implementing `Device`, used for Windows Bluetooth Classic.
pub struct ClassicDevice {
    inner: BluetoothDevice,
    addr: ClassicAddress,
}

impl BleDevice {
    /// `BleDevice` constructor.
    pub async fn new(addr: BleAddress, service_data: Vec<ServiceData<u16>>) -> Result<Self, anyhow::Error> {
        let kind = BluetoothAddressType::from(addr.get_kind());
        let raw_addr = u64::from(addr);

        let inner = BluetoothLEDevice::FromBluetoothAddressWithBluetoothAddressTypeAsync(
            raw_addr, kind,
        )?
        .await?;

        Ok(BleDevice { inner, addr, service_data })
    }
}

#[async_trait]
impl Device for BleDevice {
    fn name(&self) -> Result<String, anyhow::Error> {
        Ok(self.inner.Name()?.to_string_lossy())
    }

    fn address(&self) -> Address {
        Address::Ble(self.addr)
    }

    async fn pair(&self) -> Result<(), anyhow::Error> {
        // BLE Audio isn't supported on Windows natively, so devices can pair
        // but don't playback. Might possibly work with UWP. Since the Classic
        // and BLE APIs are very similar, it might be possible to copy-paste
        // `ClassicDevice::pair` directly.
        unimplemented!("BLE Pairing is currently unsupported.")
    }

    fn get_service_data(&self) -> &Vec<ServiceData<u16>> {
        &self.service_data
    }
}


impl ClassicDevice {
    /// `ClassicDevice` constructor.
    pub async fn new(addr: ClassicAddress) -> Result<Self, anyhow::Error> {
        let raw_addr = u64::from(addr);

        let inner = BluetoothDevice::FromBluetoothAddressAsync(
            raw_addr,
        )?
        .await?;

        Ok(ClassicDevice { inner, addr })
    }
}

#[async_trait]
impl Device for ClassicDevice {
    fn name(&self) -> Result<String, anyhow::Error> {
        Ok(self.inner.Name()?.to_string_lossy())
    }

    fn address(&self) -> Address {
        Address::Classic(self.addr)
    }

    async fn pair(&self) -> Result<(), anyhow::Error> {
        let pair_info = self.inner.DeviceInformation()?.Pairing()?;
        if pair_info.IsPaired()? {
            println!("Device already paired");
            Ok(())
        } else if !pair_info.CanPair()? {
            println!("Device can't pair");
            Ok(())
        } else {  
            let custom = pair_info.Custom()?;
            custom.PairingRequested(&TypedEventHandler::new(
                |_custom: &Option<DeviceInformationCustomPairing>, 
                event_args: &Option<DevicePairingRequestedEventArgs>,
                |  {
                    if let Some(event_args) = event_args {
                        match event_args.PairingKind()? {
                            DevicePairingKinds::ConfirmOnly => {
                                event_args.Accept()                            
                            }
                            _ => {
                                warn!("Unsupported pairing kind {:?}", event_args.PairingKind());
                                Ok(())
                            }
                        }
                    } else {
                        warn!("Empty pairing event arguments");
                        Ok(())
                    }

                },
            ))?;
            let res = custom
                .PairAsync(
                    DevicePairingKinds::ConfirmOnly
                        | DevicePairingKinds::ProvidePin
                        | DevicePairingKinds::ConfirmPinMatch
                        | DevicePairingKinds::DisplayPin,
                )?
                .await?;
            let status = res.Status()?;

            match status {
                DevicePairingResultStatus::Paired
                | DevicePairingResultStatus::AlreadyPaired => {
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Error while pairing: {:?}", status)),
            }
        }
    }

    fn get_service_data(&self) -> &Vec<ServiceData<u16>> {
        unimplemented!("Service data is currently unsupported for Classic devices.")
    }
}

mod tests {
    use super::*;

    // TODO b/288592509 unit tests
}
