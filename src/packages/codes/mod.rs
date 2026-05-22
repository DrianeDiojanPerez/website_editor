// Domain-level error codes returned to clients in error responses.

pub const INTERNAL_ERROR: i32 = 1000;

pub const ERR_RESOURCE_NOT_FOUND: i32 = 2001;
pub const ERR_RESOURCE_CREATE_ERROR: i32 = 2002;
pub const ERR_RESOURCE_UPDATE_ERROR: i32 = 2003;
pub const ERR_RESOURCE_DELETE_ERROR: i32 = 2004;
pub const ERR_VALIDATION: i32 = 2005;

pub const ERR_UNAUTHORIZED: i32 = 3001;
pub const ERR_INVALID_CREDENTIALS: i32 = 3002;
pub const ERR_INVALID_REFRESH_TOKEN: i32 = 3003;
pub const ERR_PASSWORD_CHANGE_REQUIRED: i32 = 3004;
