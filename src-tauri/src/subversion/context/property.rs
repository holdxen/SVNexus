use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PropertyGetOptions {
    property_name: String,
    target: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    inherited: bool,
    actual_revision: bool,
    changelists: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PropertyGetResult {
    properties: HashMap<String, String>,
    inherited_properties: Option<Vec<InheritedProperty>>,
    #[ts(type = "number | null")]
    actual_revision: Option<RevisionNumber>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RevisionPropertyListOptions {
    url: String,
    revision: Revision,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RevisionPropertyListResult {
    properties: HashMap<String, String>,
    #[ts(type = "number")]
    revison: RevisionNumber,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PropertyListOptions {
    target: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    changelists: Option<Vec<String>>,
    inherited: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PropertyListResult {
    entries: Vec<PropertyListEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PropertyListEntry {
    path: String,
    properties: Option<HashMap<String, String>>,
    inherited_properties: Option<Vec<InheritedProperty>>,
}

#[derive(Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InheritedProperty {
    path: String,
    properties: HashMap<String, String>,
}

impl From<*const ffi::svn_prop_inherited_item_t> for InheritedProperty {
    fn from(value: *const ffi::svn_prop_inherited_item_t) -> Self {
        unsafe {
            let value = value.as_ref().expect("Failed to get reference to value");
            let mut pool = apr::Pool::create();

            let path = value.path_or_url.to_str().to_string();
            let properties = pool.convert_to_hash_map(value.prop_hash);

            Self { path, properties }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PropertySetOptions {
    #[serde(rename_all = "camelCase")]
    Local {
        name: String,
        value: Option<String>,
        targets: Vec<String>,
        depth: Depth,
        skip_checks: bool,
        changelists: Option<Vec<String>>,
    },
    #[serde(rename_all = "camelCase")]
    Remote {
        name: String,
        value: Option<String>,
        url: String,
        skip_checks: bool,
        #[ts(type = "number")]
        base_revision_for_url: RevisionNumber,
        revision_properties: Option<HashMap<String, String>>,
        commit_message: String,
    },
}

impl Context {
    pub fn revision_property_list(
        &mut self,
        opts: RevisionPropertyListOptions,
    ) -> error::Result<RevisionPropertyListResult> {
        unsafe {
            let mut pool = apr::Pool::create();
            let mut hash: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let url = pool.string(opts.url)?;

            let revision = opts.revision.to_opt_revision();

            let mut revision_number: ffi::svn_revnum_t = 0;

            let error = ffi::svn_client_revprop_list(
                hash.pointer_mut(),
                url,
                revision.pointer(),
                revision_number.pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(RevisionPropertyListResult {
                revison: revision_number
                    .try_into()
                    .expect("Failed to convert revision number"),
                properties: pool.convert_to_hash_map(hash),
            })
        }
    }

    pub fn property_list(
        &mut self,
        opts: PropertyListOptions,
    ) -> error::Result<PropertyListResult> {
        unsafe extern "C" fn receiver(
            baton: *mut c_void,
            path: *const c_char,
            prop_hash: *mut ffi::apr_hash_t,
            inherited_props: *mut ffi::apr_array_header_t,
            scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let result = (baton as *mut PropertyListResult)
                    .as_mut()
                    .expect("Failed to cast baton to mutable reference");
                let path = path.to_str().to_string();
                let properties = prop_hash.map_mut(|hash| scratch_pool.convert_to_hash_map(hash));
                let inherited_properties =
                    inherited_props.map(|e| e.to_vec(|e| InheritedProperty::from(e as *const _)));

                result.entries.push(PropertyListEntry {
                    path,
                    properties,
                    inherited_properties,
                });
            }

            svn_no_error()
        }
        let mut result = PropertyListResult::default();
        unsafe {
            let mut pool = apr::Pool::create();
            let target = pool.canonicalize_target(&opts.target)?;

            let peg_revision = opts.peg_revision.to_opt_revision();
            let revision = opts.revision.to_opt_revision();

            let changelists = opts
                .changelists
                .as_ref()
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_proplist4(
                target,
                peg_revision.pointer(),
                revision.pointer(),
                opts.depth.into(),
                changelists,
                opts.inherited.into(),
                Some(receiver),
                result.pointer_mut() as _,
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
            Ok(result)
        }
    }

    pub fn property_get(&mut self, opts: PropertyGetOptions) -> error::Result<PropertyGetResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let mut inherited_properties: *mut ffi::apr_array_header_t = std::ptr::null_mut();

            let mut properties: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let property_name = pool.string(opts.property_name.as_str())?;

            let target = pool.canonicalize_target(&opts.target)?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            let mut actual_revision: ffi::svn_revnum_t = 0;

            let changelist = opts
                .changelists
                .map(|v| pool.string_array(v.len(), v.iter()))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_propget5(
                properties.pointer_mut(),
                if opts.inherited {
                    inherited_properties.pointer_mut()
                } else {
                    std::ptr::null_mut()
                },
                property_name,
                target,
                peg_revision.pointer(),
                revision.pointer(),
                if opts.actual_revision {
                    actual_revision.pointer_mut()
                } else {
                    std::ptr::null_mut()
                },
                opts.depth.into(),
                changelist,
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let inherited_properties = inherited_properties.map(|v| {
                v.to_vec(|p| InheritedProperty::from(p as *const ffi::svn_prop_inherited_item_t))
            });

            let actual_revision = if opts.actual_revision {
                Some(
                    actual_revision
                        .try_into()
                        .expect("Failed to convert revision number"),
                )
            } else {
                None
            };

            let properties = if properties.is_null() {
                tracing::info!("Unexpected null properties, treat as empty");
                Default::default()
            } else {
                pool.convert_to_hash_map(properties)
            };

            let result = PropertyGetResult {
                properties,
                inherited_properties,
                actual_revision,
            };

            Ok(result)
        }
    }

    pub fn property_set(&mut self, opts: PropertySetOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();

            match opts {
                PropertySetOptions::Local {
                    name,
                    value,
                    targets,
                    depth,
                    skip_checks,
                    changelists,
                } => {
                    let name = pool.string(name)?;
                    let value = value
                        .map(|e| {
                            ffi::svn_string_ncreate(
                                e.as_ptr() as _,
                                e.len().try_into().expect("Failed to convert size"),
                                pool.as_mut_ptr(),
                            )
                        })
                        .unwrap_or_default();
                    let targets = pool.canonicalize_dirent_array(targets.len(), targets.iter())?;
                    let changelists = changelists
                        .map(|v| pool.string_array(v.len(), v.iter()))
                        .transpose()?
                        .unwrap_or_default();

                    let error = ffi::svn_client_propset_local(
                        name,
                        value,
                        targets,
                        depth.into(),
                        skip_checks.into(),
                        changelists,
                        self.ctx(),
                        pool.as_mut_ptr(),
                    );
                    SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
                }
                PropertySetOptions::Remote {
                    name,
                    value,
                    url,
                    skip_checks,
                    base_revision_for_url,
                    revision_properties,
                    commit_message,
                } => {
                    let name = pool.string(name)?;
                    let value = value
                        .map(|e| pool.svn_string(e))
                        .transpose()?
                        .unwrap_or_default();
                    let url = pool.canonicalize_uri(&url)?;

                    let revision_properties = revision_properties
                        .map(|e| {
                            pool.string_hash_map(
                                e.iter(),
                                |p: &mut apr::Pool, k: &str| p.string(k).map(|o| o as _),
                                |p: &mut apr::Pool, v: &str| p.svn_string(v).map(|o| o as _),
                            )
                        })
                        .transpose()?
                        .unwrap_or_default();

                    self.inner.commit_message = commit_message;

                    let error = ffi::svn_client_propset_remote(
                        name,
                        value,
                        url,
                        skip_checks.into(),
                        base_revision_for_url
                            .try_into()
                            .expect("Failed to convert revision number"),
                        revision_properties,
                        Some(commit_callback),
                        self.inner.inner_void_pointer_mut(),
                        self.ctx(),
                        pool.as_mut_ptr(),
                    );
                    SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
                }
            }
        }
        Ok(())
    }
}
