/**
 * @file   ngx_rust.c
 * @author Sehyo Chang <sehyo@nginx.com>
 * @date
 *
 * @brief  Dummy module
 *
 * @section LICENSE
 *
 * Copyright (C) 2011 by Nginx
 *
 */
#include <ngx_config.h>
#include <ngx_core.h>
#include <ngx_http.h>


/**
 * @brief element mixer configuration
 */
typedef struct {
    ngx_str_t name;              /**< test name */
} ngx_http_rust_main_conf_t;



static void *ngx_http_rust_create_main_conf(ngx_conf_t *cf);


/*
 * dummy rust
 */
static ngx_command_t ngx_http_rust_commands[] = {
    {
      ngx_string("rust"), /* dummy directive */
      NGX_HTTP_MAIN_CONF|NGX_CONF_TAKE1,  // server takes 1 //
      ngx_conf_set_str_slot, /* configuration setup function */
      NGX_HTTP_MAIN_CONF_OFFSET,
      offsetof(ngx_http_rust_main_conf_t, name),
      NULL
    },
    ngx_null_command /* command termination */
};


/* The module context. */
static ngx_http_module_t ngx_http_rust_module_ctx = {
    NULL, /* preconfiguration */
    NULL, /* postconfiguration */
    ngx_http_rust_create_main_conf, /* create main configuration */
    NULL, /* init main configuration */

    NULL, /* create server configuration */
    NULL, /* merge server configuration */

    NULL,
    NULL
};

/* Module definition. */
ngx_module_t ngx_http_rust_module = {
    NGX_MODULE_V1,
    &ngx_http_rust_module_ctx, /* module context */
    ngx_http_rust_commands, /* module directives */
    NGX_HTTP_MODULE, /* module type */
    NULL, /* init master */
    NULL, /* init module */
    NULL, /* init process */
    NULL, /* init thread */
    NULL, /* exit thread */
    NULL, /* exit process */
    NULL, /* exit master */
    NGX_MODULE_V1_PADDING
};



static void *ngx_http_rust_create_main_conf(ngx_conf_t *cf)
{
  ngx_http_rust_main_conf_t *conf;

  ngx_log_error(NGX_LOG_ERR, ngx_cycle->log, 0, "setting up main config");


  conf = ngx_pcalloc(cf->pool, sizeof(ngx_http_rust_main_conf_t));
  if (conf == NULL) {
    return NULL;
  }


  return conf;
}