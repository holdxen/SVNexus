
#include "apr_patch.h"
#include "apr_errno.h"

int32_t patches_apr_status_is_eacces(int32_t status) {
    return APR_STATUS_IS_EACCES(status);
}