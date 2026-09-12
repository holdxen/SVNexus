
#pragma once

#include <stdint.h>
#include "svn_types.h"

int32_t patches_svn_apr_status_is_enotdir(int32_t status);
svn_boolean_t patches_svn_config_default_option_store_passwords();
svn_boolean_t patches_svn_config_default_option_store_auth_creds();