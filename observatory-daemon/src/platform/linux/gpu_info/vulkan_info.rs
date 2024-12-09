/* sys_info_v2/observatory-daemon/src/platform/linux/gpu_info/vulkan_info.rs
 *
 * Copyright 2023 Romeo Calota
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pub struct VulkanInfo {
    _entry: ash::Entry,
    vk_instance: ash::Instance,
}

impl Drop for VulkanInfo {
    fn drop(&mut self) {
        unsafe {
            self.vk_instance.destroy_instance(None);
        }
    }
}

impl VulkanInfo {
    pub fn new() -> Option<Self> {
        use crate::{critical, debug};
        use ash::{vk, Entry};

        let _entry = match unsafe { Entry::load() } {
            Ok(e) => e,
            Err(e) => {
                critical!(
                    "Gatherer::GPU",
                    "Failed to get Vulkan information: Could not load 'libvulkan.so.1'; {}",
                    e
                );
                return None;
            }
        };
        debug!("Gatherer::VkInfo", "Loaded Vulkan library");

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };

        let instance = match unsafe { _entry.create_instance(&create_info, None) } {
            Ok(i) => i,
            Err(e) => {
                critical!(
                    "Gatherer::GPU",
                    "Failed to get Vulkan information: Could not create instance; {}",
                    e
                );
                return None;
            }
        };
        debug!("Gatherer::VkInfo", "Created Vulkan instance");

        Some(Self {
            _entry: _entry,
            vk_instance: instance,
        })
    }

    pub unsafe fn supported_vulkan_versions(
        &self,
    ) -> Option<std::collections::HashMap<u32, crate::platform::ApiVersion>> {
        use crate::{debug, platform::ApiVersion, warning};

        let physical_devices = match self.vk_instance.enumerate_physical_devices() {
            Ok(pd) => pd,
            Err(e) => {
                warning!(
                    "Gatherer::GPU",
                    "Failed to get Vulkan information: No Vulkan capable devices found ({})",
                    e
                );
                vec![]
            }
        };

        let mut supported_versions = std::collections::HashMap::new();

        for device in physical_devices {
            let properties = self.vk_instance.get_physical_device_properties(device);
            debug!(
                "Gatherer::GPU",
                "Found Vulkan device: {:?}",
                std::ffi::CStr::from_ptr(properties.device_name.as_ptr())
            );

            let version = properties.api_version;
            let major = (version >> 22) as u16;
            let minor = ((version >> 12) & 0x3ff) as u16;
            let patch = (version & 0xfff) as u16;

            let vendor_id = properties.vendor_id & 0xffff;
            let device_id = properties.device_id & 0xffff;

            supported_versions.insert(
                (vendor_id << 16) | device_id,
                ApiVersion {
                    major,
                    minor,
                    patch,
                },
            );
        }

        Some(supported_versions)
    }
}
