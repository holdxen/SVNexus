use super::*;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ClientDifferenceSource {
    #[serde(rename_all = "camelCase")]
    Target {
        path1: String,
        revision1: Revision,
        path2: String,
        revision2: Revision,
    },
    #[serde(rename_all = "camelCase")]
    Peg {
        path: String,
        peg_revision: Revision,
        start_revision: Revision,
        end_revision: Revision,
    },
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClientDifferenceOptions {
    options: Option<Vec<String>>,
    source: ClientDifferenceSource,
    relate_to: Option<String>,
    depth: Depth,
    ignore_ancestry: bool,
    no_added: bool,
    no_deleted: bool,
    show_copies_as_adds: bool,
    ignore_content_type: bool,
    ignore_properties: bool,
    properties_only: bool,
    use_git_format: bool,
    pretty_print_merge_info: bool,
    header_encoding: String,
    changelists: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClientDifferenceResult {
    #[ts(type = "Uint8Array")]
    #[serde(with = "serde_bytes")]
    out: Vec<u8>,
    #[ts(type = "Uint8Array")]
    #[serde(with = "serde_bytes")]
    err: Vec<u8>,
}

impl Context {
    pub fn difference(
        &mut self,
        opts: ClientDifferenceOptions,
    ) -> error::Result<ClientDifferenceResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let options = opts
                .options
                .map(|p| pool.string_array(p.len(), p.iter()))
                .transpose()?
                .unwrap_or_default();

            let relate_to = opts
                .relate_to
                .as_ref()
                .map(|v| pool.string(v))
                .transpose()?
                .unwrap_or_default();

            let header_encoding = pool.string(opts.header_encoding)?;

            let changelists = opts
                .changelists
                .map(|p| pool.string_array(p.len(), p.iter()))
                .transpose()?
                .unwrap_or_default();

            let mut out = Stream::create(Default::default());
            let mut err = Stream::create(Default::default());

            let error = match opts.source {
                ClientDifferenceSource::Target {
                    path1,
                    revision1,
                    path2,
                    revision2,
                } => {
                    let path1 = pool.canonicalize_target(&path1)?;
                    let revision1 = pool.revision(revision1);
                    let path2 = pool.canonicalize_target(&path2)?;
                    let revision2 = pool.revision(revision2);
                    let error = ffi::svn_client_diff7(
                        options,
                        path1,
                        revision1,
                        path2,
                        revision2,
                        relate_to,
                        opts.depth.into(),
                        opts.ignore_ancestry.into(),
                        opts.no_added.into(),
                        opts.no_deleted.into(),
                        opts.show_copies_as_adds.into(),
                        opts.ignore_content_type.into(),
                        opts.ignore_properties.into(),
                        opts.properties_only.into(),
                        opts.use_git_format.into(),
                        opts.pretty_print_merge_info.into(),
                        header_encoding,
                        out.ptr(),
                        err.ptr(),
                        changelists,
                        self.ctx(),
                        pool.as_mut_ptr(),
                    );
                    error
                }
                ClientDifferenceSource::Peg {
                    path,
                    peg_revision,
                    start_revision,
                    end_revision,
                } => {
                    let path = pool.canonicalize_target(&path)?;
                    let peg_revision = pool.revision(peg_revision);
                    let start_revision = pool.revision(start_revision);
                    let end_revision = pool.revision(end_revision);

                    let error = ffi::svn_client_diff_peg7(
                        options,
                        path,
                        peg_revision,
                        start_revision,
                        end_revision,
                        relate_to,
                        opts.depth.into(),
                        opts.ignore_ancestry.into(),
                        opts.no_added.into(),
                        opts.no_deleted.into(),
                        opts.show_copies_as_adds.into(),
                        opts.ignore_content_type.into(),
                        opts.ignore_properties.into(),
                        opts.properties_only.into(),
                        opts.use_git_format.into(),
                        opts.pretty_print_merge_info.into(),
                        header_encoding,
                        out.ptr(),
                        err.ptr(),
                        changelists,
                        self.ctx(),
                        pool.as_mut_ptr(),
                    );
                    error
                }
            };

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let out = out.take_write_buffer();
            let err = err.take_write_buffer();

            Ok(ClientDifferenceResult { out, err })
        }
    }
}
