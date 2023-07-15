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

pub enum BleDataSection {
    ServiceData16BitUUid = 0x16,
}

pub struct ServiceData<Uuid: Copy> {
    uuid: Uuid,
    data: Vec<u8>,
}

impl<Uuid: Copy> ServiceData<Uuid> {
    pub fn new(uuid: Uuid, data: Vec<u8>) -> Self {
        ServiceData { uuid, data }
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}
