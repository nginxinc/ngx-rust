ARG NGINX_VER
FROM nginx:$NGINX_VER

MAINTAINER Sehyo Chang "sehyo@nginx.com"


# install nginx required libraries

RUN apt-get update

# install dev en
RUN apt-get install curl wget unzip gnupg vim netcat make net-tools iptables procps -y

RUN curl -sL https://deb.nodesource.com/setup_8.x | bash - && \
    apt-get install -y nodejs

RUN apt-get install -y iproute2
