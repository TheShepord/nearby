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

// Whether the Bluetooth advertisement is Public, Random or Unspecified.
//https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.bluetoothaddresstype?view=winrt-22621
use windows::Devices::Bluetooth::BluetoothAddressType;

use crate::bluetooth::common::BleAddressKind;

// Convenience for converting from Windows API to crate API.
impl TryFrom<BluetoothAddressType> for BleAddressKind {
    type Error = anyhow::Error;

    fn try_from(kind: BluetoothAddressType) -> Result<Self, Self::Error> {
        match kind {
            BluetoothAddressType::Public => Ok(BleAddressKind::Public),
            BluetoothAddressType::Random => Ok(BleAddressKind::Random),
            BluetoothAddressType::Unspecified => Err(anyhow::anyhow!(
                "Attempting to construct `BleAddressKind` with device \
                advertising Unspecified address type."
            )),
            _ => Err(anyhow::anyhow!(
                "Attempting to construct `BleAddressKind` with device \
                advertising invalid address type ."
            )),
        }
    }
}

// Convenience for converting from crate API to Windows API.
impl From<BleAddressKind> for BluetoothAddressType {
    fn from(kind: BleAddressKind) -> Self {
        match kind {
            BleAddressKind::Public => BluetoothAddressType::Public,
            BleAddressKind::Random => BluetoothAddressType::Random,
        }
    }
}
