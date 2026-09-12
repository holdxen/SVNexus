

#include "svn_patch.h"
#include "svn_types.h"
#include "svn_config.h"

int32_t patches_svn_apr_status_is_enotdir(int32_t status) {
    return SVN__APR_STATUS_IS_ENOTDIR(status);
}

svn_boolean_t patches_svn_config_default_option_store_passwords() {
    return SVN_CONFIG_DEFAULT_OPTION_STORE_PASSWORDS;
}

svn_boolean_t patches_svn_config_default_option_store_auth_creds() {
    return SVN_CONFIG_DEFAULT_OPTION_STORE_AUTH_CREDS;
}