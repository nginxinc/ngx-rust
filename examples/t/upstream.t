#!/usr/bin/perl

# (C) Nginx, Inc

# Tests for ngx-rust example modules.

###############################################################################

use warnings;
use strict;

use Test::More;

BEGIN { use FindBin; chdir($FindBin::Bin); }

use lib 'lib';
use Test::Nginx;

###############################################################################

select STDERR; $| = 1;
select STDOUT; $| = 1;

my $t = Test::Nginx->new()->has(qw/http proxy/)->plan(2)
	->write_file_expand('nginx.conf', <<"EOF");

%%TEST_GLOBALS%%

daemon off;

events {
}

http {
    %%TEST_GLOBALS_HTTP%%

    upstream u {
        server 127.0.0.1:8081;
        custom 32;
    }

    server {
        listen       127.0.0.1:8080;
        server_name  localhost;

        error_log %%TESTDIR%%/e_debug.log debug;

        location / {
            proxy_pass http://u;
        }
    }

    server {
        listen       127.0.0.1:8081;
        server_name  localhost;

        location / { }
    }
}

EOF

$t->write_file('index.html', '');
$t->run();

###############################################################################

like(http_get('/'), qr/200 OK/, 'custom upstream');

$t->stop();

SKIP: {
	skip "no --with-debug", 1 unless $t->has_module('--with-debug');

	like($t->read_file('e_debug.log'), qr/CUSTOM UPSTREAM request/,
		'log - custom upstream');
}

###############################################################################
