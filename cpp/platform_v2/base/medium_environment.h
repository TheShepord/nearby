#ifndef PLATFORM_V2_BASE_MEDIUM_ENVIRONMENT_H_
#define PLATFORM_V2_BASE_MEDIUM_ENVIRONMENT_H_

#include <atomic>

#include "platform_v2/api/bluetooth_adapter.h"
#include "platform_v2/api/bluetooth_classic.h"
#include "platform_v2/api/webrtc.h"
#include "platform_v2/base/byte_array.h"
#include "platform_v2/base/listeners.h"
#include "platform_v2/public/single_thread_executor.h"
#include "absl/container/flat_hash_map.h"
#include "absl/strings/string_view.h"

namespace location {
namespace nearby {

// MediumEnvironment is a simulated environment which allows multiple instances
// of simulated HW devices to "work" together as if they are physical.
// For each medium type it provides necessary methods to implement
// advertising, discovery and establishment of a data link.
// NOTE: this code depends on public:types target.
class MediumEnvironment {
 public:
  using BluetoothDiscoveryCallback =
      api::BluetoothClassicMedium::DiscoveryCallback;
  using OnSignalingMessageCallback =
      api::WebRtcSignalingMessenger::OnSignalingMessageCallback;
  using WifiLanDiscoveredServiceCallback =
      api::WifiLanMedium::DiscoveredServiceCallback;
  using WifiLanAcceptedConnectionCallback =
      api::WifiLanMedium::AcceptedConnectionCallback;
  MediumEnvironment(const MediumEnvironment&) = delete;
  MediumEnvironment& operator=(const MediumEnvironment&) = delete;

  // Creates and returns a reference to the global test environment instance.
  static MediumEnvironment& Instance();

  // Global ON/OFF switch for medium environment.
  // Start & Stop work as On/Off switch for this object.
  // Default state (after creation) is ON, to make it compatible with early
  // tests that are already using it and relying on it being ON.

  // Enables Medium environment.
  void Start();
  // Disables Medium environment.
  void Stop();

  // Clears state. No notifications are sent.
  void Reset();

  // Waits for all previously scheduled jobs to finish.
  // This method works as a barrier that guarantees that after it returns, all
  // the activities that started before it was called, or while it was running
  // are ended. This means that system is at the state of relaxation when this
  // code returns. It requires external stimulus to get out of relaxation state.
  //
  // If enable_notifications is true (default), simulation environment
  // will send all future notification events to all registered objects,
  // whenever protocol requires that. This is expected behavior.
  // If enabled_notifications is false, future event notifications will not be
  // sent to registered instances. This is useful for protocol shutdown,
  // where we no longer care about notifications, and where notifications may
  // otherwise be delivered after the notification source or target lifeteme has
  // ended, and cause undefined behavior.
  void Sync(bool enable_notifications = true);

  // Adds an adapter to internal container.
  // Notify BluetoothClassicMediums if any that adapter state has changed.
  void OnBluetoothAdapterChangedState(api::BluetoothAdapter& adapter,
                                      api::BluetoothDevice& adapter_device,
                                      std::string name, bool enabled,
                                      api::BluetoothAdapter::ScanMode mode);

  // Adds medium-related info to allow for adapter discovery to work.
  // This provides acccess to this medium from other mediums, when protocol
  // expects they should communicate.
  void RegisterBluetoothMedium(api::BluetoothClassicMedium& medium,
                               api::BluetoothAdapter& medium_adapter);

  // Updates callback info to allow for dispatch of discovery events.
  //
  // Invokes callback asynchronously when any changes happen to discoverable
  // devices, or if the defice is turned off, whether or not it is discoverable,
  // if it was ever reported as discoverable.
  //
  // This should be called when discoverable state changes.
  // with user-specified callback when discovery is enabled, and with default
  // (empty) callback otherwise.
  void UpdateBluetoothMedium(api::BluetoothClassicMedium& medium,
                             BluetoothDiscoveryCallback callback);

  // Removes medium-related info. This should correspond to device power off.
  void UnregisterBluetoothMedium(api::BluetoothClassicMedium& medium);

  // Registers |callback| to receive messages sent to device with id |self_id|.
  void RegisterWebRtcSignalingMessenger(absl::string_view self_id,
                                        OnSignalingMessageCallback callback);

  // Unregisters the callback listening to incoming messages for |self_id|.
  void UnregisterWebRtcSignalingMessenger(absl::string_view self_id);

  // Simulates sending a signaling message |message| to device with id
  // |peer_id|.
  void SendWebRtcSignalingMessage(absl::string_view peer_id,
                                  const ByteArray& message);
  // Adds medium-related info to allow for discovery/advertising to work.
  // This provides acccess to this medium from other mediums, when protocol
  // expects they should communicate.
  void RegisterWifiLanMedium(api::WifiLanMedium& medium,
                             api::WifiLanService& service);

  // Updates advertising info to indicate the current medium is exposing
  // advertising event.
  void UpdateWifiLanMediumForAdvertising(
      api::WifiLanMedium& medium, api::WifiLanService& service,
      const std::string& service_id, bool enabled);

  // Updates discovery callback info to allow for dispatch of discovery events.
  //
  // Invokes callback asynchronously when any changes happen to discoverable
  // devices, or if the defice is turned off, whether or not it is discoverable,
  // if it was ever reported as discoverable.
  //
  // This should be called when discoverable state changes.
  // with user-specified callback when discovery is enabled, and with default
  // (empty) callback otherwise.
  void UpdateWifiLanMediumForDiscovery(
      api::WifiLanMedium& medium, api::WifiLanService& service,
      const std::string& service_id,
      WifiLanDiscoveredServiceCallback discovery_callback, bool enabled);

  // Updates Accepted connection callback info to allow for dispatch of
  // advertising events.
  void UpdateWifiLanMediumForAcceptedConnection(
      api::WifiLanMedium& medium, api::WifiLanService& service,
      const std::string& service_id,
      WifiLanAcceptedConnectionCallback accepted_connection_callback);

  // Removes medium-related info. This should correspond to device power off.
  void UnregisterWifiLanMedium(api::WifiLanMedium& medium);

  // Call back when advertising has created the server socket and is ready for
  // connect.
  void CallWifiLanAcceptedConnectionCallback(api::WifiLanMedium& medium,
                                             api::WifiLanSocket& socket,
                                             const std::string& service_id);

 private:
  struct BluetoothMediumContext {
    BluetoothDiscoveryCallback callback;
    api::BluetoothAdapter* adapter = nullptr;
    // discovered device vs device name map.
    absl::flat_hash_map<api::BluetoothDevice*, std::string> devices;
  };

  struct WifiLanMediumContext {
    WifiLanDiscoveredServiceCallback discovery_callback;
    WifiLanAcceptedConnectionCallback accepted_connection_callback;
    api::WifiLanService* service = nullptr;
    bool advertising = false;
  };

  // This is a singleton object, for which destructor will never be called.
  // Constructor will be invoked once from Instance() static method.
  // Object is create in-place (with a placement new) to guarantee that
  // destructor is not scheduled for execution at exit.
  MediumEnvironment() = default;
  ~MediumEnvironment() = default;

  void OnBluetoothDeviceStateChanged(BluetoothMediumContext& info,
                                     api::BluetoothDevice& device,
                                     const std::string& name,
                                     api::BluetoothAdapter::ScanMode mode,
                                     bool enabled);

  void OnWifiLanServiceStateChanged(WifiLanMediumContext& info,
                                    api::WifiLanService& service,
                                    const std::string& service_id,
                                    bool enabled);

  void RunOnMediumEnvironmentThread(std::function<void()> runnable);

  std::atomic_bool enabled_ = true;
  std::atomic_int job_count_ = 0;
  std::atomic_bool enable_notifications_ = false;
  SingleThreadExecutor executor_;

  // The following data members are accessed in the context of a private
  // executor_ thread.
  absl::flat_hash_map<api::BluetoothAdapter*, api::BluetoothDevice*>
      bluetooth_adapters_;
  absl::flat_hash_map<api::BluetoothClassicMedium*, BluetoothMediumContext>
      bluetooth_mediums_;

  // Maps peer id to callback for receiving signaling messages.
  absl::flat_hash_map<std::string, OnSignalingMessageCallback>
      webrtc_signaling_callback_;

  absl::flat_hash_map<api::WifiLanMedium*, WifiLanMediumContext>
      wifi_lan_mediums_;
};

}  // namespace nearby
}  // namespace location

#endif  // PLATFORM_V2_BASE_MEDIUM_ENVIRONMENT_H_
