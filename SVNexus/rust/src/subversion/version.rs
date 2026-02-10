use crate::apr;
use derive_new::new;
use std::ffi::CStr;

use super::ffi;

#[derive(new)]
pub struct Version {
    major: i32,
    minor: i32,
    patch: i32,
    tag: String,
}

#[derive(new)]
pub struct LinkedLibrary {
    name: String,
    compiled_version: String,
    runtime_versino: Option<String>,
}

#[derive(new)]
pub struct LoadedLibrary {
    name: String,
    version: String,
}

#[derive(new)]
pub struct ExtendedVersion {
    build_date: String,
    build_time: String,
    build_host: String,
    copyright: String,
    runtime_host: String,
    runtime_os_name: String,
    linked_libraries: Vec<LinkedLibrary>,
    loaded_libraries: Vec<LoadedLibrary>,
}

pub fn version() -> Version {
    unsafe {
        let v = ffi::svn_subr_version();

        let v = &*v;

        Version::new(
            v.major,
            v.minor,
            v.patch,
            apr::char_array_to_string(v.tag).unwrap(),
        )
    }
}

pub fn extended_version(verbose: bool) -> ExtendedVersion {
    unsafe {
        let mut pool = apr::Pool::create();
        let v = ffi::svn_version_extended(verbose.into(), pool.as_mut_ptr());

        let build_date = CStr::from_ptr(ffi::svn_version_ext_build_date(v))
            .to_str()
            .unwrap()
            .to_string();

        let build_time = apr::char_array_to_string(ffi::svn_version_ext_build_time(v)).unwrap();

        let build_host = apr::char_array_to_string(ffi::svn_version_ext_build_host(v)).unwrap();

        let copyright = CStr::from_ptr(ffi::svn_version_ext_copyright(v))
            .to_str()
            .unwrap()
            .to_string();

        let runtime_host = CStr::from_ptr(ffi::svn_version_ext_runtime_host(v))
            .to_str()
            .unwrap()
            .to_string();

        let runtime_os_name = CStr::from_ptr(ffi::svn_version_ext_runtime_osname(v))
            .to_str()
            .unwrap()
            .to_string();

        let linked = ffi::svn_version_ext_linked_libs(v);

        let loaded = ffi::svn_version_ext_loaded_libs(v);

        let linked = &*linked;

        let loaded = &*loaded;

        let mut linked_libraries = Vec::with_capacity(linked.nelts.try_into().unwrap());

        for i in 0..linked.nelts {
            let elements = linked.elts as *const *const ffi::svn_version_ext_linked_lib_t;
            let element = &**elements.offset(i.try_into().unwrap());
            let name = CStr::from_ptr(element.name).to_str().unwrap().to_string();
            let runtime_version = if element.runtime_version.is_null() {
                None
            } else {
                Some(
                    CStr::from_ptr(element.runtime_version)
                        .to_str()
                        .unwrap()
                        .to_string(),
                )
            };
            let compiled_version = CStr::from_ptr(element.compiled_version)
                .to_str()
                .unwrap()
                .to_string();

            linked_libraries.push(LinkedLibrary::new(name, compiled_version, runtime_version));
        }

        let mut loaded_libraries = Vec::with_capacity(loaded.nelts.try_into().unwrap());

        for i in 0..loaded.nelts {
            let elements = linked.elts as *const *const ffi::svn_version_ext_loaded_lib_t;
            let element = &**elements.offset(i.try_into().unwrap());
            let name = CStr::from_ptr(element.name).to_str().unwrap().to_string();
            let version = CStr::from_ptr(element.version)
                .to_str()
                .unwrap()
                .to_string();

            loaded_libraries.push(LoadedLibrary::new(name, version));
        }

        ExtendedVersion::new(
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
