use super::ffi;


fn cbytes_as_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(&bytes[0..bytes.len() - 1]).unwrap()
}

#[derive(Debug, uniffi::Enum, Clone, Copy, PartialEq, Eq)]
enum RevisionPropertyName {
    Author,
    Date,
    Log,
    OriginalDate,
    Autoversioned,
    Lock,
    FromUrl,
    FromUuid,
    LastMergeRevision,
    CurrentCopying
}


#[uniffi::export]
impl RevisionPropertyName {
    fn name(&self) -> String {
        self.as_str().to_string()
    }
}

impl RevisionPropertyName {
    fn as_str(&self) -> &'static str {
        match self {
            RevisionPropertyName::Author => cbytes_as_str(ffi::SVN_PROP_REVISION_AUTHOR),
            RevisionPropertyName::Date => cbytes_as_str(ffi::SVN_PROP_REVISION_DATE),
            RevisionPropertyName::Log => cbytes_as_str(ffi::SVN_PROP_REVISION_LOG),
            RevisionPropertyName::OriginalDate => cbytes_as_str(ffi::SVN_PROP_REVISION_ORIG_DATE),
            RevisionPropertyName::Autoversioned => cbytes_as_str(ffi::SVN_PROP_REVISION_AUTOVERSIONED),
            RevisionPropertyName::Lock => cbytes_as_str(ffi::SVN_PROP_REVISION_LOG),
            RevisionPropertyName::FromUrl => cbytes_as_str(ffi::SVNSYNC_PROP_FROM_URL),
            RevisionPropertyName::FromUuid => cbytes_as_str(ffi::SVNSYNC_PROP_FROM_UUID),
            RevisionPropertyName::LastMergeRevision => cbytes_as_str(ffi::SVNSYNC_PROP_LAST_MERGED_REV),
            RevisionPropertyName::CurrentCopying => cbytes_as_str(ffi::SVNSYNC_PROP_CURRENTLY_COPYING),
        }
    }
}


#[derive(Debug, uniffi::Enum, Clone, Copy, PartialEq, Eq)]
enum NodePropertyName {
    MimeType,
    Ignore,
    EolStyle,
    Keywords,
    Exeucable,
    NeedsLock,
    Special,
    Externals,
    MergeInfo,
    AutoProperties,
    GlobalIgnores,
    TextTime,
    Owner,
    Group,
    UnixMode,
}

#[uniffi::export]
impl NodePropertyName {
    fn name(&self) -> String {
        self.as_str().to_string()
    }
}

impl NodePropertyName {
    fn as_str(&self) -> &'static str {
        match self {
            NodePropertyName::MimeType => cbytes_as_str(ffi::SVN_PROP_MIME_TYPE),
            NodePropertyName::Ignore => cbytes_as_str(ffi::SVN_PROP_IGNORE),
            NodePropertyName::EolStyle => cbytes_as_str(ffi::SVN_PROP_EOL_STYLE),
            NodePropertyName::Keywords => cbytes_as_str(ffi::SVN_PROP_KEYWORDS),
            NodePropertyName::Exeucable => cbytes_as_str(ffi::SVN_PROP_EXECUTABLE),
            NodePropertyName::NeedsLock => cbytes_as_str(ffi::SVN_PROP_NEEDS_LOCK),
            NodePropertyName::Special => cbytes_as_str(ffi::SVN_PROP_SPECIAL),
            NodePropertyName::Externals => cbytes_as_str(ffi::SVN_PROP_EXTERNALS),
            NodePropertyName::MergeInfo => cbytes_as_str(ffi::SVN_PROP_MERGEINFO),
            NodePropertyName::AutoProperties => cbytes_as_str(ffi::SVN_PROP_INHERITABLE_AUTO_PROPS)    ,
            NodePropertyName::GlobalIgnores => cbytes_as_str(ffi::SVN_PROP_INHERITABLE_IGNORES),
            NodePropertyName::TextTime => cbytes_as_str(ffi::SVN_PROP_TEXT_TIME),
            NodePropertyName::Owner => cbytes_as_str(ffi::SVN_PROP_OWNER),
            NodePropertyName::Group => cbytes_as_str(ffi::SVN_PROP_GROUP),
            NodePropertyName::UnixMode => cbytes_as_str(ffi::SVN_PROP_UNIX_MODE),
        }
    }
}