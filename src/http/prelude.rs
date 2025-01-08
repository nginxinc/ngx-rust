use core::mem::offset_of;

use crate::ffi::*;

pub use crate::ffi::ngx_http_handler_pt;
pub use crate::ffi::ngx_http_module_t;
pub use crate::ffi::ngx_http_request_t;
pub use crate::ffi::ngx_http_variable_t;
pub use crate::ffi::ngx_http_variable_value_t;

/// The offset of the `main_conf` field in the `ngx_http_conf_ctx_t` struct.
///
/// This is used to access the main configuration context for an HTTP module.
pub const NGX_HTTP_MAIN_CONF_OFFSET: usize = offset_of!(ngx_http_conf_ctx_t, main_conf);
/// The offset of the `srv_conf` field in the `ngx_http_conf_ctx_t` struct.
///
/// This is used to access the server configuration context for an HTTP module.
pub const NGX_HTTP_SRV_CONF_OFFSET: usize = offset_of!(ngx_http_conf_ctx_t, srv_conf);
/// The offset of the `loc_conf` field in the `ngx_http_conf_ctx_t` struct.
///
/// This is used to access the location configuration context for an HTTP module.
pub const NGX_HTTP_LOC_CONF_OFFSET: usize = offset_of!(ngx_http_conf_ctx_t, loc_conf);

pub use crate::ffi::NGX_HTTP_MODULE;

pub use crate::ffi::NGX_HTTP_LIF_CONF;
pub use crate::ffi::NGX_HTTP_LMT_CONF;
pub use crate::ffi::NGX_HTTP_LOC_CONF;
pub use crate::ffi::NGX_HTTP_MAIN_CONF;
pub use crate::ffi::NGX_HTTP_SIF_CONF;
pub use crate::ffi::NGX_HTTP_SRV_CONF;
pub use crate::ffi::NGX_HTTP_UPS_CONF;

/// First phase.
///
/// The ngx_http_realip_module registers its handler at this phase to enable
/// substitution of client addresses before any other module is invoked.
pub const NGX_HTTP_POST_READ_PHASE: usize = ngx_http_phases_NGX_HTTP_POST_READ_PHASE as usize;
/// Phase where rewrite directives defined in a server block (but outside a location block) are processed.
///
/// The ngx_http_rewrite_module installs its handler at this phase.
pub const NGX_HTTP_SERVER_REWRITE_PHASE: usize = ngx_http_phases_NGX_HTTP_SERVER_REWRITE_PHASE as usize;
/// Same as NGX_HTTP_SERVER_REWRITE_PHASE, but for rewrite rules defined in the location, chosen in
/// the previous phase.
pub const NGX_HTTP_REWRITE_PHASE: usize = ngx_http_phases_NGX_HTTP_REWRITE_PHASE as usize;
/// A common phase for different types of handlers, not associated with access control.
///
/// The standard nginx modules ngx_http_limit_conn_module and ngx_http_limit_req_module register
/// their handlers at this phase.
pub const NGX_HTTP_PREACCESS_PHASE: usize = ngx_http_phases_NGX_HTTP_PREACCESS_PHASE as usize;
/// Phase where it is verified that the client is authorized to make the request.
pub const NGX_HTTP_ACCESS_PHASE: usize = ngx_http_phases_NGX_HTTP_ACCESS_PHASE as usize;
/// Phase for handlers to be called prior to generating content.
///
/// Standard modules such as ngx_http_try_files_module and ngx_http_mirror_module register their
/// handlers at this phase.
pub const NGX_HTTP_PRECONTENT_PHASE: usize = ngx_http_phases_NGX_HTTP_PRECONTENT_PHASE as usize;
/// Phase where the response is normally generated.
///
/// Multiple nginx standard modules register their handlers at this phase.
pub const NGX_HTTP_CONTENT_PHASE: usize = ngx_http_phases_NGX_HTTP_CONTENT_PHASE as usize;
/// Phase where request logging is performed.
///
/// Currently, only the ngx_http_log_module registers its handler at this stage for access logging.
pub const NGX_HTTP_LOG_PHASE: usize = ngx_http_phases_NGX_HTTP_LOG_PHASE as usize;

pub use crate::ffi::ngx_http_core_module;

pub use crate::ffi::ngx_http_add_variable;
