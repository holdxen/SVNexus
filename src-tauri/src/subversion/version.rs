use crate::{apr, utils::CStringer};
use derive_new::new;
use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use ts_rs::TS;

use super::ffi;

#[derive(new, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Version {
    major: i32,
    minor: i32,
    patch: i32,
    tag: Option<String>,
}

// impl Version {
//     pub fn compare(&self, other: &Self) -> i32 {
//         if self.major > other.major {
//             return 1;
//         } else if self.major < other.major {
//             return -1;
//         }
//         if self.minor > other.minor {
//             return 1;
//         } else if self.minor < other.minor {
//             return -1;
//         }
//         if self.patch > other.patch {
//             return 1;
//         } else if self.patch < other.patch {
//             return -1;
//         }
//         0
//     }
// }

impl From<*const ffi::svn_version_t> for Version {
    fn from(value: *const ffi::svn_version_t) -> Self {
        unsafe {
            let value = value.as_ref().expect("Failed to get reference to value");
            Self {
                major: value
                    .major
                    .try_into()
                    .expect("Failed to convert version number"),
                minor: value
                    .minor
                    .try_into()
                    .expect("Failed to convert version number"),
                patch: value
                    .patch
                    .try_into()
                    .expect("Failed to convert version number"),
                tag: value.tag.to_nullable_string(),
            }
        }
    }
}

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LinkedLibrary {
    name: String,
    compiled_version: String,
    runtime_version: Option<String>,
}

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoadedLibrary {
    name: String,
    version: String,
}

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExtendedVersion {
    version: Version,
    build_date: String,
    build_time: String,
    build_host: String,
    copyright: String,
    runtime_host: String,
    runtime_os_name: String,
    linked_libraries: Vec<LinkedLibrary>,
    loaded_libraries: Vec<LoadedLibrary>,
}

pub fn extended_version(verbose: bool) -> ExtendedVersion {
    unsafe {
        let mut pool = apr::Pool::create();
        let v = ffi::svn_version_extended(verbose.into(), pool.as_mut_ptr());

        let build_date = CStr::from_ptr(ffi::svn_version_ext_build_date(v))
            .to_str()
            .expect("Unexpected failure")
            .to_string();

        let build_time = ffi::svn_version_ext_build_time(v).to_str().to_string();

        let build_host = ffi::svn_version_ext_build_host(v).to_str().to_string();

        let copyright = CStr::from_ptr(ffi::svn_version_ext_copyright(v))
            .to_str()
            .expect("Unexpected failure")
            .to_string();

        let runtime_host_ptr = ffi::svn_version_ext_runtime_host(v);
        let runtime_host = if runtime_host_ptr.is_null() {
            String::new()
        } else {
            CStr::from_ptr(runtime_host_ptr)
                .to_str()
                .expect("Unexpected failure")
                .to_string()
        };

        let runtime_os_name_ptr = ffi::svn_version_ext_runtime_osname(v);
        let runtime_os_name = if runtime_os_name_ptr.is_null() {
            String::new()
        } else {
            CStr::from_ptr(runtime_os_name_ptr)
                .to_str()
                .expect("Unexpected failure")
                .to_string()
        };

        let linked_ptr = ffi::svn_version_ext_linked_libs(v);
        let linked_libraries = if linked_ptr.is_null() {
            Vec::new()
        } else {
            let linked = &*linked_ptr;
            let mut libs =
                Vec::with_capacity(linked.nelts.try_into().expect("Failed to convert size"));

            let elements = linked.elts as *const ffi::svn_version_ext_linked_lib_t;
            for i in 0..linked.nelts as usize {
                let element = &*elements.add(i);
                let name = CStr::from_ptr(element.name)
                    .to_str()
                    .expect("Invalid UTF-8 string")
                    .to_string();
                let runtime_version = if element.runtime_version.is_null() {
                    None
                } else {
                    Some(
                        CStr::from_ptr(element.runtime_version)
                            .to_str()
                            .expect("Unexpected failure")
                            .to_string(),
                    )
                };
                let compiled_version = CStr::from_ptr(element.compiled_version)
                    .to_str()
                    .expect("Unexpected failure")
                    .to_string();

                libs.push(LinkedLibrary::new(name, compiled_version, runtime_version));
            }
            libs
        };

        let loaded_ptr = ffi::svn_version_ext_loaded_libs(v);
        let loaded_libraries = if loaded_ptr.is_null() {
            Vec::new()
        } else {
            let loaded = &*loaded_ptr;
            let mut libs =
                Vec::with_capacity(loaded.nelts.try_into().expect("Failed to convert size"));

            let elements = loaded.elts as *const ffi::svn_version_ext_loaded_lib_t;
            for i in 0..loaded.nelts as usize {
                let element = &*elements.add(i);
                let name = CStr::from_ptr(element.name)
                    .to_str()
                    .expect("Invalid UTF-8 string")
                    .to_string();
                let version = if element.version.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr(element.version)
                        .to_str()
                        .expect("Unexpected failure")
                        .to_string()
                };

                libs.push(LoadedLibrary::new(name, version));
            }
            libs
        };

        let version = ffi::svn_client_version();

        let version = Version::from(version);

        ExtendedVersion::new(
            version,
            build_date,
            build_time,
            build_host,
            copyright,
            runtime_host,
            runtime_os_name,
            linked_libraries,
            loaded_libraries,
        )
    }
}
