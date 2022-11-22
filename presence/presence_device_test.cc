// Copyright 2020 Google LLC
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

#include "presence/presence_device.h"

#include "gmock/gmock.h"
#include "protobuf-matchers/protocol-buffer-matchers.h"
#include "gtest/gtest.h"
#include "internal/proto/device_metadata.proto.h"

namespace nearby {
namespace presence {
namespace {
constexpr DeviceMotion::MotionType kDefaultMotionType =
    DeviceMotion::MotionType::kPointAndHold;
constexpr float kDefaultConfidence = 0;
constexpr float kTestConfidence = 0.1;
constexpr absl::string_view kMacAddr = "\x4C\x8B\x1D\xCE\xBA\xD1";

DeviceMetadata CreateTestDeviceMetadata() {
  DeviceMetadata device_metadata;
  device_metadata.set_stable_device_id("test_device_id");
  device_metadata.set_account_name("test_account");
  device_metadata.set_device_name("NP test device");
  device_metadata.set_icon_url("test_image.test.com");
  device_metadata.set_bluetooth_mac_address(kMacAddr);
  device_metadata.set_device_type(internal::DeviceMetadata::PHONE);
  return device_metadata;
}

TEST(PresenceDeviceTest, DefaultMotionEquals) {
  DeviceMetadata metadata = CreateTestDeviceMetadata();
  PresenceDevice device1(metadata);
  PresenceDevice device2(metadata);
  EXPECT_EQ(device1, device2);
}

TEST(PresenceDeviceTest, ExplicitInitEquals) {
  DeviceMetadata metadata = CreateTestDeviceMetadata();
  PresenceDevice device1 =
      PresenceDevice({kDefaultMotionType, kTestConfidence}, metadata);
  PresenceDevice device2 =
      PresenceDevice({kDefaultMotionType, kTestConfidence}, metadata);
  EXPECT_EQ(device1, device2);
}

TEST(PresenceDeviceTest, ExplicitInitNotEquals) {
  DeviceMetadata metadata = CreateTestDeviceMetadata();
  PresenceDevice device1 = PresenceDevice({kDefaultMotionType}, metadata);
  PresenceDevice device2 =
      PresenceDevice({kDefaultMotionType, kTestConfidence}, metadata);
  EXPECT_NE(device1, device2);
}

TEST(PresenceDeviceTest, CopyInitEquals) {
  DeviceMetadata metadata = CreateTestDeviceMetadata();
  PresenceDevice device1 =
      PresenceDevice({kDefaultMotionType, kTestConfidence}, metadata);
  PresenceDevice device2 = {device1};
  EXPECT_EQ(device1, device2);
}

}  // namespace
}  // namespace presence
}  // namespace nearby
