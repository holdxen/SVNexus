pub mod context;
pub mod export;
mod property;
pub mod ra;
pub mod stream;
pub mod utils;
pub mod version;
pub mod wc;

#[cfg(test)]
mod tests;

use std::{ffi::CString, str::FromStr};

use crate::{
    apr,
    error::{self, builder},
    extensions::ResultExtension,
    utils::{CStringer, Pointer},
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[allow(bad_style)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[allow(unnecessary_transmutes)]
#[allow(unsafe_op_in_unsafe_fn)]
pub mod ffi {
    pub use crate::apr::ffi::*;
    include!(concat!(env!("OUT_DIR"), "/subversion.rs"));

    // subversion private api
    unsafe extern "C" {
        pub fn svn_config__get_default_config(
            cfg_hash: *mut *mut apr_hash_t,
            pool: *mut apr_pool_t,
        ) -> *mut svn_error_t;
    }
}

// pub struct WorkingCopyNotify {
//     path: String,
//     action: ffi::svn_wc_notify_action_t,
//     kind: ffi::svn_node_kind_t,
//     mime_type: Option<String>,
// }

// pub struct WCNotify {
//     path: String,
//     action: ffi::svn_wc_notify_action_t,
//     kind: ffi::svn_node_kind_t,
//     mime_type: Option<String>,
// }

/// Subversion error codes as a Rust enum.
///
/// Each variant corresponds to an `SVN_ERR_*` constant from the Subversion C API.
/// Use [`SvnErrno::to_i32`] to get the raw error code value.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SubversionErrorCode {
    Warning,
    BadContainingPool,
    BadFilename,
    BadUrl,
    BadDate,
    BadMimeType,
    BadPropertyValue,
    BadRelativePath,
    BadUuid,
    BadConfigValue,
    BadChecksumKind,
    BadChecksumParse,
    BadToken,
    BadChangelistName,
    BadAtomic,
    BadPropertyValueEol,
    XmlAttribNotFound,
    XmlMissingAncestry,
    XmlUnknownEncoding,
    XmlMalformed,
    XmlUnescapableData,
    IoInconsistentEol,
    IoUnknownEol,
    IoCorruptEol,
    IoPipeFrameError,
    IoPipeReadError,
    IoWriteError,
    IoPipeWriteError,
    StreamUnexpectedEof,
    StreamMalformedData,
    StreamNotSupported,
    NodeUnknownKind,
    NodeUnexpectedKind,
    EntryNotFound,
    EntryExists,
    EntryMissingUrl,
    EntryForbidden,
    WcObstructedUpdate,
    WcUnwindMismatch,
    WcUnwindEmpty,
    WcUnwindNotEmpty,
    WcLocked,
    WcNotLocked,
    WcInvalidLock,
    WcNotWorkingCopyOrDirectory,
    WcNotFile,
    WcBadAdmLog,
    WcPathNotFound,
    WcNotUpToDate,
    WcLeftLocalMod,
    WcScheduleConflict,
    WcPathFound,
    WcFoundConflict,
    WcCorrupt,
    WcCorruptTextBase,
    WcNodeKindChange,
    WcInvalidOpOnCwd,
    WcBadAdmLogStart,
    WcUnsupportedFormat,
    WcBadPath,
    WcInvalidSchedule,
    WcInvalidRelocation,
    WcInvalidSwitch,
    WcChangelistMove,
    WcDbError,
    WcMissing,
    WcNotSymlink,
    WcUpgradeRequired,
    WcCleanupRequired,
    WcPathAccessDenied,
    WcMixedRevisions,
    FsGeneral,
    FsCleanup,
    FsAlreadyOpen,
    FsNotOpen,
    FsCorrupt,
    FsPathSyntax,
    FsNoSuchRevision,
    FsNoSuchTransaction,
    FsNoSuchEntry,
    FsNoSuchString,
    FsNoSuchCopy,
    FsNotFound,
    FsIdNotFound,
    FsNotId,
    FsNotDirectory,
    FsNotFile,
    FsNotMutable,
    FsAlreadyExists,
    FsRootDir,
    FsNotTxnRoot,
    FsNotRevisionRoot,
    FsConflict,
    FsRepChanged,
    FsRepNotMutable,
    FsMalformedSkel,
    FsTxnOutOfDate,
    FsBerkeleyDb,
    FsTransactionDead,
    FsUnknownFsType,
    FsNoUser,
    FsPathAlreadyLocked,
    FsPathNotLocked,
    FsBadLockToken,
    FsNoLockToken,
    FsLockOwnerMismatch,
    FsNoSuchLock,
    FsLockExpired,
    FsOutOfDate,
    FsUnsupportedFormat,
    FsRepBeingWritten,
    FsTxnNameTooLong,
    FsNoSuchNodeOrigin,
    FsMalformedTxnId,
    FsIndexCorruption,
    FsIndexRevision,
    FsIndexOverflow,
    FsContainerIndex,
    FsIndexInconsistent,
    FsUnsupportedType,
    FsContainerSize,
    FsInvalidGeneration,
    FsCorruptProplist,
    ReposLocked,
    ReposHookFailure,
    ReposBadArgs,
    RaIllegalUrl,
    RaNotAuthorized,
    RaUnknownAuth,
    RaNotImplemented,
    RaOutOfDate,
    RaNoReposUuid,
    RaNotLocked,
    RaUuidMismatch,
    RaDavSockInit,
    RaDavRequestFailed,
    RaDavPropsNotFound,
    RaDavAlreadyExists,
    RaDavPathNotFound,
    RaDavMalformedData,
    RaDavRelocated,
    RaDavConnTimeout,
    RaDavForbidden,
    SvndiffBackwardView,
    SvndiffInvalidOps,
    ApmodMalformedUri,
    ApmodBadBaseline,
    ClientBadRevision,
    ClientIsBinaryFile,
    ClientModified,
    ClientIsDirectory,
    ClientRevisionRange,
    ClientPropertyName,
    ClientCycleDetected,
    ClientNoLockToken,
    Base,
    PluginLoadFailure,
    MalformedFile,
    IncompleteData,
    IncorrectParams,
    UnversionedResource,
    TestFailed,
    UnsupportedFeature,
    BadPropKind,
    IllegalTarget,
    DirNotEmpty,
    ExternalProgram,
    SwigPyExceptionSet,
    ChecksumMismatch,
    Cancelled,
    InvalidDiffOption,
    PropertyNotFound,
    NoAuthFilePath,
    VersionMismatch,
    MergeinfoParseError,
    CeaseInvocation,
    RevnumParseFailure,
    IterBreak,
    UnknownChangelist,
    UnknownCapability,
    TestSkipped,
    NoAprMemcache,
    AtomicInitFailure,
    SqliteError,
    SqliteReadonly,
    SqliteBusy,
    SqliteConstraint,
    Utf8procError,
    Utf8Glob,
    CorruptPackedData,
    ComposedError,
    InvalidInput,
    ClArgParsingError,
    ClInsufficientArgs,
    ClAdmDirReserved,
    ClCommitInAddedDir,
    ClNoExternalEditor,
    ClBadLogMessage,
    ClReposVerifyFailed,
    RaSvnCmdErr,
    RaSvnUnknownCmd,
    RaSvnIoError,
    RaSvnMalformedData,
    RaSvnReposNotFound,
    RaSvnBadVersion,
    RaSvnNoMechanisms,
    RaSvnEditAborted,
    RaSvnRequestSize,
    RaSvnResponseSize,
    AuthnNoProvider,
    AuthnCredsNotSaved,
    AuthnFailed,
    AuthzRootUnreadable,
    AuthzUnreadable,
    AuthzInvalidConfig,
    AuthzUnwritable,
    DiffUnexpectedData,
    RaSerfWrappedError,
    AssertionFail,
    Asn1OutOfData,
    Asn1UnexpectedTag,
    Asn1InvalidLength,
    Asn1LengthMismatch,
    Asn1InvalidData,
    X509CertInvalidPem,
    X509CertInvalidAlg,
    X509CertInvalidName,
    X509CertInvalidDate,
    X509CertSigMismatch,
    BadVersionFileFormat,
    BadServerSpecification,
    BadCompressionMethod,
    XmlUnexpectedElement,
    IoUniqueNamesExhausted,
    StreamUnrecognizedData,
    StreamSeekNotSupported,
    EntryMissingRevision,
    EntryAttributeInvalid,
    WcMismatchedChangelist,
    WcConflictResolverFailure,
    WcCopyfromPathNotFound,
    WcCannotDeleteFileExternal,
    WcCannotMoveFileExternal,
    WcPathUnexpectedStatus,
    WcInvalidOperationDepth,
    WcDuplicateExternalsTarget,
    WcIncompatibleSettings,
    WcDeprecatedApiStorePristine,
    WcPristineDehydrated,
    FsNoSuchRepresentation,
    FsTransactionNotMutable,
    FsNotSinglePathComponent,
    FsBerkeleyDbDeadlock,
    FsTransactionNotDead,
    FsUnsupportedUpgrade,
    FsNoSuchChecksumRep,
    FsPropBasevalueMismatch,
    FsIncorrectEditorCompletion,
    FsPackedRevpropReadFailure,
    FsRevpropCacheInitFailure,
    FsLockOperationFailed,
    FsMalformedNoderevId,
    FsCorruptRevpropManifest,
    FsAmbiguousChecksumRep,
    FsUnrecognizedIoctlCode,
    FsRepSharingNotAllowed,
    FsRepSharingNotSupported,
    ReposNoDataForReport,
    ReposBadRevisionReport,
    ReposUnsupportedVersion,
    ReposDisabledFeature,
    ReposPostCommitHookFailed,
    ReposPostLockHookFailed,
    ReposPostUnlockHookFailed,
    ReposUnsupportedUpgrade,
    RaUnsupportedAbiVersion,
    RaPartialReplayNotSupported,
    RaReposRootUrlMismatch,
    RaSessionUrlMismatch,
    RaCannotCreateTunnel,
    RaCannotCreateSession,
    RaDavCreatingRequest,
    RaDavOptionsReqFailed,
    RaDavInvalidConfigValue,
    RaDavProppatchFailed,
    RaDavResponseHeaderBadness,
    RaDavPreconditionFailed,
    RaDavMethodNotAllowed,
    RaLocalReposNotFound,
    RaLocalReposOpenFailed,
    SvndiffInvalidHeader,
    SvndiffCorruptWindow,
    SvndiffUnexpectedEnd,
    SvndiffInvalidCompressedData,
    ApmodMissingPathToFs,
    ApmodActivityNotFound,
    ApmodConnectionAborted,
    ClientVersionedPathRequired,
    ClientRaAccessRequired,
    ClientDuplicateCommitUrl,
    ClientInvalidExternalsDescription,
    ClientInvalidRelocation,
    ClientRevisionAuthorContainsNewline,
    ClientUnrelatedResources,
    ClientMissingLockToken,
    ClientMultipleSourcesDisallowed,
    ClientNoVersionedParent,
    ClientNotReadyToMerge,
    ClientFileExternalOverwriteVersioned,
    ClientPatchBadStripCount,
    ClientMergeUpdateRequired,
    ClientInvalidMergeinfoNoMergetracking,
    ClientForbiddenByServer,
    ClientConflictOptionNotApplicable,
    DeltaMd5ChecksumAbsent,
    ReservedFilenameSpecified,
    SqliteUnsupportedSchema,
    SqliteResettingForRollback,
    TooManyMemcachedServers,
    MalformedVersionString,
    CorruptedAtomicStorage,
    SqliteRollbackFailed,
    Lz4CompressionFailed,
    Lz4DecompressionFailed,
    CanonicalizationFailed,
    ClMutuallyExclusiveArgs,
    ClLogMessageIsVersionedFile,
    ClLogMessageIsPathname,
    ClUnnecessaryLogMessage,
    ClNoExternalMergeTool,
    ClErrorProcessingExternals,
    RaSvnConnectionClosed,
    AuthnCredsUnavailable,
    AuthnProvidersExhausted,
    AuthzPartiallyReadable,
    DiffDatasourceModified,
    RaSerfSspiInitialisationFailed,
    RaSerfSslCertUntrusted,
    RaSerfGssapiInitialisationFailed,
    RaSerfStreamBucketReadError,
    AssertionOnlyTracingLinks,
    X509FeatureUnavailable,
    X509CertInvalidFormat,
    X509CertInvalidVersion,
    X509CertInvalidSerial,
    X509CertInvalidPubkey,
    X509CertInvalidSignature,
    X509CertInvalidExtensions,
    X509CertUnknownVersion,
    X509CertUnknownPkAlg,
    X509CertVerifyFailed,
    AprNoStat,
    AprNoPool,
    AprBadDate,
    AprInvalidSocket,
    AprNoProc,
    AprNoTime,
    AprNoDir,
    AprNoLock,
    AprNoPoll,
    AprNoSocket,
    AprNoThread,
    AprNoThreadKey,
    AprGeneral,
    AprNoSharedMemory,
    AprBadIp,
    AprBadMask,
    AprDsoOpen,
    AprAbsolutePath,
    AprRelativePath,
    AprIncompletePath,
    AprAboveRoot,
    AprBadPath,
    AprPathWildcard,
    AprSymbolNotFound,
    AprProcUnknown,
    AprNotEnoughEntropy,
    AprInChild,
    AprInParent,
    AprDetached,
    AprNotDetached,
    AprChildDone,
    AprChildNotDone,
    AprTimeUp,
    AprIncomplete,
    AprBadOption,
    AprBadArg,
    AprEof,
    AprNotFound,
    AprAnonymousShm,
    AprFileBasedShm,
    AprKeyBasedShm,
    AprInit,
    AprNotImplemented,
    AprMismatch,
    AprBusy,
    AprPermissionDenied,
    AprAlreadyExists,
    AprNameTooLong,
    AprNoEntry,
    AprNotDir,
    AprNoSpace,
    AprOutOfMemory,
    AprTooManyOpenFiles,
    AprFileTableOverflow,
    AprBadFileDescriptor,
    AprInvalidArg,
    AprIllegalSeek,
    AprWouldBlock,
    AprInterrupted,
    AprNotSock,
    AprConnectionRefused,
    AprInProgress,
    AprConnectionAborted,
    AprConnectionReset,
    AprTimedOut,
    AprHostUnreachable,
    AprNetworkUnreachable,
    AprBadFileType,
    AprBrokenPipe,
    AprCrossDevice,
    AprDirNotEmpty,
    AprAddressFamilyNotSupported,
    AprOperationNotSupported,
    AprOutOfRange,
    Unknown(i32),
}

impl SubversionErrorCode {
    /// 返回原始 `i32` 错误码
    #[must_use]
    pub fn to_i32(self) -> i32 {
        match self {
            Self::Warning => ffi::svn_errno_t_SVN_WARNING as i32,
            Self::BadContainingPool => ffi::svn_errno_t_SVN_ERR_BAD_CONTAINING_POOL as i32,
            Self::BadFilename => ffi::svn_errno_t_SVN_ERR_BAD_FILENAME as i32,
            Self::BadUrl => ffi::svn_errno_t_SVN_ERR_BAD_URL as i32,
            Self::BadDate => ffi::svn_errno_t_SVN_ERR_BAD_DATE as i32,
            Self::BadMimeType => ffi::svn_errno_t_SVN_ERR_BAD_MIME_TYPE as i32,
            Self::BadPropertyValue => ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE as i32,
            Self::BadRelativePath => ffi::svn_errno_t_SVN_ERR_BAD_RELATIVE_PATH as i32,
            Self::BadUuid => ffi::svn_errno_t_SVN_ERR_BAD_UUID as i32,
            Self::BadConfigValue => ffi::svn_errno_t_SVN_ERR_BAD_CONFIG_VALUE as i32,
            Self::BadChecksumKind => ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_KIND as i32,
            Self::BadChecksumParse => ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_PARSE as i32,
            Self::BadToken => ffi::svn_errno_t_SVN_ERR_BAD_TOKEN as i32,
            Self::BadChangelistName => ffi::svn_errno_t_SVN_ERR_BAD_CHANGELIST_NAME as i32,
            Self::BadAtomic => ffi::svn_errno_t_SVN_ERR_BAD_ATOMIC as i32,
            Self::BadPropertyValueEol => ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE_EOL as i32,
            Self::XmlAttribNotFound => ffi::svn_errno_t_SVN_ERR_XML_ATTRIB_NOT_FOUND as i32,
            Self::XmlMissingAncestry => ffi::svn_errno_t_SVN_ERR_XML_MISSING_ANCESTRY as i32,
            Self::XmlUnknownEncoding => ffi::svn_errno_t_SVN_ERR_XML_UNKNOWN_ENCODING as i32,
            Self::XmlMalformed => ffi::svn_errno_t_SVN_ERR_XML_MALFORMED as i32,
            Self::XmlUnescapableData => ffi::svn_errno_t_SVN_ERR_XML_UNESCAPABLE_DATA as i32,
            Self::IoInconsistentEol => ffi::svn_errno_t_SVN_ERR_IO_INCONSISTENT_EOL as i32,
            Self::IoUnknownEol => ffi::svn_errno_t_SVN_ERR_IO_UNKNOWN_EOL as i32,
            Self::IoCorruptEol => ffi::svn_errno_t_SVN_ERR_IO_CORRUPT_EOL as i32,
            Self::IoPipeFrameError => ffi::svn_errno_t_SVN_ERR_IO_PIPE_FRAME_ERROR as i32,
            Self::IoPipeReadError => ffi::svn_errno_t_SVN_ERR_IO_PIPE_READ_ERROR as i32,
            Self::IoWriteError => ffi::svn_errno_t_SVN_ERR_IO_WRITE_ERROR as i32,
            Self::IoPipeWriteError => ffi::svn_errno_t_SVN_ERR_IO_PIPE_WRITE_ERROR as i32,
            Self::StreamUnexpectedEof => ffi::svn_errno_t_SVN_ERR_STREAM_UNEXPECTED_EOF as i32,
            Self::StreamMalformedData => ffi::svn_errno_t_SVN_ERR_STREAM_MALFORMED_DATA as i32,
            Self::StreamNotSupported => ffi::svn_errno_t_SVN_ERR_STREAM_NOT_SUPPORTED as i32,
            Self::NodeUnknownKind => ffi::svn_errno_t_SVN_ERR_NODE_UNKNOWN_KIND as i32,
            Self::NodeUnexpectedKind => ffi::svn_errno_t_SVN_ERR_NODE_UNEXPECTED_KIND as i32,
            Self::EntryNotFound => ffi::svn_errno_t_SVN_ERR_ENTRY_NOT_FOUND as i32,
            Self::EntryExists => ffi::svn_errno_t_SVN_ERR_ENTRY_EXISTS as i32,
            Self::EntryMissingUrl => ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_URL as i32,
            Self::EntryForbidden => ffi::svn_errno_t_SVN_ERR_ENTRY_FORBIDDEN as i32,
            Self::WcObstructedUpdate => ffi::svn_errno_t_SVN_ERR_WC_OBSTRUCTED_UPDATE as i32,
            Self::WcUnwindMismatch => ffi::svn_errno_t_SVN_ERR_WC_UNWIND_MISMATCH as i32,
            Self::WcUnwindEmpty => ffi::svn_errno_t_SVN_ERR_WC_UNWIND_EMPTY as i32,
            Self::WcUnwindNotEmpty => ffi::svn_errno_t_SVN_ERR_WC_UNWIND_NOT_EMPTY as i32,
            Self::WcLocked => ffi::svn_errno_t_SVN_ERR_WC_LOCKED as i32,
            Self::WcNotLocked => ffi::svn_errno_t_SVN_ERR_WC_NOT_LOCKED as i32,
            Self::WcInvalidLock => ffi::svn_errno_t_SVN_ERR_WC_INVALID_LOCK as i32,
            Self::WcNotWorkingCopyOrDirectory => {
                ffi::svn_errno_t_SVN_ERR_WC_NOT_WORKING_COPY as i32
            }
            Self::WcNotFile => ffi::svn_errno_t_SVN_ERR_WC_NOT_FILE as i32,
            Self::WcBadAdmLog => ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG as i32,
            Self::WcPathNotFound => ffi::svn_errno_t_SVN_ERR_WC_PATH_NOT_FOUND as i32,
            Self::WcNotUpToDate => ffi::svn_errno_t_SVN_ERR_WC_NOT_UP_TO_DATE as i32,
            Self::WcLeftLocalMod => ffi::svn_errno_t_SVN_ERR_WC_LEFT_LOCAL_MOD as i32,
            Self::WcScheduleConflict => ffi::svn_errno_t_SVN_ERR_WC_SCHEDULE_CONFLICT as i32,
            Self::WcPathFound => ffi::svn_errno_t_SVN_ERR_WC_PATH_FOUND as i32,
            Self::WcFoundConflict => ffi::svn_errno_t_SVN_ERR_WC_FOUND_CONFLICT as i32,
            Self::WcCorrupt => ffi::svn_errno_t_SVN_ERR_WC_CORRUPT as i32,
            Self::WcCorruptTextBase => ffi::svn_errno_t_SVN_ERR_WC_CORRUPT_TEXT_BASE as i32,
            Self::WcNodeKindChange => ffi::svn_errno_t_SVN_ERR_WC_NODE_KIND_CHANGE as i32,
            Self::WcInvalidOpOnCwd => ffi::svn_errno_t_SVN_ERR_WC_INVALID_OP_ON_CWD as i32,
            Self::WcBadAdmLogStart => ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG_START as i32,
            Self::WcUnsupportedFormat => ffi::svn_errno_t_SVN_ERR_WC_UNSUPPORTED_FORMAT as i32,
            Self::WcBadPath => ffi::svn_errno_t_SVN_ERR_WC_BAD_PATH as i32,
            Self::WcInvalidSchedule => ffi::svn_errno_t_SVN_ERR_WC_INVALID_SCHEDULE as i32,
            Self::WcInvalidRelocation => ffi::svn_errno_t_SVN_ERR_WC_INVALID_RELOCATION as i32,
            Self::WcInvalidSwitch => ffi::svn_errno_t_SVN_ERR_WC_INVALID_SWITCH as i32,
            Self::WcChangelistMove => ffi::svn_errno_t_SVN_ERR_WC_CHANGELIST_MOVE as i32,
            Self::WcDbError => ffi::svn_errno_t_SVN_ERR_WC_DB_ERROR as i32,
            Self::WcMissing => ffi::svn_errno_t_SVN_ERR_WC_MISSING as i32,
            Self::WcNotSymlink => ffi::svn_errno_t_SVN_ERR_WC_NOT_SYMLINK as i32,
            Self::WcUpgradeRequired => ffi::svn_errno_t_SVN_ERR_WC_UPGRADE_REQUIRED as i32,
            Self::WcCleanupRequired => ffi::svn_errno_t_SVN_ERR_WC_CLEANUP_REQUIRED as i32,
            Self::WcPathAccessDenied => ffi::svn_errno_t_SVN_ERR_WC_PATH_ACCESS_DENIED as i32,
            Self::WcMixedRevisions => ffi::svn_errno_t_SVN_ERR_WC_MIXED_REVISIONS as i32,
            Self::FsGeneral => ffi::svn_errno_t_SVN_ERR_FS_GENERAL as i32,
            Self::FsCleanup => ffi::svn_errno_t_SVN_ERR_FS_CLEANUP as i32,
            Self::FsAlreadyOpen => ffi::svn_errno_t_SVN_ERR_FS_ALREADY_OPEN as i32,
            Self::FsNotOpen => ffi::svn_errno_t_SVN_ERR_FS_NOT_OPEN as i32,
            Self::FsCorrupt => ffi::svn_errno_t_SVN_ERR_FS_CORRUPT as i32,
            Self::FsPathSyntax => ffi::svn_errno_t_SVN_ERR_FS_PATH_SYNTAX as i32,
            Self::FsNoSuchRevision => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REVISION as i32,
            Self::FsNoSuchTransaction => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_TRANSACTION as i32,
            Self::FsNoSuchEntry => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_ENTRY as i32,
            Self::FsNoSuchString => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_STRING as i32,
            Self::FsNoSuchCopy => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_COPY as i32,
            Self::FsNotFound => ffi::svn_errno_t_SVN_ERR_FS_NOT_FOUND as i32,
            Self::FsIdNotFound => ffi::svn_errno_t_SVN_ERR_FS_ID_NOT_FOUND as i32,
            Self::FsNotId => ffi::svn_errno_t_SVN_ERR_FS_NOT_ID as i32,
            Self::FsNotDirectory => ffi::svn_errno_t_SVN_ERR_FS_NOT_DIRECTORY as i32,
            Self::FsNotFile => ffi::svn_errno_t_SVN_ERR_FS_NOT_FILE as i32,
            Self::FsNotMutable => ffi::svn_errno_t_SVN_ERR_FS_NOT_MUTABLE as i32,
            Self::FsAlreadyExists => ffi::svn_errno_t_SVN_ERR_FS_ALREADY_EXISTS as i32,
            Self::FsRootDir => ffi::svn_errno_t_SVN_ERR_FS_ROOT_DIR as i32,
            Self::FsNotTxnRoot => ffi::svn_errno_t_SVN_ERR_FS_NOT_TXN_ROOT as i32,
            Self::FsNotRevisionRoot => ffi::svn_errno_t_SVN_ERR_FS_NOT_REVISION_ROOT as i32,
            Self::FsConflict => ffi::svn_errno_t_SVN_ERR_FS_CONFLICT as i32,
            Self::FsRepChanged => ffi::svn_errno_t_SVN_ERR_FS_REP_CHANGED as i32,
            Self::FsRepNotMutable => ffi::svn_errno_t_SVN_ERR_FS_REP_NOT_MUTABLE as i32,
            Self::FsMalformedSkel => ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_SKEL as i32,
            Self::FsTxnOutOfDate => ffi::svn_errno_t_SVN_ERR_FS_TXN_OUT_OF_DATE as i32,
            Self::FsBerkeleyDb => ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB as i32,
            Self::FsTransactionDead => ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_DEAD as i32,
            Self::FsUnknownFsType => ffi::svn_errno_t_SVN_ERR_FS_UNKNOWN_FS_TYPE as i32,
            Self::FsNoUser => ffi::svn_errno_t_SVN_ERR_FS_NO_USER as i32,
            Self::FsPathAlreadyLocked => ffi::svn_errno_t_SVN_ERR_FS_PATH_ALREADY_LOCKED as i32,
            Self::FsPathNotLocked => ffi::svn_errno_t_SVN_ERR_FS_PATH_NOT_LOCKED as i32,
            Self::FsBadLockToken => ffi::svn_errno_t_SVN_ERR_FS_BAD_LOCK_TOKEN as i32,
            Self::FsNoLockToken => ffi::svn_errno_t_SVN_ERR_FS_NO_LOCK_TOKEN as i32,
            Self::FsLockOwnerMismatch => ffi::svn_errno_t_SVN_ERR_FS_LOCK_OWNER_MISMATCH as i32,
            Self::FsNoSuchLock => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_LOCK as i32,
            Self::FsLockExpired => ffi::svn_errno_t_SVN_ERR_FS_LOCK_EXPIRED as i32,
            Self::FsOutOfDate => ffi::svn_errno_t_SVN_ERR_FS_OUT_OF_DATE as i32,
            Self::FsUnsupportedFormat => ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_FORMAT as i32,
            Self::FsRepBeingWritten => ffi::svn_errno_t_SVN_ERR_FS_REP_BEING_WRITTEN as i32,
            Self::FsTxnNameTooLong => ffi::svn_errno_t_SVN_ERR_FS_TXN_NAME_TOO_LONG as i32,
            Self::FsNoSuchNodeOrigin => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_NODE_ORIGIN as i32,
            Self::FsMalformedTxnId => ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_TXN_ID as i32,
            Self::FsIndexCorruption => ffi::svn_errno_t_SVN_ERR_FS_INDEX_CORRUPTION as i32,
            Self::FsIndexRevision => ffi::svn_errno_t_SVN_ERR_FS_INDEX_REVISION as i32,
            Self::FsIndexOverflow => ffi::svn_errno_t_SVN_ERR_FS_INDEX_OVERFLOW as i32,
            Self::FsContainerIndex => ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_INDEX as i32,
            Self::FsIndexInconsistent => ffi::svn_errno_t_SVN_ERR_FS_INDEX_INCONSISTENT as i32,
            Self::FsUnsupportedType => ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_TYPE as i32,
            Self::FsContainerSize => ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_SIZE as i32,
            Self::FsInvalidGeneration => ffi::svn_errno_t_SVN_ERR_FS_INVALID_GENERATION as i32,
            Self::FsCorruptProplist => ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_PROPLIST as i32,
            Self::ReposLocked => ffi::svn_errno_t_SVN_ERR_REPOS_LOCKED as i32,
            Self::ReposHookFailure => ffi::svn_errno_t_SVN_ERR_REPOS_HOOK_FAILURE as i32,
            Self::ReposBadArgs => ffi::svn_errno_t_SVN_ERR_REPOS_BAD_ARGS as i32,
            Self::RaIllegalUrl => ffi::svn_errno_t_SVN_ERR_RA_ILLEGAL_URL as i32,
            Self::RaNotAuthorized => ffi::svn_errno_t_SVN_ERR_RA_NOT_AUTHORIZED as i32,
            Self::RaUnknownAuth => ffi::svn_errno_t_SVN_ERR_RA_UNKNOWN_AUTH as i32,
            Self::RaNotImplemented => ffi::svn_errno_t_SVN_ERR_RA_NOT_IMPLEMENTED as i32,
            Self::RaOutOfDate => ffi::svn_errno_t_SVN_ERR_RA_OUT_OF_DATE as i32,
            Self::RaNoReposUuid => ffi::svn_errno_t_SVN_ERR_RA_NO_REPOS_UUID as i32,
            Self::RaNotLocked => ffi::svn_errno_t_SVN_ERR_RA_NOT_LOCKED as i32,
            Self::RaUuidMismatch => ffi::svn_errno_t_SVN_ERR_RA_UUID_MISMATCH as i32,
            Self::RaDavSockInit => ffi::svn_errno_t_SVN_ERR_RA_DAV_SOCK_INIT as i32,
            Self::RaDavRequestFailed => ffi::svn_errno_t_SVN_ERR_RA_DAV_REQUEST_FAILED as i32,
            Self::RaDavPropsNotFound => ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPS_NOT_FOUND as i32,
            Self::RaDavAlreadyExists => ffi::svn_errno_t_SVN_ERR_RA_DAV_ALREADY_EXISTS as i32,
            Self::RaDavPathNotFound => ffi::svn_errno_t_SVN_ERR_RA_DAV_PATH_NOT_FOUND as i32,
            Self::RaDavMalformedData => ffi::svn_errno_t_SVN_ERR_RA_DAV_MALFORMED_DATA as i32,
            Self::RaDavRelocated => ffi::svn_errno_t_SVN_ERR_RA_DAV_RELOCATED as i32,
            Self::RaDavConnTimeout => ffi::svn_errno_t_SVN_ERR_RA_DAV_CONN_TIMEOUT as i32,
            Self::RaDavForbidden => ffi::svn_errno_t_SVN_ERR_RA_DAV_FORBIDDEN as i32,
            Self::SvndiffBackwardView => ffi::svn_errno_t_SVN_ERR_SVNDIFF_BACKWARD_VIEW as i32,
            Self::SvndiffInvalidOps => ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_OPS as i32,
            Self::ApmodMalformedUri => ffi::svn_errno_t_SVN_ERR_APMOD_MALFORMED_URI as i32,
            Self::ApmodBadBaseline => ffi::svn_errno_t_SVN_ERR_APMOD_BAD_BASELINE as i32,
            Self::ClientBadRevision => ffi::svn_errno_t_SVN_ERR_CLIENT_BAD_REVISION as i32,
            Self::ClientIsBinaryFile => ffi::svn_errno_t_SVN_ERR_CLIENT_IS_BINARY_FILE as i32,
            Self::ClientModified => ffi::svn_errno_t_SVN_ERR_CLIENT_MODIFIED as i32,
            Self::ClientIsDirectory => ffi::svn_errno_t_SVN_ERR_CLIENT_IS_DIRECTORY as i32,
            Self::ClientRevisionRange => ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_RANGE as i32,
            Self::ClientPropertyName => ffi::svn_errno_t_SVN_ERR_CLIENT_PROPERTY_NAME as i32,
            Self::ClientCycleDetected => ffi::svn_errno_t_SVN_ERR_CLIENT_CYCLE_DETECTED as i32,
            Self::ClientNoLockToken => ffi::svn_errno_t_SVN_ERR_CLIENT_NO_LOCK_TOKEN as i32,
            Self::Base => ffi::svn_errno_t_SVN_ERR_BASE as i32,
            Self::PluginLoadFailure => ffi::svn_errno_t_SVN_ERR_PLUGIN_LOAD_FAILURE as i32,
            Self::MalformedFile => ffi::svn_errno_t_SVN_ERR_MALFORMED_FILE as i32,
            Self::IncompleteData => ffi::svn_errno_t_SVN_ERR_INCOMPLETE_DATA as i32,
            Self::IncorrectParams => ffi::svn_errno_t_SVN_ERR_INCORRECT_PARAMS as i32,
            Self::UnversionedResource => ffi::svn_errno_t_SVN_ERR_UNVERSIONED_RESOURCE as i32,
            Self::TestFailed => ffi::svn_errno_t_SVN_ERR_TEST_FAILED as i32,
            Self::UnsupportedFeature => ffi::svn_errno_t_SVN_ERR_UNSUPPORTED_FEATURE as i32,
            Self::BadPropKind => ffi::svn_errno_t_SVN_ERR_BAD_PROP_KIND as i32,
            Self::IllegalTarget => ffi::svn_errno_t_SVN_ERR_ILLEGAL_TARGET as i32,
            Self::DirNotEmpty => ffi::svn_errno_t_SVN_ERR_DIR_NOT_EMPTY as i32,
            Self::ExternalProgram => ffi::svn_errno_t_SVN_ERR_EXTERNAL_PROGRAM as i32,
            Self::SwigPyExceptionSet => ffi::svn_errno_t_SVN_ERR_SWIG_PY_EXCEPTION_SET as i32,
            Self::ChecksumMismatch => ffi::svn_errno_t_SVN_ERR_CHECKSUM_MISMATCH as i32,
            Self::Cancelled => ffi::svn_errno_t_SVN_ERR_CANCELLED as i32,
            Self::InvalidDiffOption => ffi::svn_errno_t_SVN_ERR_INVALID_DIFF_OPTION as i32,
            Self::PropertyNotFound => ffi::svn_errno_t_SVN_ERR_PROPERTY_NOT_FOUND as i32,
            Self::NoAuthFilePath => ffi::svn_errno_t_SVN_ERR_NO_AUTH_FILE_PATH as i32,
            Self::VersionMismatch => ffi::svn_errno_t_SVN_ERR_VERSION_MISMATCH as i32,
            Self::MergeinfoParseError => ffi::svn_errno_t_SVN_ERR_MERGEINFO_PARSE_ERROR as i32,
            Self::CeaseInvocation => ffi::svn_errno_t_SVN_ERR_CEASE_INVOCATION as i32,
            Self::RevnumParseFailure => ffi::svn_errno_t_SVN_ERR_REVNUM_PARSE_FAILURE as i32,
            Self::IterBreak => ffi::svn_errno_t_SVN_ERR_ITER_BREAK as i32,
            Self::UnknownChangelist => ffi::svn_errno_t_SVN_ERR_UNKNOWN_CHANGELIST as i32,
            Self::UnknownCapability => ffi::svn_errno_t_SVN_ERR_UNKNOWN_CAPABILITY as i32,
            Self::TestSkipped => ffi::svn_errno_t_SVN_ERR_TEST_SKIPPED as i32,
            Self::NoAprMemcache => ffi::svn_errno_t_SVN_ERR_NO_APR_MEMCACHE as i32,
            Self::AtomicInitFailure => ffi::svn_errno_t_SVN_ERR_ATOMIC_INIT_FAILURE as i32,
            Self::SqliteError => ffi::svn_errno_t_SVN_ERR_SQLITE_ERROR as i32,
            Self::SqliteReadonly => ffi::svn_errno_t_SVN_ERR_SQLITE_READONLY as i32,
            Self::SqliteBusy => ffi::svn_errno_t_SVN_ERR_SQLITE_BUSY as i32,
            Self::SqliteConstraint => ffi::svn_errno_t_SVN_ERR_SQLITE_CONSTRAINT as i32,
            Self::Utf8procError => ffi::svn_errno_t_SVN_ERR_UTF8PROC_ERROR as i32,
            Self::Utf8Glob => ffi::svn_errno_t_SVN_ERR_UTF8_GLOB as i32,
            Self::CorruptPackedData => ffi::svn_errno_t_SVN_ERR_CORRUPT_PACKED_DATA as i32,
            Self::ComposedError => ffi::svn_errno_t_SVN_ERR_COMPOSED_ERROR as i32,
            Self::InvalidInput => ffi::svn_errno_t_SVN_ERR_INVALID_INPUT as i32,
            Self::ClArgParsingError => ffi::svn_errno_t_SVN_ERR_CL_ARG_PARSING_ERROR as i32,
            Self::ClInsufficientArgs => ffi::svn_errno_t_SVN_ERR_CL_INSUFFICIENT_ARGS as i32,
            Self::ClAdmDirReserved => ffi::svn_errno_t_SVN_ERR_CL_ADM_DIR_RESERVED as i32,
            Self::ClCommitInAddedDir => ffi::svn_errno_t_SVN_ERR_CL_COMMIT_IN_ADDED_DIR as i32,
            Self::ClNoExternalEditor => ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_EDITOR as i32,
            Self::ClBadLogMessage => ffi::svn_errno_t_SVN_ERR_CL_BAD_LOG_MESSAGE as i32,
            Self::ClReposVerifyFailed => ffi::svn_errno_t_SVN_ERR_CL_REPOS_VERIFY_FAILED as i32,
            Self::RaSvnCmdErr => ffi::svn_errno_t_SVN_ERR_RA_SVN_CMD_ERR as i32,
            Self::RaSvnUnknownCmd => ffi::svn_errno_t_SVN_ERR_RA_SVN_UNKNOWN_CMD as i32,
            Self::RaSvnIoError => ffi::svn_errno_t_SVN_ERR_RA_SVN_IO_ERROR as i32,
            Self::RaSvnMalformedData => ffi::svn_errno_t_SVN_ERR_RA_SVN_MALFORMED_DATA as i32,
            Self::RaSvnReposNotFound => ffi::svn_errno_t_SVN_ERR_RA_SVN_REPOS_NOT_FOUND as i32,
            Self::RaSvnBadVersion => ffi::svn_errno_t_SVN_ERR_RA_SVN_BAD_VERSION as i32,
            Self::RaSvnNoMechanisms => ffi::svn_errno_t_SVN_ERR_RA_SVN_NO_MECHANISMS as i32,
            Self::RaSvnEditAborted => ffi::svn_errno_t_SVN_ERR_RA_SVN_EDIT_ABORTED as i32,
            Self::RaSvnRequestSize => ffi::svn_errno_t_SVN_ERR_RA_SVN_REQUEST_SIZE as i32,
            Self::RaSvnResponseSize => ffi::svn_errno_t_SVN_ERR_RA_SVN_RESPONSE_SIZE as i32,
            Self::AuthnNoProvider => ffi::svn_errno_t_SVN_ERR_AUTHN_NO_PROVIDER as i32,
            Self::AuthnCredsNotSaved => ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_NOT_SAVED as i32,
            Self::AuthnFailed => ffi::svn_errno_t_SVN_ERR_AUTHN_FAILED as i32,
            Self::AuthzRootUnreadable => ffi::svn_errno_t_SVN_ERR_AUTHZ_ROOT_UNREADABLE as i32,
            Self::AuthzUnreadable => ffi::svn_errno_t_SVN_ERR_AUTHZ_UNREADABLE as i32,
            Self::AuthzInvalidConfig => ffi::svn_errno_t_SVN_ERR_AUTHZ_INVALID_CONFIG as i32,
            Self::AuthzUnwritable => ffi::svn_errno_t_SVN_ERR_AUTHZ_UNWRITABLE as i32,
            Self::DiffUnexpectedData => ffi::svn_errno_t_SVN_ERR_DIFF_UNEXPECTED_DATA as i32,
            Self::RaSerfWrappedError => ffi::svn_errno_t_SVN_ERR_RA_SERF_WRAPPED_ERROR as i32,
            Self::AssertionFail => ffi::svn_errno_t_SVN_ERR_ASSERTION_FAIL as i32,
            Self::Asn1OutOfData => ffi::svn_errno_t_SVN_ERR_ASN1_OUT_OF_DATA as i32,
            Self::Asn1UnexpectedTag => ffi::svn_errno_t_SVN_ERR_ASN1_UNEXPECTED_TAG as i32,
            Self::Asn1InvalidLength => ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_LENGTH as i32,
            Self::Asn1LengthMismatch => ffi::svn_errno_t_SVN_ERR_ASN1_LENGTH_MISMATCH as i32,
            Self::Asn1InvalidData => ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_DATA as i32,
            Self::X509CertInvalidPem => ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PEM as i32,
            Self::X509CertInvalidAlg => ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_ALG as i32,
            Self::X509CertInvalidName => ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_NAME as i32,
            Self::X509CertInvalidDate => ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_DATE as i32,
            Self::X509CertSigMismatch => ffi::svn_errno_t_SVN_ERR_X509_CERT_SIG_MISMATCH as i32,
            Self::BadVersionFileFormat => ffi::svn_errno_t_SVN_ERR_BAD_VERSION_FILE_FORMAT as i32,
            Self::BadServerSpecification => {
                ffi::svn_errno_t_SVN_ERR_BAD_SERVER_SPECIFICATION as i32
            }
            Self::BadCompressionMethod => ffi::svn_errno_t_SVN_ERR_BAD_COMPRESSION_METHOD as i32,
            Self::XmlUnexpectedElement => ffi::svn_errno_t_SVN_ERR_XML_UNEXPECTED_ELEMENT as i32,
            Self::IoUniqueNamesExhausted => {
                ffi::svn_errno_t_SVN_ERR_IO_UNIQUE_NAMES_EXHAUSTED as i32
            }
            Self::StreamUnrecognizedData => {
                ffi::svn_errno_t_SVN_ERR_STREAM_UNRECOGNIZED_DATA as i32
            }
            Self::StreamSeekNotSupported => {
                ffi::svn_errno_t_SVN_ERR_STREAM_SEEK_NOT_SUPPORTED as i32
            }
            Self::EntryMissingRevision => ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_REVISION as i32,
            Self::EntryAttributeInvalid => ffi::svn_errno_t_SVN_ERR_ENTRY_ATTRIBUTE_INVALID as i32,
            Self::WcMismatchedChangelist => {
                ffi::svn_errno_t_SVN_ERR_WC_MISMATCHED_CHANGELIST as i32
            }
            Self::WcConflictResolverFailure => {
                ffi::svn_errno_t_SVN_ERR_WC_CONFLICT_RESOLVER_FAILURE as i32
            }
            Self::WcCopyfromPathNotFound => {
                ffi::svn_errno_t_SVN_ERR_WC_COPYFROM_PATH_NOT_FOUND as i32
            }
            Self::WcCannotDeleteFileExternal => {
                ffi::svn_errno_t_SVN_ERR_WC_CANNOT_DELETE_FILE_EXTERNAL as i32
            }
            Self::WcCannotMoveFileExternal => {
                ffi::svn_errno_t_SVN_ERR_WC_CANNOT_MOVE_FILE_EXTERNAL as i32
            }
            Self::WcPathUnexpectedStatus => {
                ffi::svn_errno_t_SVN_ERR_WC_PATH_UNEXPECTED_STATUS as i32
            }
            Self::WcInvalidOperationDepth => {
                ffi::svn_errno_t_SVN_ERR_WC_INVALID_OPERATION_DEPTH as i32
            }
            Self::WcDuplicateExternalsTarget => {
                ffi::svn_errno_t_SVN_ERR_WC_DUPLICATE_EXTERNALS_TARGET as i32
            }
            Self::WcIncompatibleSettings => {
                ffi::svn_errno_t_SVN_ERR_WC_INCOMPATIBLE_SETTINGS as i32
            }
            Self::WcDeprecatedApiStorePristine => {
                ffi::svn_errno_t_SVN_ERR_WC_DEPRECATED_API_STORE_PRISTINE as i32
            }
            Self::WcPristineDehydrated => ffi::svn_errno_t_SVN_ERR_WC_PRISTINE_DEHYDRATED as i32,
            Self::FsNoSuchRepresentation => {
                ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REPRESENTATION as i32
            }
            Self::FsTransactionNotMutable => {
                ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_MUTABLE as i32
            }
            Self::FsNotSinglePathComponent => {
                ffi::svn_errno_t_SVN_ERR_FS_NOT_SINGLE_PATH_COMPONENT as i32
            }
            Self::FsBerkeleyDbDeadlock => ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB_DEADLOCK as i32,
            Self::FsTransactionNotDead => ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_DEAD as i32,
            Self::FsUnsupportedUpgrade => ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_UPGRADE as i32,
            Self::FsNoSuchChecksumRep => ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_CHECKSUM_REP as i32,
            Self::FsPropBasevalueMismatch => {
                ffi::svn_errno_t_SVN_ERR_FS_PROP_BASEVALUE_MISMATCH as i32
            }
            Self::FsIncorrectEditorCompletion => {
                ffi::svn_errno_t_SVN_ERR_FS_INCORRECT_EDITOR_COMPLETION as i32
            }
            Self::FsPackedRevpropReadFailure => {
                ffi::svn_errno_t_SVN_ERR_FS_PACKED_REVPROP_READ_FAILURE as i32
            }
            Self::FsRevpropCacheInitFailure => {
                ffi::svn_errno_t_SVN_ERR_FS_REVPROP_CACHE_INIT_FAILURE as i32
            }
            Self::FsLockOperationFailed => ffi::svn_errno_t_SVN_ERR_FS_LOCK_OPERATION_FAILED as i32,
            Self::FsMalformedNoderevId => ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_NODEREV_ID as i32,
            Self::FsCorruptRevpropManifest => {
                ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_REVPROP_MANIFEST as i32
            }
            Self::FsAmbiguousChecksumRep => {
                ffi::svn_errno_t_SVN_ERR_FS_AMBIGUOUS_CHECKSUM_REP as i32
            }
            Self::FsUnrecognizedIoctlCode => {
                ffi::svn_errno_t_SVN_ERR_FS_UNRECOGNIZED_IOCTL_CODE as i32
            }
            Self::FsRepSharingNotAllowed => {
                ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_ALLOWED as i32
            }
            Self::FsRepSharingNotSupported => {
                ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_SUPPORTED as i32
            }
            Self::ReposNoDataForReport => ffi::svn_errno_t_SVN_ERR_REPOS_NO_DATA_FOR_REPORT as i32,
            Self::ReposBadRevisionReport => {
                ffi::svn_errno_t_SVN_ERR_REPOS_BAD_REVISION_REPORT as i32
            }
            Self::ReposUnsupportedVersion => {
                ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_VERSION as i32
            }
            Self::ReposDisabledFeature => ffi::svn_errno_t_SVN_ERR_REPOS_DISABLED_FEATURE as i32,
            Self::ReposPostCommitHookFailed => {
                ffi::svn_errno_t_SVN_ERR_REPOS_POST_COMMIT_HOOK_FAILED as i32
            }
            Self::ReposPostLockHookFailed => {
                ffi::svn_errno_t_SVN_ERR_REPOS_POST_LOCK_HOOK_FAILED as i32
            }
            Self::ReposPostUnlockHookFailed => {
                ffi::svn_errno_t_SVN_ERR_REPOS_POST_UNLOCK_HOOK_FAILED as i32
            }
            Self::ReposUnsupportedUpgrade => {
                ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_UPGRADE as i32
            }
            Self::RaUnsupportedAbiVersion => {
                ffi::svn_errno_t_SVN_ERR_RA_UNSUPPORTED_ABI_VERSION as i32
            }
            Self::RaPartialReplayNotSupported => {
                ffi::svn_errno_t_SVN_ERR_RA_PARTIAL_REPLAY_NOT_SUPPORTED as i32
            }
            Self::RaReposRootUrlMismatch => {
                ffi::svn_errno_t_SVN_ERR_RA_REPOS_ROOT_URL_MISMATCH as i32
            }
            Self::RaSessionUrlMismatch => ffi::svn_errno_t_SVN_ERR_RA_SESSION_URL_MISMATCH as i32,
            Self::RaCannotCreateTunnel => ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_TUNNEL as i32,
            Self::RaCannotCreateSession => ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_SESSION as i32,
            Self::RaDavCreatingRequest => ffi::svn_errno_t_SVN_ERR_RA_DAV_CREATING_REQUEST as i32,
            Self::RaDavOptionsReqFailed => {
                ffi::svn_errno_t_SVN_ERR_RA_DAV_OPTIONS_REQ_FAILED as i32
            }
            Self::RaDavInvalidConfigValue => {
                ffi::svn_errno_t_SVN_ERR_RA_DAV_INVALID_CONFIG_VALUE as i32
            }
            Self::RaDavProppatchFailed => ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPPATCH_FAILED as i32,
            Self::RaDavResponseHeaderBadness => {
                ffi::svn_errno_t_SVN_ERR_RA_DAV_RESPONSE_HEADER_BADNESS as i32
            }
            Self::RaDavPreconditionFailed => {
                ffi::svn_errno_t_SVN_ERR_RA_DAV_PRECONDITION_FAILED as i32
            }
            Self::RaDavMethodNotAllowed => {
                ffi::svn_errno_t_SVN_ERR_RA_DAV_METHOD_NOT_ALLOWED as i32
            }
            Self::RaLocalReposNotFound => ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_NOT_FOUND as i32,
            Self::RaLocalReposOpenFailed => {
                ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_OPEN_FAILED as i32
            }
            Self::SvndiffInvalidHeader => ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_HEADER as i32,
            Self::SvndiffCorruptWindow => ffi::svn_errno_t_SVN_ERR_SVNDIFF_CORRUPT_WINDOW as i32,
            Self::SvndiffUnexpectedEnd => ffi::svn_errno_t_SVN_ERR_SVNDIFF_UNEXPECTED_END as i32,
            Self::SvndiffInvalidCompressedData => {
                ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_COMPRESSED_DATA as i32
            }
            Self::ApmodMissingPathToFs => ffi::svn_errno_t_SVN_ERR_APMOD_MISSING_PATH_TO_FS as i32,
            Self::ApmodActivityNotFound => ffi::svn_errno_t_SVN_ERR_APMOD_ACTIVITY_NOT_FOUND as i32,
            Self::ApmodConnectionAborted => {
                ffi::svn_errno_t_SVN_ERR_APMOD_CONNECTION_ABORTED as i32
            }
            Self::ClientVersionedPathRequired => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_VERSIONED_PATH_REQUIRED as i32
            }
            Self::ClientRaAccessRequired => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_RA_ACCESS_REQUIRED as i32
            }
            Self::ClientDuplicateCommitUrl => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_DUPLICATE_COMMIT_URL as i32
            }
            Self::ClientInvalidExternalsDescription => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_EXTERNALS_DESCRIPTION as i32
            }
            Self::ClientInvalidRelocation => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_RELOCATION as i32
            }
            Self::ClientRevisionAuthorContainsNewline => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_AUTHOR_CONTAINS_NEWLINE as i32
            }
            Self::ClientUnrelatedResources => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_UNRELATED_RESOURCES as i32
            }
            Self::ClientMissingLockToken => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_MISSING_LOCK_TOKEN as i32
            }
            Self::ClientMultipleSourcesDisallowed => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_MULTIPLE_SOURCES_DISALLOWED as i32
            }
            Self::ClientNoVersionedParent => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_NO_VERSIONED_PARENT as i32
            }
            Self::ClientNotReadyToMerge => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_NOT_READY_TO_MERGE as i32
            }
            Self::ClientFileExternalOverwriteVersioned => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_FILE_EXTERNAL_OVERWRITE_VERSIONED as i32
            }
            Self::ClientPatchBadStripCount => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_PATCH_BAD_STRIP_COUNT as i32
            }
            Self::ClientMergeUpdateRequired => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_MERGE_UPDATE_REQUIRED as i32
            }
            Self::ClientInvalidMergeinfoNoMergetracking => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_MERGEINFO_NO_MERGETRACKING as i32
            }
            Self::ClientForbiddenByServer => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_FORBIDDEN_BY_SERVER as i32
            }
            Self::ClientConflictOptionNotApplicable => {
                ffi::svn_errno_t_SVN_ERR_CLIENT_CONFLICT_OPTION_NOT_APPLICABLE as i32
            }
            Self::DeltaMd5ChecksumAbsent => {
                ffi::svn_errno_t_SVN_ERR_DELTA_MD5_CHECKSUM_ABSENT as i32
            }
            Self::ReservedFilenameSpecified => {
                ffi::svn_errno_t_SVN_ERR_RESERVED_FILENAME_SPECIFIED as i32
            }
            Self::SqliteUnsupportedSchema => {
                ffi::svn_errno_t_SVN_ERR_SQLITE_UNSUPPORTED_SCHEMA as i32
            }
            Self::SqliteResettingForRollback => {
                ffi::svn_errno_t_SVN_ERR_SQLITE_RESETTING_FOR_ROLLBACK as i32
            }
            Self::TooManyMemcachedServers => {
                ffi::svn_errno_t_SVN_ERR_TOO_MANY_MEMCACHED_SERVERS as i32
            }
            Self::MalformedVersionString => {
                ffi::svn_errno_t_SVN_ERR_MALFORMED_VERSION_STRING as i32
            }
            Self::CorruptedAtomicStorage => {
                ffi::svn_errno_t_SVN_ERR_CORRUPTED_ATOMIC_STORAGE as i32
            }
            Self::SqliteRollbackFailed => ffi::svn_errno_t_SVN_ERR_SQLITE_ROLLBACK_FAILED as i32,
            Self::Lz4CompressionFailed => ffi::svn_errno_t_SVN_ERR_LZ4_COMPRESSION_FAILED as i32,
            Self::Lz4DecompressionFailed => {
                ffi::svn_errno_t_SVN_ERR_LZ4_DECOMPRESSION_FAILED as i32
            }
            Self::CanonicalizationFailed => ffi::svn_errno_t_SVN_ERR_CANONICALIZATION_FAILED as i32,
            Self::ClMutuallyExclusiveArgs => {
                ffi::svn_errno_t_SVN_ERR_CL_MUTUALLY_EXCLUSIVE_ARGS as i32
            }
            Self::ClLogMessageIsVersionedFile => {
                ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_VERSIONED_FILE as i32
            }
            Self::ClLogMessageIsPathname => {
                ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_PATHNAME as i32
            }
            Self::ClUnnecessaryLogMessage => {
                ffi::svn_errno_t_SVN_ERR_CL_UNNECESSARY_LOG_MESSAGE as i32
            }
            Self::ClNoExternalMergeTool => {
                ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_MERGE_TOOL as i32
            }
            Self::ClErrorProcessingExternals => {
                ffi::svn_errno_t_SVN_ERR_CL_ERROR_PROCESSING_EXTERNALS as i32
            }
            Self::RaSvnConnectionClosed => ffi::svn_errno_t_SVN_ERR_RA_SVN_CONNECTION_CLOSED as i32,
            Self::AuthnCredsUnavailable => ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_UNAVAILABLE as i32,
            Self::AuthnProvidersExhausted => {
                ffi::svn_errno_t_SVN_ERR_AUTHN_PROVIDERS_EXHAUSTED as i32
            }
            Self::AuthzPartiallyReadable => {
                ffi::svn_errno_t_SVN_ERR_AUTHZ_PARTIALLY_READABLE as i32
            }
            Self::DiffDatasourceModified => {
                ffi::svn_errno_t_SVN_ERR_DIFF_DATASOURCE_MODIFIED as i32
            }
            Self::RaSerfSspiInitialisationFailed => {
                ffi::svn_errno_t_SVN_ERR_RA_SERF_SSPI_INITIALISATION_FAILED as i32
            }
            Self::RaSerfSslCertUntrusted => {
                ffi::svn_errno_t_SVN_ERR_RA_SERF_SSL_CERT_UNTRUSTED as i32
            }
            Self::RaSerfGssapiInitialisationFailed => {
                ffi::svn_errno_t_SVN_ERR_RA_SERF_GSSAPI_INITIALISATION_FAILED as i32
            }
            Self::RaSerfStreamBucketReadError => {
                ffi::svn_errno_t_SVN_ERR_RA_SERF_STREAM_BUCKET_READ_ERROR as i32
            }
            Self::AssertionOnlyTracingLinks => {
                ffi::svn_errno_t_SVN_ERR_ASSERTION_ONLY_TRACING_LINKS as i32
            }
            Self::X509FeatureUnavailable => {
                ffi::svn_errno_t_SVN_ERR_X509_FEATURE_UNAVAILABLE as i32
            }
            Self::X509CertInvalidFormat => ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_FORMAT as i32,
            Self::X509CertInvalidVersion => {
                ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_VERSION as i32
            }
            Self::X509CertInvalidSerial => ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SERIAL as i32,
            Self::X509CertInvalidPubkey => ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PUBKEY as i32,
            Self::X509CertInvalidSignature => {
                ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SIGNATURE as i32
            }
            Self::X509CertInvalidExtensions => {
                ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_EXTENSIONS as i32
            }
            Self::X509CertUnknownVersion => {
                ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_VERSION as i32
            }
            Self::X509CertUnknownPkAlg => ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_PK_ALG as i32,
            Self::X509CertVerifyFailed => ffi::svn_errno_t_SVN_ERR_X509_CERT_VERIFY_FAILED as i32,
            Self::AprNoStat => ffi::APR_ENOSTAT as i32,
            Self::AprNoPool => ffi::APR_ENOPOOL as i32,
            Self::AprBadDate => ffi::APR_EBADDATE as i32,
            Self::AprInvalidSocket => ffi::APR_EINVALSOCK as i32,
            Self::AprNoProc => ffi::APR_ENOPROC as i32,
            Self::AprNoTime => ffi::APR_ENOTIME as i32,
            Self::AprNoDir => ffi::APR_ENODIR as i32,
            Self::AprNoLock => ffi::APR_ENOLOCK as i32,
            Self::AprNoPoll => ffi::APR_ENOPOLL as i32,
            Self::AprNoSocket => ffi::APR_ENOSOCKET as i32,
            Self::AprNoThread => ffi::APR_ENOTHREAD as i32,
            Self::AprNoThreadKey => ffi::APR_ENOTHDKEY as i32,
            Self::AprGeneral => ffi::APR_EGENERAL as i32,
            Self::AprNoSharedMemory => ffi::APR_ENOSHMAVAIL as i32,
            Self::AprBadIp => ffi::APR_EBADIP as i32,
            Self::AprBadMask => ffi::APR_EBADMASK as i32,
            Self::AprDsoOpen => ffi::APR_EDSOOPEN as i32,
            Self::AprAbsolutePath => ffi::APR_EABSOLUTE as i32,
            Self::AprRelativePath => ffi::APR_ERELATIVE as i32,
            Self::AprIncompletePath => ffi::APR_EINCOMPLETE as i32,
            Self::AprAboveRoot => ffi::APR_EABOVEROOT as i32,
            Self::AprBadPath => ffi::APR_EBADPATH as i32,
            Self::AprPathWildcard => ffi::APR_EPATHWILD as i32,
            Self::AprSymbolNotFound => ffi::APR_ESYMNOTFOUND as i32,
            Self::AprProcUnknown => ffi::APR_EPROC_UNKNOWN as i32,
            Self::AprNotEnoughEntropy => ffi::APR_ENOTENOUGHENTROPY as i32,
            Self::AprInChild => ffi::APR_INCHILD as i32,
            Self::AprInParent => ffi::APR_INPARENT as i32,
            Self::AprDetached => ffi::APR_DETACH as i32,
            Self::AprNotDetached => ffi::APR_NOTDETACH as i32,
            Self::AprChildDone => ffi::APR_CHILD_DONE as i32,
            Self::AprChildNotDone => ffi::APR_CHILD_NOTDONE as i32,
            Self::AprTimeUp => ffi::APR_TIMEUP as i32,
            Self::AprIncomplete => ffi::APR_INCOMPLETE as i32,
            Self::AprBadOption => ffi::APR_BADCH as i32,
            Self::AprBadArg => ffi::APR_BADARG as i32,
            Self::AprEof => ffi::APR_EOF as i32,
            Self::AprNotFound => ffi::APR_NOTFOUND as i32,
            Self::AprAnonymousShm => ffi::APR_ANONYMOUS as i32,
            Self::AprFileBasedShm => ffi::APR_FILEBASED as i32,
            Self::AprKeyBasedShm => ffi::APR_KEYBASED as i32,
            Self::AprInit => ffi::APR_EINIT as i32,
            Self::AprNotImplemented => ffi::APR_ENOTIMPL as i32,
            Self::AprMismatch => ffi::APR_EMISMATCH as i32,
            Self::AprBusy => ffi::APR_EBUSY as i32,
            Self::AprPermissionDenied => ffi::APR_EACCES as i32,
            Self::AprAlreadyExists => ffi::APR_EEXIST as i32,
            Self::AprNameTooLong => ffi::APR_ENAMETOOLONG as i32,
            Self::AprNoEntry => ffi::APR_ENOENT as i32,
            Self::AprNotDir => ffi::APR_ENOTDIR as i32,
            Self::AprNoSpace => ffi::APR_ENOSPC as i32,
            Self::AprOutOfMemory => ffi::APR_ENOMEM as i32,
            Self::AprTooManyOpenFiles => ffi::APR_EMFILE as i32,
            Self::AprFileTableOverflow => ffi::APR_ENFILE as i32,
            Self::AprBadFileDescriptor => ffi::APR_EBADF as i32,
            Self::AprInvalidArg => ffi::APR_EINVAL as i32,
            Self::AprIllegalSeek => ffi::APR_ESPIPE as i32,
            Self::AprWouldBlock => ffi::APR_EAGAIN as i32,
            Self::AprInterrupted => ffi::APR_EINTR as i32,
            Self::AprNotSock => ffi::APR_ENOTSOCK as i32,
            Self::AprConnectionRefused => ffi::APR_ECONNREFUSED as i32,
            Self::AprInProgress => ffi::APR_EINPROGRESS as i32,
            Self::AprConnectionAborted => ffi::APR_ECONNABORTED as i32,
            Self::AprConnectionReset => ffi::APR_ECONNRESET as i32,
            Self::AprTimedOut => ffi::APR_ETIMEDOUT as i32,
            Self::AprHostUnreachable => ffi::APR_EHOSTUNREACH as i32,
            Self::AprNetworkUnreachable => ffi::APR_ENETUNREACH as i32,
            Self::AprBadFileType => ffi::APR_EFTYPE as i32,
            Self::AprBrokenPipe => ffi::APR_EPIPE as i32,
            Self::AprCrossDevice => ffi::APR_EXDEV as i32,
            Self::AprDirNotEmpty => ffi::APR_ENOTEMPTY as i32,
            Self::AprAddressFamilyNotSupported => ffi::APR_EAFNOSUPPORT as i32,
            Self::AprOperationNotSupported => ffi::APR_EOPNOTSUPP as i32,
            Self::AprOutOfRange => ffi::APR_ERANGE as i32,
            Self::Unknown(code) => code,
        }
    }

    /// 从原始 `i32` 错误码构建，未知值归入 `Unknown`
    #[must_use]
    pub fn from_i32(code: i32) -> Self {
        match code {
            v if v == ffi::svn_errno_t_SVN_WARNING as i32 => Self::Warning,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_CONTAINING_POOL as i32 => {
                Self::BadContainingPool
            }
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_FILENAME as i32 => Self::BadFilename,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_URL as i32 => Self::BadUrl,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_DATE as i32 => Self::BadDate,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_MIME_TYPE as i32 => Self::BadMimeType,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE as i32 => Self::BadPropertyValue,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_RELATIVE_PATH as i32 => Self::BadRelativePath,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_UUID as i32 => Self::BadUuid,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_CONFIG_VALUE as i32 => Self::BadConfigValue,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_KIND as i32 => Self::BadChecksumKind,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_PARSE as i32 => Self::BadChecksumParse,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_TOKEN as i32 => Self::BadToken,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_CHANGELIST_NAME as i32 => {
                Self::BadChangelistName
            }
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_ATOMIC as i32 => Self::BadAtomic,
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE_EOL as i32 => {
                Self::BadPropertyValueEol
            }
            v if v == ffi::svn_errno_t_SVN_ERR_XML_ATTRIB_NOT_FOUND as i32 => {
                Self::XmlAttribNotFound
            }
            v if v == ffi::svn_errno_t_SVN_ERR_XML_MISSING_ANCESTRY as i32 => {
                Self::XmlMissingAncestry
            }
            v if v == ffi::svn_errno_t_SVN_ERR_XML_UNKNOWN_ENCODING as i32 => {
                Self::XmlUnknownEncoding
            }
            v if v == ffi::svn_errno_t_SVN_ERR_XML_MALFORMED as i32 => Self::XmlMalformed,
            v if v == ffi::svn_errno_t_SVN_ERR_XML_UNESCAPABLE_DATA as i32 => {
                Self::XmlUnescapableData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_IO_INCONSISTENT_EOL as i32 => {
                Self::IoInconsistentEol
            }
            v if v == ffi::svn_errno_t_SVN_ERR_IO_UNKNOWN_EOL as i32 => Self::IoUnknownEol,
            v if v == ffi::svn_errno_t_SVN_ERR_IO_CORRUPT_EOL as i32 => Self::IoCorruptEol,
            v if v == ffi::svn_errno_t_SVN_ERR_IO_PIPE_FRAME_ERROR as i32 => Self::IoPipeFrameError,
            v if v == ffi::svn_errno_t_SVN_ERR_IO_PIPE_READ_ERROR as i32 => Self::IoPipeReadError,
            v if v == ffi::svn_errno_t_SVN_ERR_IO_WRITE_ERROR as i32 => Self::IoWriteError,
            v if v == ffi::svn_errno_t_SVN_ERR_IO_PIPE_WRITE_ERROR as i32 => Self::IoPipeWriteError,
            v if v == ffi::svn_errno_t_SVN_ERR_STREAM_UNEXPECTED_EOF as i32 => {
                Self::StreamUnexpectedEof
            }
            v if v == ffi::svn_errno_t_SVN_ERR_STREAM_MALFORMED_DATA as i32 => {
                Self::StreamMalformedData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_STREAM_NOT_SUPPORTED as i32 => {
                Self::StreamNotSupported
            }
            v if v == ffi::svn_errno_t_SVN_ERR_NODE_UNKNOWN_KIND as i32 => Self::NodeUnknownKind,
            v if v == ffi::svn_errno_t_SVN_ERR_NODE_UNEXPECTED_KIND as i32 => {
                Self::NodeUnexpectedKind
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ENTRY_NOT_FOUND as i32 => Self::EntryNotFound,
            v if v == ffi::svn_errno_t_SVN_ERR_ENTRY_EXISTS as i32 => Self::EntryExists,
            v if v == ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_URL as i32 => Self::EntryMissingUrl,
            v if v == ffi::svn_errno_t_SVN_ERR_ENTRY_FORBIDDEN as i32 => Self::EntryForbidden,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_OBSTRUCTED_UPDATE as i32 => {
                Self::WcObstructedUpdate
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_UNWIND_MISMATCH as i32 => Self::WcUnwindMismatch,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_UNWIND_EMPTY as i32 => Self::WcUnwindEmpty,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_UNWIND_NOT_EMPTY as i32 => Self::WcUnwindNotEmpty,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_LOCKED as i32 => Self::WcLocked,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_NOT_LOCKED as i32 => Self::WcNotLocked,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_INVALID_LOCK as i32 => Self::WcInvalidLock,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_NOT_WORKING_COPY as i32 => {
                Self::WcNotWorkingCopyOrDirectory
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_NOT_FILE as i32 => Self::WcNotFile,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG as i32 => Self::WcBadAdmLog,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_PATH_NOT_FOUND as i32 => Self::WcPathNotFound,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_NOT_UP_TO_DATE as i32 => Self::WcNotUpToDate,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_LEFT_LOCAL_MOD as i32 => Self::WcLeftLocalMod,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_SCHEDULE_CONFLICT as i32 => {
                Self::WcScheduleConflict
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_PATH_FOUND as i32 => Self::WcPathFound,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_FOUND_CONFLICT as i32 => Self::WcFoundConflict,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_CORRUPT as i32 => Self::WcCorrupt,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_CORRUPT_TEXT_BASE as i32 => {
                Self::WcCorruptTextBase
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_NODE_KIND_CHANGE as i32 => Self::WcNodeKindChange,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_INVALID_OP_ON_CWD as i32 => {
                Self::WcInvalidOpOnCwd
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG_START as i32 => {
                Self::WcBadAdmLogStart
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_UNSUPPORTED_FORMAT as i32 => {
                Self::WcUnsupportedFormat
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_BAD_PATH as i32 => Self::WcBadPath,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_INVALID_SCHEDULE as i32 => {
                Self::WcInvalidSchedule
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_INVALID_RELOCATION as i32 => {
                Self::WcInvalidRelocation
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_INVALID_SWITCH as i32 => Self::WcInvalidSwitch,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_CHANGELIST_MOVE as i32 => Self::WcChangelistMove,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_DB_ERROR as i32 => Self::WcDbError,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_MISSING as i32 => Self::WcMissing,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_NOT_SYMLINK as i32 => Self::WcNotSymlink,
            v if v == ffi::svn_errno_t_SVN_ERR_WC_UPGRADE_REQUIRED as i32 => {
                Self::WcUpgradeRequired
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_CLEANUP_REQUIRED as i32 => {
                Self::WcCleanupRequired
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_PATH_ACCESS_DENIED as i32 => {
                Self::WcPathAccessDenied
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_MIXED_REVISIONS as i32 => Self::WcMixedRevisions,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_GENERAL as i32 => Self::FsGeneral,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_CLEANUP as i32 => Self::FsCleanup,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_ALREADY_OPEN as i32 => Self::FsAlreadyOpen,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_OPEN as i32 => Self::FsNotOpen,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_CORRUPT as i32 => Self::FsCorrupt,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_PATH_SYNTAX as i32 => Self::FsPathSyntax,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REVISION as i32 => Self::FsNoSuchRevision,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_TRANSACTION as i32 => {
                Self::FsNoSuchTransaction
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_ENTRY as i32 => Self::FsNoSuchEntry,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_STRING as i32 => Self::FsNoSuchString,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_COPY as i32 => Self::FsNoSuchCopy,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_FOUND as i32 => Self::FsNotFound,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_ID_NOT_FOUND as i32 => Self::FsIdNotFound,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_ID as i32 => Self::FsNotId,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_DIRECTORY as i32 => Self::FsNotDirectory,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_FILE as i32 => Self::FsNotFile,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_MUTABLE as i32 => Self::FsNotMutable,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_ALREADY_EXISTS as i32 => Self::FsAlreadyExists,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_ROOT_DIR as i32 => Self::FsRootDir,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_TXN_ROOT as i32 => Self::FsNotTxnRoot,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_REVISION_ROOT as i32 => {
                Self::FsNotRevisionRoot
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_CONFLICT as i32 => Self::FsConflict,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_REP_CHANGED as i32 => Self::FsRepChanged,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_REP_NOT_MUTABLE as i32 => Self::FsRepNotMutable,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_SKEL as i32 => Self::FsMalformedSkel,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_TXN_OUT_OF_DATE as i32 => Self::FsTxnOutOfDate,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB as i32 => Self::FsBerkeleyDb,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_DEAD as i32 => {
                Self::FsTransactionDead
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_UNKNOWN_FS_TYPE as i32 => Self::FsUnknownFsType,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_USER as i32 => Self::FsNoUser,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_PATH_ALREADY_LOCKED as i32 => {
                Self::FsPathAlreadyLocked
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_PATH_NOT_LOCKED as i32 => Self::FsPathNotLocked,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_BAD_LOCK_TOKEN as i32 => Self::FsBadLockToken,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_LOCK_TOKEN as i32 => Self::FsNoLockToken,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_LOCK_OWNER_MISMATCH as i32 => {
                Self::FsLockOwnerMismatch
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_LOCK as i32 => Self::FsNoSuchLock,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_LOCK_EXPIRED as i32 => Self::FsLockExpired,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_OUT_OF_DATE as i32 => Self::FsOutOfDate,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_FORMAT as i32 => {
                Self::FsUnsupportedFormat
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_REP_BEING_WRITTEN as i32 => {
                Self::FsRepBeingWritten
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_TXN_NAME_TOO_LONG as i32 => {
                Self::FsTxnNameTooLong
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_NODE_ORIGIN as i32 => {
                Self::FsNoSuchNodeOrigin
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_TXN_ID as i32 => Self::FsMalformedTxnId,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_INDEX_CORRUPTION as i32 => {
                Self::FsIndexCorruption
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_INDEX_REVISION as i32 => Self::FsIndexRevision,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_INDEX_OVERFLOW as i32 => Self::FsIndexOverflow,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_INDEX as i32 => Self::FsContainerIndex,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_INDEX_INCONSISTENT as i32 => {
                Self::FsIndexInconsistent
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_TYPE as i32 => {
                Self::FsUnsupportedType
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_SIZE as i32 => Self::FsContainerSize,
            v if v == ffi::svn_errno_t_SVN_ERR_FS_INVALID_GENERATION as i32 => {
                Self::FsInvalidGeneration
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_PROPLIST as i32 => {
                Self::FsCorruptProplist
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_LOCKED as i32 => Self::ReposLocked,
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_HOOK_FAILURE as i32 => Self::ReposHookFailure,
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_BAD_ARGS as i32 => Self::ReposBadArgs,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_ILLEGAL_URL as i32 => Self::RaIllegalUrl,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_NOT_AUTHORIZED as i32 => Self::RaNotAuthorized,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_UNKNOWN_AUTH as i32 => Self::RaUnknownAuth,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_NOT_IMPLEMENTED as i32 => Self::RaNotImplemented,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_OUT_OF_DATE as i32 => Self::RaOutOfDate,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_NO_REPOS_UUID as i32 => Self::RaNoReposUuid,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_NOT_LOCKED as i32 => Self::RaNotLocked,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_UUID_MISMATCH as i32 => Self::RaUuidMismatch,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_SOCK_INIT as i32 => Self::RaDavSockInit,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_REQUEST_FAILED as i32 => {
                Self::RaDavRequestFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPS_NOT_FOUND as i32 => {
                Self::RaDavPropsNotFound
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_ALREADY_EXISTS as i32 => {
                Self::RaDavAlreadyExists
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_PATH_NOT_FOUND as i32 => {
                Self::RaDavPathNotFound
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_MALFORMED_DATA as i32 => {
                Self::RaDavMalformedData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_RELOCATED as i32 => Self::RaDavRelocated,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_CONN_TIMEOUT as i32 => Self::RaDavConnTimeout,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_FORBIDDEN as i32 => Self::RaDavForbidden,
            v if v == ffi::svn_errno_t_SVN_ERR_SVNDIFF_BACKWARD_VIEW as i32 => {
                Self::SvndiffBackwardView
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_OPS as i32 => {
                Self::SvndiffInvalidOps
            }
            v if v == ffi::svn_errno_t_SVN_ERR_APMOD_MALFORMED_URI as i32 => {
                Self::ApmodMalformedUri
            }
            v if v == ffi::svn_errno_t_SVN_ERR_APMOD_BAD_BASELINE as i32 => Self::ApmodBadBaseline,
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_BAD_REVISION as i32 => {
                Self::ClientBadRevision
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_IS_BINARY_FILE as i32 => {
                Self::ClientIsBinaryFile
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_MODIFIED as i32 => Self::ClientModified,
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_IS_DIRECTORY as i32 => {
                Self::ClientIsDirectory
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_RANGE as i32 => {
                Self::ClientRevisionRange
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_PROPERTY_NAME as i32 => {
                Self::ClientPropertyName
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_CYCLE_DETECTED as i32 => {
                Self::ClientCycleDetected
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_NO_LOCK_TOKEN as i32 => {
                Self::ClientNoLockToken
            }
            v if v == ffi::svn_errno_t_SVN_ERR_BASE as i32 => Self::Base,
            v if v == ffi::svn_errno_t_SVN_ERR_PLUGIN_LOAD_FAILURE as i32 => {
                Self::PluginLoadFailure
            }
            v if v == ffi::svn_errno_t_SVN_ERR_MALFORMED_FILE as i32 => Self::MalformedFile,
            v if v == ffi::svn_errno_t_SVN_ERR_INCOMPLETE_DATA as i32 => Self::IncompleteData,
            v if v == ffi::svn_errno_t_SVN_ERR_INCORRECT_PARAMS as i32 => Self::IncorrectParams,
            v if v == ffi::svn_errno_t_SVN_ERR_UNVERSIONED_RESOURCE as i32 => {
                Self::UnversionedResource
            }
            v if v == ffi::svn_errno_t_SVN_ERR_TEST_FAILED as i32 => Self::TestFailed,
            v if v == ffi::svn_errno_t_SVN_ERR_UNSUPPORTED_FEATURE as i32 => {
                Self::UnsupportedFeature
            }
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_PROP_KIND as i32 => Self::BadPropKind,
            v if v == ffi::svn_errno_t_SVN_ERR_ILLEGAL_TARGET as i32 => Self::IllegalTarget,
            v if v == ffi::svn_errno_t_SVN_ERR_DIR_NOT_EMPTY as i32 => Self::DirNotEmpty,
            v if v == ffi::svn_errno_t_SVN_ERR_EXTERNAL_PROGRAM as i32 => Self::ExternalProgram,
            v if v == ffi::svn_errno_t_SVN_ERR_SWIG_PY_EXCEPTION_SET as i32 => {
                Self::SwigPyExceptionSet
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CHECKSUM_MISMATCH as i32 => Self::ChecksumMismatch,
            v if v == ffi::svn_errno_t_SVN_ERR_CANCELLED as i32 => Self::Cancelled,
            v if v == ffi::svn_errno_t_SVN_ERR_INVALID_DIFF_OPTION as i32 => {
                Self::InvalidDiffOption
            }
            v if v == ffi::svn_errno_t_SVN_ERR_PROPERTY_NOT_FOUND as i32 => Self::PropertyNotFound,
            v if v == ffi::svn_errno_t_SVN_ERR_NO_AUTH_FILE_PATH as i32 => Self::NoAuthFilePath,
            v if v == ffi::svn_errno_t_SVN_ERR_VERSION_MISMATCH as i32 => Self::VersionMismatch,
            v if v == ffi::svn_errno_t_SVN_ERR_MERGEINFO_PARSE_ERROR as i32 => {
                Self::MergeinfoParseError
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CEASE_INVOCATION as i32 => Self::CeaseInvocation,
            v if v == ffi::svn_errno_t_SVN_ERR_REVNUM_PARSE_FAILURE as i32 => {
                Self::RevnumParseFailure
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ITER_BREAK as i32 => Self::IterBreak,
            v if v == ffi::svn_errno_t_SVN_ERR_UNKNOWN_CHANGELIST as i32 => Self::UnknownChangelist,
            v if v == ffi::svn_errno_t_SVN_ERR_UNKNOWN_CAPABILITY as i32 => Self::UnknownCapability,
            v if v == ffi::svn_errno_t_SVN_ERR_TEST_SKIPPED as i32 => Self::TestSkipped,
            v if v == ffi::svn_errno_t_SVN_ERR_NO_APR_MEMCACHE as i32 => Self::NoAprMemcache,
            v if v == ffi::svn_errno_t_SVN_ERR_ATOMIC_INIT_FAILURE as i32 => {
                Self::AtomicInitFailure
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SQLITE_ERROR as i32 => Self::SqliteError,
            v if v == ffi::svn_errno_t_SVN_ERR_SQLITE_READONLY as i32 => Self::SqliteReadonly,
            v if v == ffi::svn_errno_t_SVN_ERR_SQLITE_BUSY as i32 => Self::SqliteBusy,
            v if v == ffi::svn_errno_t_SVN_ERR_SQLITE_CONSTRAINT as i32 => Self::SqliteConstraint,
            v if v == ffi::svn_errno_t_SVN_ERR_UTF8PROC_ERROR as i32 => Self::Utf8procError,
            v if v == ffi::svn_errno_t_SVN_ERR_UTF8_GLOB as i32 => Self::Utf8Glob,
            v if v == ffi::svn_errno_t_SVN_ERR_CORRUPT_PACKED_DATA as i32 => {
                Self::CorruptPackedData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_COMPOSED_ERROR as i32 => Self::ComposedError,
            v if v == ffi::svn_errno_t_SVN_ERR_INVALID_INPUT as i32 => Self::InvalidInput,
            v if v == ffi::svn_errno_t_SVN_ERR_CL_ARG_PARSING_ERROR as i32 => {
                Self::ClArgParsingError
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_INSUFFICIENT_ARGS as i32 => {
                Self::ClInsufficientArgs
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_ADM_DIR_RESERVED as i32 => Self::ClAdmDirReserved,
            v if v == ffi::svn_errno_t_SVN_ERR_CL_COMMIT_IN_ADDED_DIR as i32 => {
                Self::ClCommitInAddedDir
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_EDITOR as i32 => {
                Self::ClNoExternalEditor
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_BAD_LOG_MESSAGE as i32 => Self::ClBadLogMessage,
            v if v == ffi::svn_errno_t_SVN_ERR_CL_REPOS_VERIFY_FAILED as i32 => {
                Self::ClReposVerifyFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_CMD_ERR as i32 => Self::RaSvnCmdErr,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_UNKNOWN_CMD as i32 => Self::RaSvnUnknownCmd,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_IO_ERROR as i32 => Self::RaSvnIoError,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_MALFORMED_DATA as i32 => {
                Self::RaSvnMalformedData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_REPOS_NOT_FOUND as i32 => {
                Self::RaSvnReposNotFound
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_BAD_VERSION as i32 => Self::RaSvnBadVersion,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_NO_MECHANISMS as i32 => {
                Self::RaSvnNoMechanisms
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_EDIT_ABORTED as i32 => Self::RaSvnEditAborted,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_REQUEST_SIZE as i32 => Self::RaSvnRequestSize,
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_RESPONSE_SIZE as i32 => {
                Self::RaSvnResponseSize
            }
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHN_NO_PROVIDER as i32 => Self::AuthnNoProvider,
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_NOT_SAVED as i32 => {
                Self::AuthnCredsNotSaved
            }
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHN_FAILED as i32 => Self::AuthnFailed,
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHZ_ROOT_UNREADABLE as i32 => {
                Self::AuthzRootUnreadable
            }
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHZ_UNREADABLE as i32 => Self::AuthzUnreadable,
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHZ_INVALID_CONFIG as i32 => {
                Self::AuthzInvalidConfig
            }
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHZ_UNWRITABLE as i32 => Self::AuthzUnwritable,
            v if v == ffi::svn_errno_t_SVN_ERR_DIFF_UNEXPECTED_DATA as i32 => {
                Self::DiffUnexpectedData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SERF_WRAPPED_ERROR as i32 => {
                Self::RaSerfWrappedError
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ASSERTION_FAIL as i32 => Self::AssertionFail,
            v if v == ffi::svn_errno_t_SVN_ERR_ASN1_OUT_OF_DATA as i32 => Self::Asn1OutOfData,
            v if v == ffi::svn_errno_t_SVN_ERR_ASN1_UNEXPECTED_TAG as i32 => {
                Self::Asn1UnexpectedTag
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_LENGTH as i32 => {
                Self::Asn1InvalidLength
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ASN1_LENGTH_MISMATCH as i32 => {
                Self::Asn1LengthMismatch
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_DATA as i32 => Self::Asn1InvalidData,
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PEM as i32 => {
                Self::X509CertInvalidPem
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_ALG as i32 => {
                Self::X509CertInvalidAlg
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_NAME as i32 => {
                Self::X509CertInvalidName
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_DATE as i32 => {
                Self::X509CertInvalidDate
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_SIG_MISMATCH as i32 => {
                Self::X509CertSigMismatch
            }
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_VERSION_FILE_FORMAT as i32 => {
                Self::BadVersionFileFormat
            }
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_SERVER_SPECIFICATION as i32 => {
                Self::BadServerSpecification
            }
            v if v == ffi::svn_errno_t_SVN_ERR_BAD_COMPRESSION_METHOD as i32 => {
                Self::BadCompressionMethod
            }
            v if v == ffi::svn_errno_t_SVN_ERR_XML_UNEXPECTED_ELEMENT as i32 => {
                Self::XmlUnexpectedElement
            }
            v if v == ffi::svn_errno_t_SVN_ERR_IO_UNIQUE_NAMES_EXHAUSTED as i32 => {
                Self::IoUniqueNamesExhausted
            }
            v if v == ffi::svn_errno_t_SVN_ERR_STREAM_UNRECOGNIZED_DATA as i32 => {
                Self::StreamUnrecognizedData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_STREAM_SEEK_NOT_SUPPORTED as i32 => {
                Self::StreamSeekNotSupported
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_REVISION as i32 => {
                Self::EntryMissingRevision
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ENTRY_ATTRIBUTE_INVALID as i32 => {
                Self::EntryAttributeInvalid
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_MISMATCHED_CHANGELIST as i32 => {
                Self::WcMismatchedChangelist
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_CONFLICT_RESOLVER_FAILURE as i32 => {
                Self::WcConflictResolverFailure
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_COPYFROM_PATH_NOT_FOUND as i32 => {
                Self::WcCopyfromPathNotFound
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_CANNOT_DELETE_FILE_EXTERNAL as i32 => {
                Self::WcCannotDeleteFileExternal
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_CANNOT_MOVE_FILE_EXTERNAL as i32 => {
                Self::WcCannotMoveFileExternal
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_PATH_UNEXPECTED_STATUS as i32 => {
                Self::WcPathUnexpectedStatus
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_INVALID_OPERATION_DEPTH as i32 => {
                Self::WcInvalidOperationDepth
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_DUPLICATE_EXTERNALS_TARGET as i32 => {
                Self::WcDuplicateExternalsTarget
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_INCOMPATIBLE_SETTINGS as i32 => {
                Self::WcIncompatibleSettings
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_DEPRECATED_API_STORE_PRISTINE as i32 => {
                Self::WcDeprecatedApiStorePristine
            }
            v if v == ffi::svn_errno_t_SVN_ERR_WC_PRISTINE_DEHYDRATED as i32 => {
                Self::WcPristineDehydrated
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REPRESENTATION as i32 => {
                Self::FsNoSuchRepresentation
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_MUTABLE as i32 => {
                Self::FsTransactionNotMutable
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NOT_SINGLE_PATH_COMPONENT as i32 => {
                Self::FsNotSinglePathComponent
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB_DEADLOCK as i32 => {
                Self::FsBerkeleyDbDeadlock
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_DEAD as i32 => {
                Self::FsTransactionNotDead
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_UPGRADE as i32 => {
                Self::FsUnsupportedUpgrade
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_CHECKSUM_REP as i32 => {
                Self::FsNoSuchChecksumRep
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_PROP_BASEVALUE_MISMATCH as i32 => {
                Self::FsPropBasevalueMismatch
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_INCORRECT_EDITOR_COMPLETION as i32 => {
                Self::FsIncorrectEditorCompletion
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_PACKED_REVPROP_READ_FAILURE as i32 => {
                Self::FsPackedRevpropReadFailure
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_REVPROP_CACHE_INIT_FAILURE as i32 => {
                Self::FsRevpropCacheInitFailure
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_LOCK_OPERATION_FAILED as i32 => {
                Self::FsLockOperationFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_NODEREV_ID as i32 => {
                Self::FsMalformedNoderevId
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_REVPROP_MANIFEST as i32 => {
                Self::FsCorruptRevpropManifest
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_AMBIGUOUS_CHECKSUM_REP as i32 => {
                Self::FsAmbiguousChecksumRep
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_UNRECOGNIZED_IOCTL_CODE as i32 => {
                Self::FsUnrecognizedIoctlCode
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_ALLOWED as i32 => {
                Self::FsRepSharingNotAllowed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_SUPPORTED as i32 => {
                Self::FsRepSharingNotSupported
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_NO_DATA_FOR_REPORT as i32 => {
                Self::ReposNoDataForReport
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_BAD_REVISION_REPORT as i32 => {
                Self::ReposBadRevisionReport
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_VERSION as i32 => {
                Self::ReposUnsupportedVersion
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_DISABLED_FEATURE as i32 => {
                Self::ReposDisabledFeature
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_POST_COMMIT_HOOK_FAILED as i32 => {
                Self::ReposPostCommitHookFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_POST_LOCK_HOOK_FAILED as i32 => {
                Self::ReposPostLockHookFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_POST_UNLOCK_HOOK_FAILED as i32 => {
                Self::ReposPostUnlockHookFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_UPGRADE as i32 => {
                Self::ReposUnsupportedUpgrade
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_UNSUPPORTED_ABI_VERSION as i32 => {
                Self::RaUnsupportedAbiVersion
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_PARTIAL_REPLAY_NOT_SUPPORTED as i32 => {
                Self::RaPartialReplayNotSupported
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_REPOS_ROOT_URL_MISMATCH as i32 => {
                Self::RaReposRootUrlMismatch
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SESSION_URL_MISMATCH as i32 => {
                Self::RaSessionUrlMismatch
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_TUNNEL as i32 => {
                Self::RaCannotCreateTunnel
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_SESSION as i32 => {
                Self::RaCannotCreateSession
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_CREATING_REQUEST as i32 => {
                Self::RaDavCreatingRequest
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_OPTIONS_REQ_FAILED as i32 => {
                Self::RaDavOptionsReqFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_INVALID_CONFIG_VALUE as i32 => {
                Self::RaDavInvalidConfigValue
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPPATCH_FAILED as i32 => {
                Self::RaDavProppatchFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_RESPONSE_HEADER_BADNESS as i32 => {
                Self::RaDavResponseHeaderBadness
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_PRECONDITION_FAILED as i32 => {
                Self::RaDavPreconditionFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_DAV_METHOD_NOT_ALLOWED as i32 => {
                Self::RaDavMethodNotAllowed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_NOT_FOUND as i32 => {
                Self::RaLocalReposNotFound
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_OPEN_FAILED as i32 => {
                Self::RaLocalReposOpenFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_HEADER as i32 => {
                Self::SvndiffInvalidHeader
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SVNDIFF_CORRUPT_WINDOW as i32 => {
                Self::SvndiffCorruptWindow
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SVNDIFF_UNEXPECTED_END as i32 => {
                Self::SvndiffUnexpectedEnd
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_COMPRESSED_DATA as i32 => {
                Self::SvndiffInvalidCompressedData
            }
            v if v == ffi::svn_errno_t_SVN_ERR_APMOD_MISSING_PATH_TO_FS as i32 => {
                Self::ApmodMissingPathToFs
            }
            v if v == ffi::svn_errno_t_SVN_ERR_APMOD_ACTIVITY_NOT_FOUND as i32 => {
                Self::ApmodActivityNotFound
            }
            v if v == ffi::svn_errno_t_SVN_ERR_APMOD_CONNECTION_ABORTED as i32 => {
                Self::ApmodConnectionAborted
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_VERSIONED_PATH_REQUIRED as i32 => {
                Self::ClientVersionedPathRequired
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_RA_ACCESS_REQUIRED as i32 => {
                Self::ClientRaAccessRequired
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_DUPLICATE_COMMIT_URL as i32 => {
                Self::ClientDuplicateCommitUrl
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_EXTERNALS_DESCRIPTION as i32 => {
                Self::ClientInvalidExternalsDescription
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_RELOCATION as i32 => {
                Self::ClientInvalidRelocation
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_AUTHOR_CONTAINS_NEWLINE as i32 => {
                Self::ClientRevisionAuthorContainsNewline
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_UNRELATED_RESOURCES as i32 => {
                Self::ClientUnrelatedResources
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_MISSING_LOCK_TOKEN as i32 => {
                Self::ClientMissingLockToken
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_MULTIPLE_SOURCES_DISALLOWED as i32 => {
                Self::ClientMultipleSourcesDisallowed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_NO_VERSIONED_PARENT as i32 => {
                Self::ClientNoVersionedParent
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_NOT_READY_TO_MERGE as i32 => {
                Self::ClientNotReadyToMerge
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_FILE_EXTERNAL_OVERWRITE_VERSIONED as i32 => {
                Self::ClientFileExternalOverwriteVersioned
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_PATCH_BAD_STRIP_COUNT as i32 => {
                Self::ClientPatchBadStripCount
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_MERGE_UPDATE_REQUIRED as i32 => {
                Self::ClientMergeUpdateRequired
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_MERGEINFO_NO_MERGETRACKING as i32 => {
                Self::ClientInvalidMergeinfoNoMergetracking
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_FORBIDDEN_BY_SERVER as i32 => {
                Self::ClientForbiddenByServer
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CLIENT_CONFLICT_OPTION_NOT_APPLICABLE as i32 => {
                Self::ClientConflictOptionNotApplicable
            }
            v if v == ffi::svn_errno_t_SVN_ERR_DELTA_MD5_CHECKSUM_ABSENT as i32 => {
                Self::DeltaMd5ChecksumAbsent
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RESERVED_FILENAME_SPECIFIED as i32 => {
                Self::ReservedFilenameSpecified
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SQLITE_UNSUPPORTED_SCHEMA as i32 => {
                Self::SqliteUnsupportedSchema
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SQLITE_RESETTING_FOR_ROLLBACK as i32 => {
                Self::SqliteResettingForRollback
            }
            v if v == ffi::svn_errno_t_SVN_ERR_TOO_MANY_MEMCACHED_SERVERS as i32 => {
                Self::TooManyMemcachedServers
            }
            v if v == ffi::svn_errno_t_SVN_ERR_MALFORMED_VERSION_STRING as i32 => {
                Self::MalformedVersionString
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CORRUPTED_ATOMIC_STORAGE as i32 => {
                Self::CorruptedAtomicStorage
            }
            v if v == ffi::svn_errno_t_SVN_ERR_SQLITE_ROLLBACK_FAILED as i32 => {
                Self::SqliteRollbackFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_LZ4_COMPRESSION_FAILED as i32 => {
                Self::Lz4CompressionFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_LZ4_DECOMPRESSION_FAILED as i32 => {
                Self::Lz4DecompressionFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CANONICALIZATION_FAILED as i32 => {
                Self::CanonicalizationFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_MUTUALLY_EXCLUSIVE_ARGS as i32 => {
                Self::ClMutuallyExclusiveArgs
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_VERSIONED_FILE as i32 => {
                Self::ClLogMessageIsVersionedFile
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_PATHNAME as i32 => {
                Self::ClLogMessageIsPathname
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_UNNECESSARY_LOG_MESSAGE as i32 => {
                Self::ClUnnecessaryLogMessage
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_MERGE_TOOL as i32 => {
                Self::ClNoExternalMergeTool
            }
            v if v == ffi::svn_errno_t_SVN_ERR_CL_ERROR_PROCESSING_EXTERNALS as i32 => {
                Self::ClErrorProcessingExternals
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SVN_CONNECTION_CLOSED as i32 => {
                Self::RaSvnConnectionClosed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_UNAVAILABLE as i32 => {
                Self::AuthnCredsUnavailable
            }
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHN_PROVIDERS_EXHAUSTED as i32 => {
                Self::AuthnProvidersExhausted
            }
            v if v == ffi::svn_errno_t_SVN_ERR_AUTHZ_PARTIALLY_READABLE as i32 => {
                Self::AuthzPartiallyReadable
            }
            v if v == ffi::svn_errno_t_SVN_ERR_DIFF_DATASOURCE_MODIFIED as i32 => {
                Self::DiffDatasourceModified
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SERF_SSPI_INITIALISATION_FAILED as i32 => {
                Self::RaSerfSspiInitialisationFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SERF_SSL_CERT_UNTRUSTED as i32 => {
                Self::RaSerfSslCertUntrusted
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SERF_GSSAPI_INITIALISATION_FAILED as i32 => {
                Self::RaSerfGssapiInitialisationFailed
            }
            v if v == ffi::svn_errno_t_SVN_ERR_RA_SERF_STREAM_BUCKET_READ_ERROR as i32 => {
                Self::RaSerfStreamBucketReadError
            }
            v if v == ffi::svn_errno_t_SVN_ERR_ASSERTION_ONLY_TRACING_LINKS as i32 => {
                Self::AssertionOnlyTracingLinks
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_FEATURE_UNAVAILABLE as i32 => {
                Self::X509FeatureUnavailable
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_FORMAT as i32 => {
                Self::X509CertInvalidFormat
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_VERSION as i32 => {
                Self::X509CertInvalidVersion
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SERIAL as i32 => {
                Self::X509CertInvalidSerial
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PUBKEY as i32 => {
                Self::X509CertInvalidPubkey
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SIGNATURE as i32 => {
                Self::X509CertInvalidSignature
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_EXTENSIONS as i32 => {
                Self::X509CertInvalidExtensions
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_VERSION as i32 => {
                Self::X509CertUnknownVersion
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_PK_ALG as i32 => {
                Self::X509CertUnknownPkAlg
            }
            v if v == ffi::svn_errno_t_SVN_ERR_X509_CERT_VERIFY_FAILED as i32 => {
                Self::X509CertVerifyFailed
            }
            v if v == ffi::APR_ENOSTAT as i32 => Self::AprNoStat,
            v if v == ffi::APR_ENOPOOL as i32 => Self::AprNoPool,
            v if v == ffi::APR_EBADDATE as i32 => Self::AprBadDate,
            v if v == ffi::APR_EINVALSOCK as i32 => Self::AprInvalidSocket,
            v if v == ffi::APR_ENOPROC as i32 => Self::AprNoProc,
            v if v == ffi::APR_ENOTIME as i32 => Self::AprNoTime,
            v if v == ffi::APR_ENODIR as i32 => Self::AprNoDir,
            v if v == ffi::APR_ENOLOCK as i32 => Self::AprNoLock,
            v if v == ffi::APR_ENOPOLL as i32 => Self::AprNoPoll,
            v if v == ffi::APR_ENOSOCKET as i32 => Self::AprNoSocket,
            v if v == ffi::APR_ENOTHREAD as i32 => Self::AprNoThread,
            v if v == ffi::APR_ENOTHDKEY as i32 => Self::AprNoThreadKey,
            v if v == ffi::APR_EGENERAL as i32 => Self::AprGeneral,
            v if v == ffi::APR_ENOSHMAVAIL as i32 => Self::AprNoSharedMemory,
            v if v == ffi::APR_EBADIP as i32 => Self::AprBadIp,
            v if v == ffi::APR_EBADMASK as i32 => Self::AprBadMask,
            v if v == ffi::APR_EDSOOPEN as i32 => Self::AprDsoOpen,
            v if v == ffi::APR_EABSOLUTE as i32 => Self::AprAbsolutePath,
            v if v == ffi::APR_ERELATIVE as i32 => Self::AprRelativePath,
            v if v == ffi::APR_EINCOMPLETE as i32 => Self::AprIncompletePath,
            v if v == ffi::APR_EABOVEROOT as i32 => Self::AprAboveRoot,
            v if v == ffi::APR_EBADPATH as i32 => Self::AprBadPath,
            v if v == ffi::APR_EPATHWILD as i32 => Self::AprPathWildcard,
            v if v == ffi::APR_ESYMNOTFOUND as i32 => Self::AprSymbolNotFound,
            v if v == ffi::APR_EPROC_UNKNOWN as i32 => Self::AprProcUnknown,
            v if v == ffi::APR_ENOTENOUGHENTROPY as i32 => Self::AprNotEnoughEntropy,
            v if v == ffi::APR_INCHILD as i32 => Self::AprInChild,
            v if v == ffi::APR_INPARENT as i32 => Self::AprInParent,
            v if v == ffi::APR_DETACH as i32 => Self::AprDetached,
            v if v == ffi::APR_NOTDETACH as i32 => Self::AprNotDetached,
            v if v == ffi::APR_CHILD_DONE as i32 => Self::AprChildDone,
            v if v == ffi::APR_CHILD_NOTDONE as i32 => Self::AprChildNotDone,
            v if v == ffi::APR_TIMEUP as i32 => Self::AprTimeUp,
            v if v == ffi::APR_INCOMPLETE as i32 => Self::AprIncomplete,
            v if v == ffi::APR_BADCH as i32 => Self::AprBadOption,
            v if v == ffi::APR_BADARG as i32 => Self::AprBadArg,
            v if v == ffi::APR_EOF as i32 => Self::AprEof,
            v if v == ffi::APR_NOTFOUND as i32 => Self::AprNotFound,
            v if v == ffi::APR_ANONYMOUS as i32 => Self::AprAnonymousShm,
            v if v == ffi::APR_FILEBASED as i32 => Self::AprFileBasedShm,
            v if v == ffi::APR_KEYBASED as i32 => Self::AprKeyBasedShm,
            v if v == ffi::APR_EINIT as i32 => Self::AprInit,
            v if v == ffi::APR_ENOTIMPL as i32 => Self::AprNotImplemented,
            v if v == ffi::APR_EMISMATCH as i32 => Self::AprMismatch,
            v if v == ffi::APR_EBUSY as i32 => Self::AprBusy,
            v if v == ffi::APR_EACCES as i32 => Self::AprPermissionDenied,
            v if v == ffi::APR_EEXIST as i32 => Self::AprAlreadyExists,
            v if v == ffi::APR_ENAMETOOLONG as i32 => Self::AprNameTooLong,
            v if v == ffi::APR_ENOENT as i32 => Self::AprNoEntry,
            v if v == ffi::APR_ENOTDIR as i32 => Self::AprNotDir,
            v if v == ffi::APR_ENOSPC as i32 => Self::AprNoSpace,
            v if v == ffi::APR_ENOMEM as i32 => Self::AprOutOfMemory,
            v if v == ffi::APR_EMFILE as i32 => Self::AprTooManyOpenFiles,
            v if v == ffi::APR_ENFILE as i32 => Self::AprFileTableOverflow,
            v if v == ffi::APR_EBADF as i32 => Self::AprBadFileDescriptor,
            v if v == ffi::APR_EINVAL as i32 => Self::AprInvalidArg,
            v if v == ffi::APR_ESPIPE as i32 => Self::AprIllegalSeek,
            v if v == ffi::APR_EAGAIN as i32 => Self::AprWouldBlock,
            v if v == ffi::APR_EINTR as i32 => Self::AprInterrupted,
            v if v == ffi::APR_ENOTSOCK as i32 => Self::AprNotSock,
            v if v == ffi::APR_ECONNREFUSED as i32 => Self::AprConnectionRefused,
            v if v == ffi::APR_EINPROGRESS as i32 => Self::AprInProgress,
            v if v == ffi::APR_ECONNABORTED as i32 => Self::AprConnectionAborted,
            v if v == ffi::APR_ECONNRESET as i32 => Self::AprConnectionReset,
            v if v == ffi::APR_ETIMEDOUT as i32 => Self::AprTimedOut,
            v if v == ffi::APR_EHOSTUNREACH as i32 => Self::AprHostUnreachable,
            v if v == ffi::APR_ENETUNREACH as i32 => Self::AprNetworkUnreachable,
            v if v == ffi::APR_EFTYPE as i32 => Self::AprBadFileType,
            v if v == ffi::APR_EPIPE as i32 => Self::AprBrokenPipe,
            v if v == ffi::APR_EXDEV as i32 => Self::AprCrossDevice,
            v if v == ffi::APR_ENOTEMPTY as i32 => Self::AprDirNotEmpty,
            v if v == ffi::APR_EAFNOSUPPORT as i32 => Self::AprAddressFamilyNotSupported,
            v if v == ffi::APR_EOPNOTSUPP as i32 => Self::AprOperationNotSupported,
            v if v == ffi::APR_ERANGE as i32 => Self::AprOutOfRange,
            code => Self::Unknown(code),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct SubversionErrorInfo {
    status: i32,
    code: SubversionErrorCode,
    msg: String,
    file: String,
    line: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SubversionError {
    pub msg: String,
    pub status: i32,
    pub code: SubversionErrorCode,
    pub info: Vec<SubversionErrorInfo>,
}

impl std::error::Error for SubversionError {}

impl std::fmt::Display for SubversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.msg, self.code)
    }
}

impl SubversionError {
    #[cfg(false)]
    fn to_error(&self) -> *mut ffi::svn_error_t {
        unsafe {
            let mut pool = apr::Pool::create();
            let msg = pool.string(self.msg.as_str()).unwrap_or_default();
            ffi::svn_error_create(self.code, Default::default(), msg as _)
        }
    }

    pub fn from_nullable_ptr(err: *mut ffi::svn_error_t) -> Result<(), Self> {
        if err.is_null() {
            Ok(())
        } else {
            let result = Err(Self::from_not_null_ptr(err));

            unsafe {
                ffi::svn_error_clear(err);
            }

            result
        }
    }

    fn from_not_null_ptr(err: *const ffi::svn_error_t) -> Self {
        unsafe {
            assert!(!err.is_null());

            let mut buf = [0; 512];

            let msg = ffi::svn_err_best_message(err, buf.as_mut_ptr(), (buf.len() - 1) as _);

            let msg = msg.to_str().to_string();

            let status = err.as_ref().expect("Failed to get reference").apr_err as i32;

            let code = SubversionErrorCode::from_i32(status);

            // let err = unsafe { &*err };

            let mut next = err;

            let mut info = vec![];

            while !next.is_null() {
                let err = next.as_ref().expect("Failed to get reference");

                let msg = err.message.to_nullable_string().unwrap_or_default();

                let file = err.file.to_nullable_string().unwrap_or_default();
                let error_info = SubversionErrorInfo {
                    status: err.apr_err as i32,
                    code: SubversionErrorCode::from_i32(err.apr_err as i32),
                    msg,
                    file,
                    line: err.line as _,
                };
                info.push(error_info);

                next = err.child;
            }

            Self {
                msg,
                status,
                code,
                info,
            }
        }
    }
}

const fn svn_no_error() -> *mut ffi::svn_error_t {
    ffi::SVN_NO_ERROR as *mut _
}

pub fn time_from_string(time: &str) -> error::Result<i64> {
    let mut timestamp: apr::ffi::apr_time_t = 0;
    unsafe {
        let mut pool = apr::Pool::create();
        let time = pool.string(time)?;
        let error = ffi::svn_time_from_cstring(timestamp.pointer_mut(), time, pool.as_mut_ptr());
        SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
    }
    Ok(timestamp.try_into().expect("Timestamp should be i64"))
}

#[cfg(false)]
pub fn encode_base64(bytes: &[u8], break_lines: bool) -> String {
    let mut stream = stream::Stream::create(Default::default());

    let mut input = stream.base64(break_lines);

    input.write(bytes).expect("Unexpected failure");

    input.close().expect("Unexpected failure");

    String::from_utf8(stream.take_write_buffer()).unwrap_or_default()

    // unsafe {
    //     let mut pool = apr::Pool::create();
    //     let string = ffi::svn_string_create(
    //         CString::from_str(string).unwrap().as_ptr(),
    //         pool.as_mut_ptr(),
    //     );

    //     let base64 = ffi::svn_base64_encode_string2(string, 1, pool.as_mut_ptr())
    //         .as_ref()
    //         .unwrap();
    //     std::slice::from_raw_parts(base64.data as *const u8, base64.len.try_into().unwrap())
    //         .to_vec()
    // }
}
