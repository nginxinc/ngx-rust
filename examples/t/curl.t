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

my $t = Test::Nginx->new()->has(qw/http/)->plan(2)
	->write_file_expand('nginx.conf', <<"EOF");

%%TEST_GLOBALS%%

daemon off;

events {
}

http {
    %%TEST_GLOBALS_HTTP%%

    server {
        listen       127.0.0.1:8080;
        server_name  localhost;

        location / {
            curl on;
        }
    }
}

EOF

$t->write_file('index.html', '');
$t->run();

###############################################################################

like(get('/', 'curl/1.2.3'), qr/403 Forbidden/, 'curl UA forbidden');
like(get('/', 'MSIE 6.0'), qr/200 OK/, 'other UA allowed');

###############################################################################

sub get {
	my ($url, $ua, $extra) = @_;
	return http(<<EOF);
GET $url HTTP/1.1
Host: localhost
Connection: close
User-Agent: $ua

EOF
}

###############################################################################
