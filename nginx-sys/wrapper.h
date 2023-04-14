#include <ngx_http.h>
#include <ngx_conf_file.h>
#include <ngx_config.h>
#include <ngx_core.h>

// Define as constants since bindgen can't parse these values
const size_t NGX_RS_HTTP_MAIN_CONF_OFFSET = NGX_HTTP_MAIN_CONF_OFFSET;
const size_t NGX_RS_HTTP_SRV_CONF_OFFSET = NGX_HTTP_SRV_CONF_OFFSET;
const size_t NGX_RS_HTTP_LOC_CONF_OFFSET = NGX_HTTP_LOC_CONF_OFFSET;

const char *NGX_RS_MODULE_SIGNATURE = NGX_MODULE_SIGNATURE;

// Wrappers for inline functions here
void ngx_add_ev_timer(ngx_event_t *ev, ngx_msec_t time) {
  ngx_add_timer(ev, time);
}

void ngx_del_ev_timer(ngx_event_t *ev) {
  ngx_event_del_timer(ev);
}
